// SPDX-License-Identifier: EPL-2.0
// RR20 rendered-control history journey plus A's existing dense ownership fixture.
// Actual WASM/native events only; no replaced model/HTML/command results.
const assert = require('node:assert/strict'), fs = require('node:fs'), path = require('node:path');
const {chromium} = require(process.env.SYSON_PLAYWRIGHT_MODULE || 'playwright');
const url = process.env.SYSON_PREVIEW_URL, output = process.env.SYSON_BROWSER_ARTIFACT_DIR;
assert.ok(url && output && path.resolve(output).startsWith('/workspace/'));
assert.ok(['127.0.0.1', 'localhost', '[::1]'].includes(new URL(url).hostname));
fs.mkdirSync(output, {recursive: true});
const denseOriginal = fs.readFileSync(path.join(__dirname, '../../RR18b3a/fixtures/ownership-amplification.json'), 'utf8');
assert.equal(Buffer.byteLength(denseOriginal), 192234);
const opaque = '"jointOpaque":{"minus":-0,"exponent":1e+0003,"huge":12345678901234567890123456789012345678901234567890,"marker":{"$serde_json::private::Number":"not-a-number"},"text":"\\u0000雪🧭\\r\\n<&>"}';
const dense = denseOriginal.trimEnd().slice(0, -1) + ',' + opaque + '}';
assert.ok(Buffer.byteLength(dense) < 1048576);
const literalTokens = text => {
  for (const token of ['"minus":-0,', '"exponent":1e+0003,', '"huge":12345678901234567890123456789012345678901234567890,', '"marker":{"$serde_json::private::Number":"not-a-number"}']) assert.ok(text.includes(token), token);
  assert.equal(JSON.parse(text).jointOpaque.text, '\0雪🧭\r\n<&>');
};
const summary = samples => {
  const sorted = [...samples].sort((a, b) => a - b);
  return {minMs: sorted[0], medianMs: sorted[Math.floor(sorted.length / 2)], maxMs: sorted.at(-1)};
};
(async () => {
  const browser = await chromium.launch({headless: true}), results = [];
  try {
    for (const width of [320, 1440]) {
      const page = await browser.newPage({viewport: {width, height: 1000}, serviceWorkers: 'block'}), errors = [], warnings = [];
      page.setDefaultTimeout(30000);
      page.on('pageerror', error => errors.push(error.message));
      page.on('console', message => {if (message.type() === 'error') errors.push(message.text()); if (message.type() === 'warning') warnings.push(message.text());});
      await page.route('**/*', route => {
        const requested = new URL(route.request().url());
        if (requested.origin === new URL(url).origin || requested.protocol === 'data:') return route.continue();
        errors.push(`Unexpected external request: ${requested.href}`); return route.abort();
      });
      await page.addInitScript(() => {
        window.__jointHistoryTasks = []; window.__jointHistoryOverflow = false;
        if (PerformanceObserver.supportedEntryTypes.includes('longtask')) new PerformanceObserver(list => {
          for (const entry of list.getEntries()) {
            if (window.__jointHistoryTasks.length === 512) window.__jointHistoryOverflow = true;
            else window.__jointHistoryTasks.push({start: entry.startTime, duration: entry.duration});
          }
        }).observe({type: 'longtask', buffered: true});
      });
      await page.goto(url); await page.getByRole('heading', {name: 'SysON model editor', exact: true}).waitFor();
      assert.equal(await page.title(), 'SysON · Rust model editor');
      const model = () => page.locator('#export-json').inputValue();
      const keyboardAction = async id => {await page.locator(id).focus(); await page.keyboard.press('Enter');};
      const reversibleUndoSamples = async (current, prior) => {
        const values = [];
        for (let n = 0; n < 9; n++) {
          // Same committed model and undo/redo boundary before each measured
          // action. Redo restores outside the native event timing window.
          assert.equal(await model(), current);
          await page.locator('#model-undo').focus();
          await page.evaluate(() => {
            window.__jointHistoryTiming = new Promise(resolve => {
              document.addEventListener('click', event => {
                const action = event.target.closest?.('button[data-action]')?.dataset.action;
                const start = performance.now();
                requestAnimationFrame(() => requestAnimationFrame(() => resolve({action, milliseconds: performance.now() - start})));
              }, {capture: true, once: true});
            });
          });
          await page.keyboard.press('Enter'); const measured = await page.evaluate(() => window.__jointHistoryTiming);
          assert.equal(measured.action, 'undo'); assert.equal(await model(), prior); values.push(measured.milliseconds);
          await keyboardAction('#model-redo'); assert.equal(await model(), current);
        }
        return {metric: 'Native undo click event to two animation frames; redo restores identical model/history preconditions outside timing', warmupsMs: values.slice(0, 2), measuredMs: values.slice(2), ...summary(values.slice(2))};
      };
      await page.locator('#kind').selectOption('package'); await page.locator('#name').fill('Revision0');
      await page.locator('#stage-create').click(); await page.locator('#commit-staged').click();
      const packageId = JSON.parse(await model()).content[0].data.elementId;
      await page.locator(`button[data-value="${packageId}"]`).click();
      const name = async () => JSON.parse(await model()).content[0].data.declaredName;
      const commitTrajectoryMs = []; let revision69;
      for (let n = 1; n <= 70; n++) {
        const before = await model(), start = Date.now();
        await page.locator('#rename').fill(`Revision${n}`); await page.locator('#stage-rename').click();
        assert.equal(await model(), before); await page.locator('#commit-staged').click();
        assert.equal(await name(), `Revision${n}`); commitTrajectoryMs.push(Date.now() - start);
        if (n === 69) revision69 = await model();
      }
      const final = await model();
      const sparseUndo = await reversibleUndoSamples(final, revision69);
      for (let n = 69; n >= 6; n--) {await keyboardAction('#model-undo'); assert.equal(await name(), `Revision${n}`);}
      const boundary = await model(); await keyboardAction('#model-undo'); assert.equal(await model(), boundary);
      assert.match(await page.locator('#editor-status').innerText(), /No history available/);
      const boundaryFocus = await page.evaluate(() => {const e = document.activeElement, r = e.getBoundingClientRect(); return {id: e.id, top: r.top, bottom: r.bottom, visible: r.bottom > 0 && r.top < innerHeight};});
      assert.equal(boundaryFocus.id, 'model-undo'); assert.ok(boundaryFocus.visible);
      await page.screenshot({path: path.join(output, `joint-history-boundary-${width}.png`)});
      for (let n = 7; n <= 70; n++) {await keyboardAction('#model-redo'); assert.equal(await name(), `Revision${n}`);}
      assert.equal(await model(), final); await keyboardAction('#model-redo'); assert.equal(await model(), final);
      // Import the unchanged A fixture augmented only with exact literal-token
      // counterexamples. UI import clears previous history by existing contract.
      const importer = page.locator('form[data-action="import"]'); await importer.locator('..').evaluate(e => {e.open = true;});
      await page.locator('#import-json').fill(dense); await importer.evaluate(form => form.requestSubmit());
      const denseBefore = await model(); literalTokens(denseBefore);
      assert.equal(await page.locator('ul[aria-label="Model hierarchy"] button[data-value^="child-"]').count(), 512);
      assert.equal(await page.locator('#model-relationships button[data-value^="membership-"]').count(), 512);
      await page.locator('#model-relationships > summary').focus(); await page.keyboard.press('Enter');
      await page.locator('button[data-value="membership-0000"]').focus(); await page.keyboard.press('Enter');
      await page.getByRole('heading', {name: 'OwningMembership', exact: true}).waitFor(); assert.equal(await model(), denseBefore);
      await page.locator('button[data-value="child-0511"]').focus(); await page.keyboard.press('Enter');
      assert.equal(await page.locator('#rename').inputValue(), 'part0511');
      await page.locator('#rename').fill('Joint changed 雪🧭'); await page.locator('#stage-rename').click();
      assert.equal(await model(), denseBefore); await page.locator('#commit-staged').click();
      const denseChanged = await model(); literalTokens(denseChanged);
      const oldName = '"declaredName":"part0511"'; assert.equal(denseBefore.split(oldName).length, 2);
      assert.equal(denseChanged, denseBefore.replace(oldName, '"declaredName":"Joint changed 雪🧭"'));
      const denseUndo = await reversibleUndoSamples(denseChanged, denseBefore);
      const owner = JSON.parse(denseBefore).content[0].data.elementId; assert.equal(owner.length, 65542);
      const ownerButton = page.locator('button[data-action="select"]').filter({hasText: /^OwnerPackage$/});
      await ownerButton.focus(); await page.keyboard.press('Enter'); assert.equal(await page.locator('#rename').inputValue(), 'Owner');
      await page.locator('#element-identity > summary').focus(); await page.keyboard.press('Enter');
      assert.equal(await page.locator('#element-identity .element-id').innerText(), owner); assert.equal(await model(), denseChanged);
      const pageBounds = await page.evaluate(() => ({client: document.documentElement.clientWidth, scroll: document.documentElement.scrollWidth}));
      assert.ok(pageBounds.scroll <= pageBounds.client); assert.deepEqual(errors, []); assert.deepEqual(warnings, []);
      assert.equal(await page.evaluate(() => window.__jointHistoryOverflow), false);
      await page.screenshot({path: path.join(output, `joint-history-dense-${width}.png`)});
      results.push({width, sparse: {commits: 70, undo: 64, redo: 64, boundaryName: 'Revision6', boundaryFocus, undoTiming: sparseUndo,
        commitTrajectoryMs, trajectoryMetric: 'Automation-inclusive observations across changing history states; not interchangeable p95/performance samples'},
        dense: {fixtureOriginalBytes: 192234, admittedInputBytes: Buffer.byteLength(dense), rows: 1025, children: 512, relationships: 512,
          ownerCodeUnits: owner.length, committedBytes: Buffer.byteLength(denseChanged), exactLiteralAuthority: true, undoTiming: denseUndo},
        pageBounds, errors, warnings, longTasks: await page.evaluate(() => window.__jointHistoryTasks),
        limits: 'Existing RR20/A joint caller qualification only; local retained charge is not RSS/durable history. Generic boundary copy/disabled availability remains RR19. Dense tree virtualization, glyphs, durable project reopen, complete renderer admission and native SysON parity remain open.'});
      await page.close();
    }
    fs.writeFileSync(path.join(output, 'history-summary-ui.json'), JSON.stringify({status: 'PASS_BOUNDED_RR20_BORROWED_SUMMARY_COMPOSITION', results}, null, 2));
  } finally {await browser.close();}
})().catch(error => {console.error(error); process.exitCode = 1;});
