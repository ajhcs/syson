// SPDX-License-Identifier: EPL-2.0
// Actual native controls/WASM/AX; passive timing observers do not replace handlers.
const assert = require('node:assert/strict'), fs = require('node:fs'), path = require('node:path');
const {chromium} = require(process.env.SYSON_PLAYWRIGHT_MODULE || 'playwright');
const url = process.env.SYSON_PREVIEW_URL, output = process.env.SYSON_BROWSER_ARTIFACT_DIR;
assert.ok(url && output && path.resolve(output).startsWith('/workspace/'));
assert.ok(['127.0.0.1', 'localhost', '[::1]'].includes(new URL(url).hostname));
fs.mkdirSync(output, {recursive: true});
const longName = 'Long <&> 雪🧭 ' + 'x'.repeat(4096) + ' END';
const dense = JSON.parse(fs.readFileSync(path.join(__dirname, '../RR18b3a/fixtures/ownership-amplification.json'), 'utf8'));
const denseNode = dense.content[0].data.ownedRelationship[511].data.ownedRelatedElement[0].data;
denseNode.declaredName = longName;
const denseId = denseNode.elementId;
const summary = values => {
  const sorted = [...values].sort((a, b) => a - b);
  return {samples: sorted.length, minMs: sorted[0], medianMs: sorted[Math.floor(sorted.length / 2)], maxMs: sorted.at(-1)};
};
(async () => {
  const browser = await chromium.launch({headless: true}), results = [];
  try {
    for (const width of [320, 1440]) {
      const page = await browser.newPage({viewport: {width, height: 1000}, serviceWorkers: 'block'});
      const errors = [], warnings = [], observations = [];
      page.setDefaultTimeout(30000);
      page.on('pageerror', error => errors.push(error.message));
      page.on('console', message => {if (message.type() === 'error') errors.push(message.text()); if (message.type() === 'warning') warnings.push(message.text());});
      await page.route('**/*', route => {
        const requested = new URL(route.request().url());
        if (requested.origin === new URL(url).origin || requested.protocol === 'data:') return route.continue();
        errors.push(`Unexpected external request: ${requested.href}`); return route.abort();
      });
      await page.addInitScript(() => {
        window.__navSamples = []; window.__navTasks = []; window.__measureNav = false; window.__navOverflow = false;
        document.addEventListener('click', event => {
          if (!window.__measureNav || event.target.closest?.('button[data-action]')?.dataset.action !== 'select') return;
          const started = performance.now();
          requestAnimationFrame(() => requestAnimationFrame(() => {
            if (window.__navSamples.length >= 32) window.__navOverflow = true;
            else window.__navSamples.push(performance.now() - started);
          }));
        }, true);
        if (PerformanceObserver.supportedEntryTypes.includes('longtask')) new PerformanceObserver(list => {
          for (const entry of list.getEntries()) {
            if (window.__navTasks.length >= 256) window.__navOverflow = true;
            else window.__navTasks.push({start: entry.startTime, duration: entry.duration});
          }
        }).observe({type: 'longtask', buffered: true});
      });
      const focusState = () => page.evaluate(() => {
        const e = document.activeElement, r = e.getBoundingClientRect();
        return {id: e.id, top: r.top, bottom: r.bottom, left: r.left, right: r.right, width: r.width, height: r.height,
          visible: r.width > 0 && r.height > 0 && r.bottom > 0 && r.top < innerHeight && r.right > 0 && r.left < innerWidth,
          caret: typeof e.selectionStart === 'number' ? [e.selectionStart, e.selectionEnd, e.selectionDirection] : null,
          page: {left: scrollX, top: scrollY}, local: {left: e.scrollLeft, top: e.scrollTop}};
      });
      const requireRepaint = async (selector, operation) => {
        const previous = await page.locator(selector).elementHandle(); assert.ok(previous);
        await operation();
        assert.equal(await previous.evaluate(element => element.isConnected), false, `No repaint replaced ${selector}`);
        await previous.dispose();
      };
      const model = () => page.locator('#export-json').inputValue();
      const importer = () => page.locator('form[data-action="import"]');
      const importDocument = async document => {
        await importer().locator('..').evaluate(element => {element.open = true;});
        await page.locator('#import-json').fill(JSON.stringify(document));
        await importer().evaluate(form => form.requestSubmit());
        assert.match(await page.locator('#editor-status').innerText(), /Imported a new document/);
      };
      const session = await page.context().newCDPSession(page); await session.send('Accessibility.enable');
      const axButton = async id => {
        const dom = await session.send('DOM.getDocument', {depth: 0});
        const {nodeId} = await session.send('DOM.querySelector', {nodeId: dom.root.nodeId, selector: `button[data-value="${id}"]`});
        const {nodes} = await session.send('Accessibility.getPartialAXTree', {nodeId, fetchRelatives: false});
        const node = nodes.find(n => !n.ignored && n.role?.value === 'button'); assert.ok(node);
        assert.ok(node.name.value.includes(longName)); assert.ok(node.name.value.includes(id === 'nav0' ? 'Package' : 'PartUsage'));
        return {role: node.role.value, name: node.name.value, description: node.description?.value};
      };
      await page.goto(url); await page.getByRole('heading', {name: 'SysON model editor', exact: true}).waitFor();
      assert.equal(await page.title(), 'SysON · Rust model editor');
      const fixture = JSON.parse(await model());
      fixture.content = [{id: 'nav-resource', eClass: 'sysml:Package', data: {elementId: 'nav0', declaredName: longName}}];
      for (const [name, document, id, expectedChildren] of [['sparse', fixture, 'nav0', 1], ['dense', dense, denseId, 513]]) {
        await importDocument(document);
        const before = await model(), button = () => page.locator(`button[data-value="${id}"]`);
        assert.equal(await page.locator('ul[aria-label="Model hierarchy"] button').count(), expectedChildren);
        await page.evaluate(() => scrollTo(0, 0));
        const bounds = await button().evaluate(element => {
          const r = element.getBoundingClientRect();
          return {height: r.height, width: r.width, text: element.textContent,
            pageWidth: document.documentElement.clientWidth, scrollWidth: document.documentElement.scrollWidth};
        });
        fs.writeFileSync(path.join(output, `navigation-${name}-${width}.json`), JSON.stringify(bounds, null, 2));
        await page.screenshot({path: path.join(output, `navigation-${name}-${width}.png`)});
        assert.ok(bounds.height <= 112, `Unbounded navigation label: ${bounds.height}px`);
        assert.ok(bounds.scrollWidth <= bounds.pageWidth); assert.ok(bounds.text.includes(longName));
        const accessibility = await axButton(id);
        await button().focus();
        const focusedNavigation = await focusState(); assert.ok(focusedNavigation.visible);
        await page.screenshot({path: path.join(output, `navigation-focused-${name}-${width}.png`)});
        await page.keyboard.press('Enter');
        assert.equal(await page.locator('#rename').inputValue(), longName);
        assert.equal(await model(), before);
        await page.locator('#rename').fill('Changed exact name'); await page.locator('#stage-rename').click();
        assert.equal(await model(), before); await page.locator('#commit-staged').click();
        const changed = await model(); assert.notEqual(changed, before);
        await page.locator('#model-undo').click(); assert.equal(await model(), before);
        await page.locator('#model-redo').click(); assert.equal(await model(), changed);
        await page.locator('#model-undo').click(); assert.equal(await model(), before);
        await page.evaluate(() => {window.__navSamples = []; window.__measureNav = true;});
        for (let sample = 0; sample < 9; sample++) {
          await button().focus(); await page.keyboard.press('Enter');
          await page.waitForFunction(count => window.__navSamples.length === count, sample + 1);
        }
        await page.evaluate(() => {window.__measureNav = false;});
        const samples = await page.evaluate(() => window.__navSamples);
        observations.push({workload: name, bounds, accessibility, focusedNavigation, warmups: samples.slice(0, 2), eventToTwoFrames: summary(samples.slice(2)), measuredSamplesMs: samples.slice(2)});
        assert.equal(await model(), before);
      }
      // Unknown imported classes are bounded opaque passthrough, not native SysON class support.
      const unknownName = 'Readonly unknown <&> 雪🧭\r\n' + 'r'.repeat(4096);
      const unknown = JSON.parse(await model());
      unknown.content = [{id: 'unknown-resource', eClass: 'sysml:UnknownClass', data: {elementId: 'unknown0', declaredName: unknownName}}];
      await importDocument(unknown); const unknownCommitted = await model();
      await page.locator('button[data-value="unknown0"]').focus(); await page.keyboard.press('Enter');
      assert.equal(await page.locator('form[data-action="rename"]').count(), 0);
      const fullReadonly = page.getByRole('textbox', {name: 'Full element name (read only)', exact: true});
      assert.equal(await fullReadonly.inputValue(), unknownName.replace(/\r\n?/g, '\n'));
      assert.equal(await fullReadonly.evaluate(element => element.readOnly), true);
      await fullReadonly.focus(); await page.keyboard.press('Control+A');
      const readonlyNameFocus = await focusState(); assert.equal(readonlyNameFocus.id, 'full-element-name'); assert.ok(readonlyNameFocus.visible);
      assert.deepEqual(readonlyNameFocus.caret.slice(0, 2), [0, unknownName.replace(/\r\n?/g, '\n').length]);
      await page.keyboard.press('Backspace'); assert.equal(await fullReadonly.inputValue(), unknownName.replace(/\r\n?/g, '\n'));
      await requireRepaint('#full-element-name', () => page.evaluate(() => document.fonts.dispatchEvent(new Event('loadingdone'))));
      const readonlyNameAfter = await focusState(); assert.equal(readonlyNameAfter.id, 'full-element-name'); assert.ok(readonlyNameAfter.visible);
      assert.deepEqual(readonlyNameAfter.caret, readonlyNameFocus.caret); assert.deepEqual(readonlyNameAfter.local, readonlyNameFocus.local);
      assert.equal(await model(), unknownCommitted); assert.deepEqual(JSON.parse(unknownCommitted), unknown);
      const readonlyDom = await session.send('DOM.getDocument', {depth: 0});
      const readonlyNode = await session.send('DOM.querySelector', {nodeId: readonlyDom.root.nodeId, selector: '#full-element-name'});
      const readonlyTree = await session.send('Accessibility.getPartialAXTree', {nodeId: readonlyNode.nodeId, fetchRelatives: false});
      const readonlyAX = readonlyTree.nodes.find(node => !node.ignored && node.role?.value === 'textbox');
      assert.ok(readonlyAX); assert.equal(readonlyAX.name.value, 'Full element name (read only)');
      assert.ok(readonlyAX.properties.some(property => property.name === 'readonly' && property.value?.value === true));
      await page.screenshot({path: path.join(output, `navigation-readonly-name-${width}.png`)});
      const unknownPassthrough = {scope: 'Imported unknown class retained opaquely; not native SysON semantic parity', readonlyNameFocus, readonlyNameAfter,
        accessibility: {role: readonlyAX.role.value, name: readonlyAX.name.value, readonly: true}, exactCanonicalModelRetained: true};
      // Work with sparse state for focus proofs; dense state was independently measured above.
      await importDocument(fixture); await page.locator('button[data-value="nav0"]').click();
      const committed = await model();
      await page.locator('#rename').fill('Unsaved inspector draft');
      await importer().locator('..').evaluate(element => {element.open = true;});
      const invalid = '\n\nnot valid JSON 雪🧭\n'; await page.locator('#import-json').fill(invalid);
      await page.locator('#import-json').evaluate(element => {element.focus(); element.setSelectionRange(2, 7, 'backward');});
      const originalFocus = await focusState(); assert.ok(originalFocus.visible);
      await requireRepaint('#import-json', () => importer().evaluate(form => form.requestSubmit()));
      const rejected = await focusState();
      assert.equal(rejected.id, 'import-json'); assert.ok(rejected.visible, JSON.stringify(rejected));
      assert.deepEqual(rejected.caret, [2, 7, 'backward']);
      assert.equal(await page.locator('#import-json').inputValue(), invalid);
      assert.equal(await page.locator('#rename').inputValue(), 'Unsaved inspector draft'); assert.equal(await model(), committed);
      await page.screenshot({path: path.join(output, `focus-rejected-${width}.png`)});
      await requireRepaint('#import-json', () => page.evaluate(() => document.fonts.dispatchEvent(new Event('loadingdone'))));
      const font = await focusState(); assert.equal(font.id, 'import-json'); assert.ok(font.visible, JSON.stringify(font));
      assert.deepEqual(font.caret, [2, 7, 'backward']); assert.equal(await model(), committed);
      await page.screenshot({path: path.join(output, `focus-font-${width}.png`)});
      // User scrolling away from a focused control must not trigger a page jump.
      await page.evaluate(() => scrollTo(0, 0)); const deliberatelyAway = await focusState(); assert.equal(deliberatelyAway.visible, false);
      await requireRepaint('#import-json', () => page.evaluate(() => document.fonts.dispatchEvent(new Event('loadingdone'))));
      const stayedAway = await focusState(); assert.equal(stayedAway.id, 'import-json');
      assert.equal(stayedAway.visible, false); assert.deepEqual(stayedAway.page, deliberatelyAway.page);
      // Readonly focus retains native selection, without retaining/copying its value as a draft.
      await page.locator('#export-json').evaluate(element => {element.focus(); element.setSelectionRange(2, 17, 'backward');});
      await page.locator('#export-json').evaluate(e => {e.scrollLeft = 60; e.scrollTop = 15;});
      const readonlyBefore = await focusState(); assert.ok(readonlyBefore.visible);
      const panBefore = await page.locator('[data-canvas-viewport]').evaluate(e => {e.scrollLeft = 9; e.scrollTop = 3; return {left: e.scrollLeft, top: e.scrollTop};});
      await requireRepaint('#export-json', () => page.evaluate(() => document.fonts.dispatchEvent(new Event('loadingdone'))));
      const readonlyAfter = await focusState(); assert.equal(readonlyAfter.id, 'export-json'); assert.ok(readonlyAfter.visible);
      assert.deepEqual(readonlyAfter.caret, [2, 17, 'backward']); assert.deepEqual(readonlyAfter.local, readonlyBefore.local); assert.equal(await model(), committed);
      const panAfter = await page.locator('[data-canvas-viewport]').evaluate(e => ({left: e.scrollLeft, top: e.scrollTop}));
      assert.deepEqual(panAfter, panBefore);
      await page.screenshot({path: path.join(output, `focus-readonly-${width}.png`)});
      assert.deepEqual(errors, []); assert.deepEqual(warnings, []);
      assert.equal(await page.evaluate(() => window.__navOverflow), false);
      results.push({width, url: page.url(), title: await page.title(), observations, unknownPassthrough,
        focus: {originalFocus, rejected, font, deliberatelyAway, stayedAway, readonlyBefore, readonlyAfter}, panBefore, panAfter,
        errors, warnings, longTasks: await page.evaluate(() => window.__navTasks),
        performance: 'Two warmups plus seven measured event-to-two-frame samples per sparse/dense workload; report min/median/max and raw values. Workload ranges are diagnostic/user expectations, not a universal50ms veto.',
        limits: 'Chromium/headless at two viewports; exact DOM/AX/model text, not glyph/native SysON/device or complete renderer acceptance.'});
      await session.detach(); await page.close();
    }
    fs.writeFileSync(path.join(output, 'navigation-focus.json'), JSON.stringify({status: 'PASS_SELECTED_NAVIGATION_AND_VISIBLE_FOCUS', results}, null, 2));
  } finally {await browser.close();}
})().catch(error => {console.error(error); process.exitCode = 1;});
