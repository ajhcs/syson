// SPDX-License-Identifier: EPL-2.0
// Native controls/default WASM; observations never replace product results.
// Browser qualification requires the exact E+F composition, not F alone.
const assert = require('node:assert/strict'), fs = require('node:fs'), path = require('node:path');
const {chromium} = require(process.env.SYSON_PLAYWRIGHT_MODULE || 'playwright');
const url = process.env.SYSON_PREVIEW_URL, output = process.env.SYSON_BROWSER_ARTIFACT_DIR;
assert.ok(url && output && path.resolve(output).startsWith('/workspace/'));
assert.ok(['127.0.0.1', 'localhost', '[::1]'].includes(new URL(url).hostname));
fs.mkdirSync(output, {recursive: true});
const body = '\n\nReader body 雪🧭\r\nsecond\tline\0<&>\"\'\n' + '&'.repeat(850000);
const aliases = ['Readonly alias 雪\r\n<&>\"'.repeat(100)];
const stats = samples => {
  const sorted = [...samples].sort((a, b) => a - b);
  return {samples, min: sorted[0], median: sorted[Math.floor(sorted.length / 2)], max: sorted.at(-1)};
};
(async () => {
  const browser = await chromium.launch({headless: true}), results = [];
  try {
    for (const width of [320, 1440]) {
      const page = await browser.newPage({viewport: {width, height: 1000}, serviceWorkers: 'block', acceptDownloads: true});
      const errors = [], warnings = []; page.setDefaultTimeout(30000);
      page.on('pageerror', error => errors.push(error.message));
      page.on('console', message => {if (message.type() === 'error') errors.push(message.text()); if (message.type() === 'warning') warnings.push(message.text());});
      await page.route('**/*', route => {
        const request = new URL(route.request().url());
        if (request.origin === new URL(url).origin || request.protocol === 'data:') return route.continue();
        errors.push(`Unexpected external request: ${request.href}`); return route.abort();
      });
      await page.addInitScript(() => {
        window.__savedTasks = [];
        window.__savedLongTaskSupported = PerformanceObserver.supportedEntryTypes.includes('longtask');
        if (window.__savedLongTaskSupported) {
          const observer = new PerformanceObserver(list => window.__savedTasks.push(...list.getEntries().map(e => ({start: e.startTime, duration: e.duration}))));
          observer.observe({type: 'longtask', buffered: true});
          window.__savedDrain = () => window.__savedTasks.push(...observer.takeRecords().map(e => ({start: e.startTime, duration: e.duration})));
        }
      });
      try {
        await page.goto(url);
        await page.getByRole('heading', {name: 'SysON model editor', exact: true}).waitFor();
        assert.equal(await page.title(), 'SysON · Rust model editor');
        const model = () => page.locator('#export-json').inputValue();
        const readonlyRepaint = async (id, expected, action) => {
          const control = page.locator(`#${id}`);
          await control.evaluate(element => {element.focus(); element.setSelectionRange(2, 17, 'backward'); element.scrollLeft = 70;});
          const state = () => page.evaluate(() => {
            const e = document.activeElement, r = e.getBoundingClientRect();
            return {id: e.id, caret: [e.selectionStart, e.selectionEnd, e.selectionDirection],
              local: [e.scrollLeft, e.scrollTop], visible: r.bottom > 0 && r.top < innerHeight && r.right > 0 && r.left < innerWidth};
          });
          const before = await state(); assert.equal(before.id, id); assert.equal(before.visible, true);
          const previous = await control.elementHandle(); assert.ok(previous);
          assert.equal(await previous.evaluate(element => element.isConnected), true);
          if (action === 'font') await page.evaluate(() => document.fonts.dispatchEvent(new Event('loadingdone')));
          else {
            await page.locator('form[data-action="import"]').locator('..').evaluate(element => {element.open = true;});
            await page.locator('#import-json').fill('invalid readonly repaint');
            await control.evaluate(element => {element.focus(); element.setSelectionRange(2, 17, 'backward'); element.scrollLeft = 70;});
            await page.locator('form[data-action="import"]').evaluate(form => form.requestSubmit());
          }
          // Refusal before dispatch must not masquerade as focus restoration.
          assert.equal(await previous.evaluate(element => element.isConnected), false, `${id}/${action} did not repaint`);
          await previous.dispose();
          // Observe native focus before screenshot/locator focus can repair it.
          const after = await state(); assert.equal(after.id, id); assert.equal(after.visible, true);
          assert.deepEqual(after.caret, before.caret); assert.deepEqual(after.local, before.local);
          assert.equal(await control.inputValue(), expected);
          return {id, action, actualRepaint: true, before, after};
        };
        const readonlyRepaints = [];
        const fixture = JSON.parse(await model());
        fixture.content = [{id: 'comment-resource', eClass: 'sysml:Comment', data: {elementId: 'comment0', declaredName: 'Ordinary comment', body, aliasIds: aliases}}];
        const raw = JSON.stringify(fixture); assert.ok(Buffer.byteLength(raw) < 1048576);
        await page.locator('form[data-action="import"]').locator('..').evaluate(element => {element.open = true;});
        await page.locator('#import-json').fill(raw);
        await page.locator('form[data-action="import"]').evaluate(form => form.requestSubmit());
        const imported = await model(); assert.deepEqual(JSON.parse(imported), fixture);
        await page.locator('button[data-value="comment0"]').click();
        assert.equal(await page.locator('#saved-value-body').evaluate(element => element.open), false);
        assert.equal(await page.locator('#saved-value-declaredName').count(), 0);
        // Geometry is captured before focus/click/screenshot can auto-scroll.
        const initial = await page.evaluate(() => {
          const state = document.querySelector('#scalar-state-body'), rect = state.getBoundingClientRect(), style = getComputedStyle(state);
          return {stateText: state.textContent, stateHeight: rect.height, lineHeight: parseFloat(style.lineHeight),
            bodyScrollHeight: document.body.scrollHeight, viewport: innerWidth, pageWidth: document.documentElement.scrollWidth,
            disclosureOpen: document.querySelector('#saved-value-body').open};
        });
        assert.match(initial.stateText, /Expand Full saved value/);
        assert.ok(initial.stateHeight <= 160 && initial.stateHeight <= 6 * initial.lineHeight + 1);
        assert.ok(initial.pageWidth <= width + 1); assert.equal(initial.disclosureOpen, false);
        await page.screenshot({path: path.join(output, `saved-initial-${width}.png`)});
        // declaredName is an advanced property. Open its real native disclosure
        // before checking the visible unchanged short-value presentation.
        const advanced = page.locator('#advanced-properties > summary');
        await advanced.focus(); await page.keyboard.press('Enter');
        assert.equal(await page.locator('#advanced-properties').evaluate(element => element.open), true);
        assert.equal(await page.locator('#scalar-state-declaredName').isVisible(), true);
        assert.equal(await page.locator('#scalar-state-declaredName').innerText(), 'Saved value: "Ordinary comment"');
        await advanced.focus(); await page.keyboard.press('Enter');
        assert.equal(await page.locator('#advanced-properties').evaluate(element => element.open), false);
        const summary = page.locator('#saved-value-body > summary');
        // Native keyboard activates details; timing starts in the actual click
        // event and ends after two animation frames, excluding focus/transport.
        const openSample = async () => {
          if (await page.locator('#saved-value-body').evaluate(element => element.open)) {
            await summary.focus(); await page.keyboard.press('Enter');
          }
          await summary.focus();
          await page.evaluate(() => {
            window.__savedTiming = new Promise(resolve => {
              document.querySelector('#saved-value-body > summary').addEventListener('click', () => {
                const start = performance.now();
                requestAnimationFrame(() => requestAnimationFrame(() => resolve(performance.now() - start)));
              }, {once: true});
            });
          });
          await page.keyboard.press('Enter');
          const elapsed = await page.evaluate(() => window.__savedTiming);
          assert.equal(await page.locator('#saved-value-body').evaluate(element => element.open), true);
          return elapsed;
        };
        const warmups = []; for (let i = 0; i < 2; i++) warmups.push(await openSample());
        const samples = []; for (let i = 0; i < 7; i++) samples.push(await openSample());
        const full = page.locator('#saved-json-body');
        assert.equal(await full.inputValue(), JSON.stringify(body));
        assert.equal(await full.evaluate(element => element.readOnly), true);
        const fullGeometry = await full.evaluate(element => {
          const rect = element.getBoundingClientRect(), style = getComputedStyle(element);
          return {height: rect.height, rows: element.rows, wrap: element.wrap, scrollWidth: element.scrollWidth,
            clientWidth: element.clientWidth, whiteSpace: style.whiteSpace, maxHeight: style.maxHeight,
            pageWidth: document.documentElement.scrollWidth, viewport: innerWidth};
        });
        assert.ok(fullGeometry.rows <= 12 && fullGeometry.height <= 200);
        assert.ok(fullGeometry.scrollWidth > fullGeometry.clientWidth);
        assert.equal(fullGeometry.wrap, 'off'); assert.equal(fullGeometry.whiteSpace, 'pre');
        assert.ok(fullGeometry.pageWidth <= width + 1);
        // Summary -> native readonly textbox via Tab, then select exact JSON.
        await summary.focus(); await page.keyboard.press('Tab');
        assert.equal(await page.evaluate(() => document.activeElement.id), 'saved-json-body');
        await page.keyboard.press('Control+A');
        const selected = await full.evaluate(element => [element.selectionStart, element.selectionEnd, element.value.length]);
        assert.deepEqual(selected, [0, JSON.stringify(body).length, JSON.stringify(body).length]);
        assert.equal(JSON.parse(await full.inputValue()), body);
        for (const action of ['font', 'error']) readonlyRepaints.push(await readonlyRepaint('saved-json-body', JSON.stringify(body), action));
        await page.screenshot({path: path.join(output, `saved-value-${width}.png`)});
        const session = await page.context().newCDPSession(page); await session.send('Accessibility.enable');
        const {root} = await session.send('DOM.getDocument', {depth: 0});
        const {nodeId} = await session.send('DOM.querySelector', {nodeId: root.nodeId, selector: '#saved-json-body'});
        const {nodes} = await session.send('Accessibility.getPartialAXTree', {nodeId, fetchRelatives: false});
        const ax = nodes.find(node => !node.ignored && node.role?.value === 'textbox');
        assert.ok(ax); assert.equal(ax.name?.value, 'Saved JSON value for Body');
        assert.ok(ax.properties.some(property => property.name === 'readonly' && property.value?.value === true));
        await session.detach();
        await page.locator('#unavailable-properties > summary').focus(); await page.keyboard.press('Enter');
        const aliasSummary = page.locator('#saved-value-aliasIds > summary');
        await aliasSummary.focus(); await page.keyboard.press('Enter');
        const alias = page.locator('#saved-json-aliasIds');
        assert.equal(await alias.inputValue(), JSON.stringify(aliases));
        assert.equal(await alias.evaluate(element => element.readOnly), true);
        assert.equal(await page.locator('form[data-attribute="aliasIds"]').count(), 0);
        await aliasSummary.focus(); await page.keyboard.press('Tab'); await page.keyboard.press('Control+A');
        assert.equal(await page.evaluate(() => document.activeElement.id), 'saved-json-aliasIds');
        assert.deepEqual(await alias.evaluate(element => [element.selectionStart, element.selectionEnd]), [0, JSON.stringify(aliases).length]);
        for (const action of ['font', 'error']) readonlyRepaints.push(await readonlyRepaint('saved-json-aliasIds', JSON.stringify(aliases), action));
        await page.screenshot({path: path.join(output, `saved-readonly-${width}.png`)});
        assert.equal(await model(), imported);
        await page.locator('#rename').fill('Renamed comment'); await page.locator('#stage-rename').click(); await page.locator('#commit-staged').click();
        const committed = await model(); assert.equal(JSON.parse(committed).content[0].data.body, body);
        assert.deepEqual(JSON.parse(committed).content[0].data.aliasIds, aliases);
        await page.locator('#model-undo').click(); assert.equal(await model(), imported);
        await page.locator('#model-redo').click(); assert.equal(await model(), committed);
        await page.locator('button[data-value="comment0"]').click();
        // Both large scalars still count their complete JSON and control text;
        // hiding saved JSON in details must not weaken helper admission.
        await page.locator('#scalar-declaredShortName').fill('&'.repeat(850000));
        await page.locator('form[data-action="attribute"][data-attribute="declaredShortName"]').evaluate(form => form.requestSubmit());
        await page.locator('#recovery-export').waitFor();
        const prefix = 'data:application/json;charset=utf-8,';
        const href = await page.locator('#recovery-export').getAttribute('href'); assert.ok(href.startsWith(prefix));
        assert.equal(decodeURIComponent(href.slice(prefix.length)), committed);
        const waiting = page.waitForEvent('download'); await page.locator('#recovery-export').focus(); await page.keyboard.press('Enter');
        const download = await waiting, downloaded = path.join(output, `saved-committed-${width}.json`);
        await download.saveAs(downloaded); assert.equal(await download.failure(), null);
        assert.equal(fs.readFileSync(downloaded, 'utf8'), committed);
        await page.locator('#recover-preview').focus(); await page.keyboard.press('Enter');
        assert.equal(await model(), committed);
        await page.locator('button[data-value="comment0"]').click();
        assert.equal(await page.locator('#saved-json-body').inputValue(), JSON.stringify(body));
        // A new document epoch with the same semantic id must not restore the
        // old readonly disclosure/selection or its old model-sized value.
        await page.locator('#saved-value-body > summary').focus(); await page.keyboard.press('Enter');
        await full.evaluate(element => {element.focus(); element.setSelectionRange(2, 17, 'backward');});
        const newEpoch = JSON.parse(committed); newEpoch.content[0].data.body = 'New epoch body '.repeat(30);
        delete newEpoch.content[0].data.aliasIds;
        await page.locator('form[data-action="import"]').locator('..').evaluate(element => {element.open = true;});
        await page.locator('#import-json').fill(JSON.stringify(newEpoch));
        await full.evaluate(element => {element.focus(); element.setSelectionRange(2, 17, 'backward');});
        await page.locator('form[data-action="import"]').evaluate(form => form.requestSubmit());
        assert.deepEqual(JSON.parse(await model()), newEpoch);
        await page.locator('button[data-value="comment0"]').click();
        assert.equal(await page.locator('#saved-value-body').evaluate(element => element.open), false);
        assert.equal(await full.inputValue(), JSON.stringify(newEpoch.content[0].data.body));
        assert.deepEqual(await full.evaluate(element => [element.selectionStart, element.selectionEnd]), [0, 0]);
        assert.equal(await page.locator('#saved-json-aliasIds').count(), 0);
        await page.evaluate(() => new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve))));
        const native = await page.evaluate(() => {window.__savedDrain?.(); return {longTasks: window.__savedTasks, supported: window.__savedLongTaskSupported};});
        assert.deepEqual(errors, []); assert.deepEqual(warnings, []);
        results.push({width, initial, fullGeometry, readonlyRepaints, newDocumentEpoch: true, accessibility: {role: ax.role.value, name: ax.name.value, readonly: true},
          valueBytes: Buffer.byteLength(JSON.stringify(body)), committedBytes: Buffer.byteLength(committed),
          disclosureTiming: {metric: 'native click event to two animation frames;2warmups+7samples', warmups, ...stats(samples)},
          longTasks: native.longTasks, longTaskSupported: native.supported, errors, warnings,
          scope: 'Bounded saved-value presentation, full native selection, exact readonly/writable data and actual committed download. Timings are workload observations, no universal50ms veto or native SysON parity claim.'});
      } catch (error) {
        try {await page.screenshot({path: path.join(output, `saved-failure-${width}.png`)});} catch {}
        throw error;
      } finally {await page.close();}
    }
    fs.writeFileSync(path.join(output, 'saved-values.json'), JSON.stringify({status: 'PASS_SELECTED_SAVED_VALUE_PRESENTATION', results}, null, 2));
  } finally {await browser.close();}
})().catch(error => {console.error(error); process.exitCode = 1;});
