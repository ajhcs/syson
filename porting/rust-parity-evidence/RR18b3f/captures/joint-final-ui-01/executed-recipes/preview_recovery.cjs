// SPDX-License-Identifier: EPL-2.0
// Actual default-feature WASM/DOM journey; no substituted model or HTML output.
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const {chromium} = require(process.env.SYSON_PLAYWRIGHT_MODULE || 'playwright');
const url = process.env.SYSON_PREVIEW_URL, output = process.env.SYSON_BROWSER_ARTIFACT_DIR;
assert.ok(url && output);
const origin = new URL(url).origin;
assert.ok(['127.0.0.1', 'localhost', '[::1]'].includes(new URL(url).hostname));
assert.ok(path.resolve(output).startsWith('/workspace/'));
fs.mkdirSync(output, {recursive: true});
const expanding = fs.readFileSync(path.join(__dirname, 'fixtures/escaped-import.json'), 'utf8');
assert.equal(Buffer.byteLength(expanding), 700258);
const hugeName = JSON.parse(expanding).content[0].data.declaredName;
(async () => {
  const browser = await chromium.launch({headless: true});
  const results = [];
  try {
    for (const width of [320, 1440]) {
      const page = await browser.newPage({viewport: {width, height: 1000}, serviceWorkers: 'block', acceptDownloads: true});
      const errors = [], warnings = [];
      page.on('pageerror', e => errors.push(e.message));
      page.on('console', m => {if (m.type() === 'error') errors.push(m.text()); if (m.type() === 'warning') warnings.push(m.text());});
      await page.route('**/*', route => {
        const requested = new URL(route.request().url());
        if (requested.origin === origin || requested.protocol === 'data:') return route.continue();
        errors.push(`Unexpected external request: ${requested.href}`); return route.abort();
      });
      await page.addInitScript(() => {
        window.__rr18LongTasks = [];
        if (PerformanceObserver.supportedEntryTypes.includes('longtask')) {
          new PerformanceObserver(list => window.__rr18LongTasks.push(...list.getEntries().map(e => ({start: e.startTime, duration: e.duration}))))
            .observe({type: 'longtask', buffered: true});
        }
      });
      await page.goto(url);
      await page.getByRole('heading', {name: 'SysON model editor', exact: true}).waitFor();
      await page.locator('#name').fill('Before');
      await page.locator('#stage-create').click();
      await page.locator('#commit-staged').click();
      const model = () => page.locator('#export-json').inputValue();
      const geometry = () => page.locator('#export-geometry').inputValue();
      const layout = async () => {
        await page.getByRole('button', {name: 'Layout canvas', exact: true}).click();
        await page.waitForFunction(() => !document.querySelector('[data-canvas-ticket]') && document.getElementById('export-geometry')?.value);
      };
      await layout();
      await page.locator('.tree button').filter({hasText: /^BeforePackage$/}).click();
      await page.locator('#rename').fill('Keep rename draft');
      await page.locator('#name').fill('Keep creation draft');
      await page.locator('#geometry-details').evaluate(e => {e.open = true;});
      await page.locator('#import-geometry').fill('Keep geometry draft');
      const before = await model(), beforeGeometry = await geometry();
      const importer = page.locator('form[data-action="import"]');
      await importer.locator('..').evaluate(e => {e.open = true;});
      const rejected = [];
      for (const raw of ['not a document', expanding]) {
        await page.locator('#import-json').fill(raw);
        await page.locator('#import-json').evaluate(e => {e.focus(); e.setSelectionRange(2, 5, 'backward');});
        const started = Date.now();
        // Native form submission retains the focused textarea and invokes the
        // installed submit listener and actual WASM dispatch, not a mock handler.
        await importer.evaluate(form => form.requestSubmit());
        await page.getByRole('alert').waitFor();
        assert.equal(await model(), before);
        assert.equal(await geometry(), beforeGeometry);
        assert.equal(await page.locator('#rename').inputValue(), 'Keep rename draft');
        assert.equal(await page.locator('#name').inputValue(), 'Keep creation draft');
        assert.equal(await page.locator('#import-geometry').inputValue(), 'Keep geometry draft');
        assert.equal(await page.locator('#import-json').inputValue(), raw);
        assert.deepEqual(await page.locator('#import-json').evaluate(e => [e.selectionStart, e.selectionEnd, e.selectionDirection]), [2, 5, 'backward']);
        assert.equal(await page.evaluate(() => document.activeElement.id), 'import-json');
        assert.ok(await page.locator('#geometry-details').evaluate(e => e.open));
        rejected.push({bytes: Buffer.byteLength(raw), elapsedMs: Date.now() - started, message: await page.getByRole('alert').innerText()});
      }
      await page.getByRole('button', {name: 'Undo', exact: true}).click();
      assert.notEqual(await model(), before, 'prior undo history must survive refused import');
      await page.getByRole('button', {name: 'Redo', exact: true}).click();
      assert.equal(await model(), before);
      // An admitted opaque payload exercises a data-URI above 2 MiB; the
      // roughly900kB JSON download payload is separately counted below.
      const admitted = JSON.parse(before); admitted.opaque = 'x'.repeat(900000);
      await importer.locator('..').evaluate(e => {e.open = true;});
      await page.locator('#import-json').fill(JSON.stringify(admitted));
      await page.getByRole('button', {name: 'Import as new document and clear history', exact: true}).click();
      const committed = await model();
      assert.equal(JSON.parse(committed).opaque.length, 900000);
      await layout();
      const committedGeometry = await geometry();
      const expandStaged = async () => {
        await page.locator('.tree button').filter({hasText: /^BeforePackage$/}).click();
        await page.locator('#rename').fill(hugeName);
        await page.locator('#stage-rename').click();
        await page.locator('#recovery-export').waitFor();
      };
      await expandStaged();
      assert.match(await page.getByRole('alert').innerText(), /Preview exceeds/);
      const href = await page.locator('#recovery-export').getAttribute('href');
      assert.equal(decodeURIComponent(href.split(',').slice(1).join(',')), committed);
      const exported = page.waitForEvent('download');
      await page.locator('#recovery-export').focus(); await page.keyboard.press('Enter');
      const download = await exported;
      const downloaded = path.join(output, `recovery-export-${width}.json`);
      await download.saveAs(downloaded);
      assert.equal(await download.failure(), null);
      assert.equal(fs.readFileSync(downloaded, 'utf8'), committed);
      assert.ok(await page.evaluate(() => document.documentElement.scrollWidth <= document.documentElement.clientWidth));
      await page.screenshot({path: path.join(output, `recovery-${width}.png`)});
      const recover = page.getByRole('button', {name: 'Recover committed view (discard staged change)', exact: true});
      await recover.focus(); await page.keyboard.press('Enter');
      assert.equal(await model(), committed);
      assert.equal(await geometry(), committedGeometry);
      assert.equal(await page.evaluate(() => document.activeElement.id), 'name');
      await expandStaged();
      const discard = page.getByRole('button', {name: 'Discard current document', exact: true});
      await discard.focus(); await page.keyboard.press('Enter');
      assert.equal(await page.evaluate(() => document.activeElement.id), 'confirm-discard');
      await page.getByRole('button', {name: 'Keep document', exact: true}).focus(); await page.keyboard.press('Enter');
      assert.equal(decodeURIComponent((await page.locator('#recovery-export').getAttribute('href')).split(',').slice(1).join(',')), committed);
      await page.getByRole('button', {name: 'Discard current document', exact: true}).focus(); await page.keyboard.press('Enter');
      await page.getByRole('button', {name: 'Confirm discard document', exact: true}).focus(); await page.keyboard.press('Enter');
      assert.equal(JSON.parse(await model()).content.length, 0);
      assert.equal(await geometry(), '');
      assert.equal(await page.evaluate(() => document.activeElement.id), 'name');
      await page.getByRole('button', {name: 'Undo', exact: true}).click();
      assert.match(await page.getByRole('status').innerText(), /No history available/);
      assert.deepEqual(errors, []); assert.deepEqual(warnings, []);
      results.push({width, rejected, downloadBytes: Buffer.byteLength(committed), uriBytes: Buffer.byteLength(href), longTasks: await page.evaluate(() => window.__rr18LongTasks), errors, warnings});
      await page.close();
    }
    fs.writeFileSync(path.join(output, 'result.json'), JSON.stringify({status: 'PASS_SELECTED_DEFAULT_WASM_RECOVERY', results,
      limits: 'RR18a lifecycle/recovery only; full HTML allocation, generic fallback drafts, globalmemory, physical touch, other browsers and latency acceptance remain OPEN'}, null, 2));
  } finally {await browser.close();}
})().catch(error => {console.error(error); process.exitCode = 1;});
