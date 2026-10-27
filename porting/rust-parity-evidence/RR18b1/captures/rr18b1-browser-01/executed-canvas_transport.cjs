// SPDX-License-Identifier: EPL-2.0
// Root-only actual default-WASM journey. Instrumentation calls original exports.
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const {chromium} = require(process.env.SYSON_PLAYWRIGHT_MODULE || 'playwright');
const url = process.env.SYSON_PREVIEW_URL, output = process.env.SYSON_BROWSER_ARTIFACT_DIR;
assert.ok(url && output && path.resolve(output).startsWith('/workspace/'));
const origin = new URL(url).origin;
assert.ok(['127.0.0.1', 'localhost', '[::1]'].includes(new URL(url).hostname));
fs.mkdirSync(output, {recursive: true});
(async () => {
  const browser = await chromium.launch({headless: true});
  const results = [];
  try {
    for (const width of [320, 1440]) {
      const page = await browser.newPage({viewport: {width, height: 1000}, serviceWorkers: 'block'});
      const errors = [], warnings = [];
      page.on('pageerror', e => errors.push(e.message));
      page.on('console', m => {if (m.type() === 'error') errors.push(m.text()); if (m.type() === 'warning') warnings.push(m.text());});
      await page.route('**/*', route => {
        if (new URL(route.request().url()).origin === origin) return route.continue();
        errors.push(`Unexpected external request: ${route.request().url()}`); return route.abort();
      });
      await page.addInitScript(() => {
        window.__transport = {dispatch: [], longTasks: []};
        if (PerformanceObserver.supportedEntryTypes.includes('longtask')) {
          new PerformanceObserver(list => window.__transport.longTasks.push(...list.getEntries().map(e => ({start: e.startTime, duration: e.duration}))))
            .observe({type: 'longtask', buffered: true});
        }
        const original = WebAssembly.instantiate;
        WebAssembly.instantiate = async (...args) => {
          const result = await original(...args);
          if (!result.instance) return result;
          const api = result.instance.exports, input = new Uint8Array(1048576);
          let size = 0;
          // Capture the real action before the timer. Rust dispatch still owns
          // mutation and rendering; no parser/model/geometry result is replaced.
          const exports = {...api,
            input_reset() {size = 0; return api.input_reset();},
            input_byte(byte) {if (size >= input.length) throw Error('probe input limit'); input[size++] = byte; return api.input_byte(byte);},
            dispatch() {
              const action = JSON.parse(new TextDecoder().decode(input.subarray(0, size))).action;
              const start = performance.now(); api.dispatch(); const elapsed = performance.now() - start;
              window.__transport.dispatch.push({action, elapsedMs: elapsed, outputBytes: api.output_len(), wasmPages: api.memory.buffer.byteLength / 65536});
            }};
          return {...result, instance: {exports}};
        };
      });
      await page.goto(url);
      await page.getByRole('heading', {name: 'SysON model editor', exact: true}).waitFor();
      await page.locator('#name').fill('Vehicle');
      await page.locator('#stage-create').click(); await page.locator('#commit-staged').click();
      const model = () => page.locator('#export-json').inputValue();
      const geometry = () => page.locator('#export-geometry').inputValue();
      const importer = page.locator('form[data-action="import"]');
      const importModel = async raw => {
        await importer.locator('..').evaluate(e => {e.open = true;});
        await page.locator('#import-json').fill(raw);
        await importer.evaluate(form => form.requestSubmit());
      };
      const layout = async (refused = false) => {
        await page.getByRole('button', {name: 'Layout canvas', exact: true}).click();
        await page.waitForFunction(refused => !document.querySelector('[data-canvas-ticket]') &&
          (refused ? document.getElementById('editor-status').textContent.includes('byte budget') : document.getElementById('export-geometry').value.length > 0), refused);
      };
      const raw = JSON.parse(await model()); raw.vendor = {opaque: 'x'.repeat(900000)};
      await importModel(JSON.stringify(raw));
      const committed = await model(); assert.equal(JSON.parse(committed).vendor.opaque.length, 900000);
      await layout();
      const saved = await geometry(); assert.equal(JSON.parse(saved).model, committed);
      await page.getByRole('button', {name: 'Canvas undo', exact: true}).click(); assert.equal(await geometry(), '');
      await page.getByRole('button', {name: 'Canvas redo', exact: true}).click(); assert.equal(await geometry(), saved);
      await page.locator('.tree button').filter({hasText: /^VehiclePackage$/}).click();
      assert.equal(await geometry(), saved);
      await page.getByRole('button', {name: 'Rotate preview', exact: true}).click();
      await page.waitForFunction(saved => !document.querySelector('[data-canvas-ticket]') && document.getElementById('export-geometry').value !== saved, saved);
      const rotated = JSON.parse(await geometry()), original = JSON.parse(saved);
      assert.equal(rotated.model, committed); assert.equal(await model(), committed);
      assert.equal(rotated.nodes.length, 1); assert.equal(original.nodes.length, 1);
      assert.equal(rotated.nodes[0].width, original.nodes[0].height);
      assert.equal(rotated.nodes[0].height, original.nodes[0].width);
      await page.getByRole('button', {name: 'Canvas undo', exact: true}).click();
      assert.equal(await geometry(), saved);
      await page.locator('#rename').fill('Draft'); await page.locator('#stage-rename').click();
      assert.equal(await model(), committed); assert.equal(await geometry(), saved);
      await page.getByRole('button', {name: 'Cancel', exact: true}).click(); assert.equal(await geometry(), saved);
      await page.locator('#geometry-details').evaluate(e => {e.open = true;});
      await page.locator('#import-geometry').fill(saved);
      await page.locator('form[data-action="geometry_import"]').evaluate(form => form.requestSubmit());
      assert.equal(await model(), committed); assert.equal(await geometry(), saved);
      await page.locator('#rename').fill('Changed'); await page.locator('#stage-rename').click(); await page.locator('#commit-staged').click();
      assert.equal(await geometry(), '');
      await page.getByRole('button', {name: 'Undo', exact: true}).click(); assert.equal(await model(), committed); assert.equal(await geometry(), '');
      const escaped = JSON.parse(committed); escaped.vendor.opaque = '\\'.repeat(150000);
      await importModel(JSON.stringify(escaped)); const beforeRefusal = await model();
      await layout(true); assert.equal(await model(), beforeRefusal); assert.equal(await geometry(), '');
      assert.equal(await page.locator('[data-canvas-ticket]').count(), 0);
      assert.ok(await page.evaluate(() => document.documentElement.scrollWidth <= document.documentElement.clientWidth));
      assert.deepEqual(errors, []); assert.deepEqual(warnings, []);
      const observations = await page.evaluate(() => window.__transport);
      for (const action of ['layout', 'rotate', 'canvas_compute', 'canvas_apply', 'canvas_undo', 'canvas_redo', 'geometry_import']) {
        assert.ok(observations.dispatch.some(event => event.action === action), `missing actual ${action}`);
      }
      await page.screenshot({path: path.join(output, `transport-${width}.png`)});
      results.push({width, committedBytes: Buffer.byteLength(committed), geometryBytes: Buffer.byteLength(saved), refusedModelBytes: Buffer.byteLength(beforeRefusal), observations, errors, warnings});
      await page.close();
    }
    fs.writeFileSync(path.join(output, 'result.json'), JSON.stringify({status: 'PASS_SELECTED_CANVAS_TRANSPORT', results,
      limits: 'Dispatch times include original Rust handling/rendering but exclude JavaScript input capture and DOM painting. Instrumentation may affect whole-task timings; WASM pages are high-water address-space observations, not live RSS. Full HTML allocation, cancellation latency and <50ms composed UI acceptance remain OPEN RR18b3.'}, null, 2));
  } finally {await browser.close();}
})().catch(error => {console.error(error); process.exitCode = 1;});
