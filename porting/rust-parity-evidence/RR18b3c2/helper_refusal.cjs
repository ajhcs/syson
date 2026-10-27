// SPDX-License-Identifier: EPL-2.0
// Actual native forms/WASM/download recovery. Observation delegates unchanged.
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const {chromium} = require(process.env.SYSON_PLAYWRIGHT_MODULE || 'playwright');
const url = process.env.SYSON_PREVIEW_URL;
const output = process.env.SYSON_BROWSER_ARTIFACT_DIR;
assert.ok(url && output && path.resolve(output).startsWith('/workspace/'));
assert.ok(['127.0.0.1', 'localhost', '[::1]'].includes(new URL(url).hostname));
fs.mkdirSync(output, {recursive: true});
const amp = '&'.repeat(850000);
const body = '\n\nReader body 雪🧭\r\nsecond\tline\n' + amp;
const displayedBody = body.replace(/\r\n?/g, '\n');
const inputLimit = 1048576;
const modelLimit = 4 * inputLimit;
const inspectorLimit = 16 * inputLimit;
// Two scalar fields each emit stored JSON string AND textarea text. Each raw
// ampersand needs five bytes in both positions, even before any HTML wrappers.
const inspectorLowerBound = 2 * (amp.length * 5 + amp.length * 5);
assert.ok(inspectorLowerBound > inspectorLimit);
(async () => {
  const browser = await chromium.launch({headless: true});
  const results = [];
  try {
    for (const width of [320, 1440]) {
      const page = await browser.newPage({viewport: {width, height: 1000}, serviceWorkers: 'block', acceptDownloads: true});
      page.setDefaultTimeout(30000);
      const errors = [], warnings = [], phases = [];
      page.on('pageerror', error => errors.push(error.message));
      page.on('console', message => {
        if (message.type() === 'error') errors.push(message.text());
        if (message.type() === 'warning') warnings.push(message.text());
      });
      await page.route('**/*', route => {
        const requested = new URL(route.request().url());
        if (requested.origin === new URL(url).origin || requested.protocol === 'data:') return route.continue();
        errors.push(`Unexpected external request: ${requested.href}`); return route.abort();
      });
      await page.addInitScript(() => {
        window.__helperRequests = []; window.__helperRequestOverflow = false; window.__helperTasks = [];
        const encode = TextEncoder.prototype.encode;
        TextEncoder.prototype.encode = function (text) {
          const bytes = encode.call(this, text);
          const action = typeof text === 'string' && /^\{"action":"([^"\n]{1,64})"/.exec(text.slice(0, 128));
          if (action) {
            if (window.__helperRequests.length === 64) window.__helperRequestOverflow = true;
            else window.__helperRequests.push({action: action[1], bytes: bytes.length});
          }
          return bytes;
        };
        const observe = entries => window.__helperTasks.push(...entries.map(entry => ({start: entry.startTime, duration: entry.duration})));
        window.__helperLongTaskSupported = PerformanceObserver.supportedEntryTypes.includes('longtask');
        if (window.__helperLongTaskSupported) {
          const observer = new PerformanceObserver(list => observe(list.getEntries()));
          observer.observe({type: 'longtask', buffered: true});
          window.__drainHelperTasks = () => observe(observer.takeRecords());
        }
      });
      const phase = async (name, operation) => {
        const start = await page.evaluate(() => performance.now());
        const value = await operation();
        const end = await page.evaluate(() => performance.now());
        phases.push({name, start, end, elapsedMs: end - start}); return value;
      };
      const focused = () => page.evaluate(() => {
        const element = document.activeElement, rect = element.getBoundingClientRect();
        const style = getComputedStyle(element);
        return {id: element.id, tag: element.tagName, top: rect.top, bottom: rect.bottom, left: rect.left, right: rect.right,
          intersectsViewport: rect.bottom > 0 && rect.top < innerHeight && rect.right > 0 && rect.left < innerWidth,
          displayed: style.display !== 'none' && style.visibility !== 'hidden'};
      });
      try {
      await page.goto(url);
      await page.getByRole('heading', {name: 'SysON model editor', exact: true}).waitFor();
      const model = () => page.locator('#export-json').inputValue();
      const importer = () => page.locator('form[data-action="import"]');
      const fixture = JSON.parse(await model());
      fixture.content = [{id: 'comment-resource', eClass: 'sysml:Comment', data: {elementId: 'comment0', declaredName: 'Helper comment', body}}];
      const raw = JSON.stringify(fixture); assert.ok(Buffer.byteLength(raw) < inputLimit);
      await phase('import committed large Comment body', async () => {
        await importer().locator('..').evaluate(element => {element.open = true;});
        await page.locator('#import-json').fill(raw); await importer().evaluate(form => form.requestSubmit());
      });
      const imported = await model(); assert.deepEqual(JSON.parse(imported), fixture);
      await page.locator('button[data-value="comment0"]').click();
      const selectedFocus = await focused();
      assert.equal(await page.locator('#scalar-body').inputValue(), displayedBody);
      // Reach the actual control with native keyboard navigation, recording the
      // rectangle before screenshot APIs can scroll anything into view.
      let bodyTabs = 0;
      while (await page.evaluate(() => document.activeElement.id) !== 'scalar-body' && bodyTabs < 24) {
        await page.keyboard.press('Tab'); bodyTabs++;
      }
      await page.keyboard.press('Control+Home');
      const scalarFocus = await focused(); assert.equal(scalarFocus.id, 'scalar-body');
      assert.equal(scalarFocus.displayed, true); assert.equal(scalarFocus.intersectsViewport, true);
      await page.screenshot({path: path.join(output, `helper-scalar-${width}.png`)});
      await phase('create real committed undo history', async () => {
        await page.locator('#rename').fill('Committed comment'); await page.locator('#stage-rename').click();
        await page.locator('#commit-staged').click();
      });
      const committed = await model(); assert.equal(JSON.parse(committed).content[0].data.body, body);
      const pendingOracle = JSON.parse(committed); pendingOracle.content[0].data.declaredShortName = amp;
      const expectedPendingPrettyBytes = Buffer.byteLength(JSON.stringify(pendingOracle, null, 2));
      assert.ok(expectedPendingPrettyBytes < modelLimit);
      const stageShort = async () => {
        await page.locator('#scalar-mode-declaredShortName').selectOption('value');
        await page.locator('#scalar-declaredShortName').fill(amp);
        await page.locator('form[data-action="attribute"][data-attribute="declaredShortName"]').evaluate(form => form.requestSubmit());
        await page.locator('#recovery-export').waitFor();
      };
      await page.locator('#name').fill('Keep unrelated creation draft');
      await importer().locator('..').evaluate(element => {element.open = true;});
      await page.locator('#import-json').fill('Keep unrelated import draft');
      await page.locator('#geometry-details').evaluate(element => {element.open = true;});
      await page.locator('#import-geometry').fill('Keep unrelated geometry draft');
      await page.locator('#scalar-mode-declaredShortName').selectOption('value');
      await page.locator('#scalar-declaredShortName').fill(amp);
      await page.locator('#name').evaluate(element => {element.focus(); element.setSelectionRange(2, 8, 'backward');});
      await phase('helper overlimit scalar form', async () => {
        await page.locator('form[data-action="attribute"][data-attribute="declaredShortName"]').evaluate(form => form.requestSubmit());
        await page.locator('#recovery-export').waitFor();
      });
      // Record true focus BEFORE a locator.focus/click or screenshot can scroll.
      const refusalFocus = await focused(); assert.equal(refusalFocus.id, 'recover-preview');
      const href = await page.locator('#recovery-export').getAttribute('href');
      const prefix = 'data:application/json;charset=utf-8,';
      assert.ok(href.startsWith(prefix)); assert.equal(href.length - prefix.length, Buffer.byteLength(committed) * 3);
      assert.equal(decodeURIComponent(href.slice(prefix.length)), committed);
      const session = await page.context().newCDPSession(page); await session.send('Accessibility.enable');
      const accessibility = [];
      for (const [selector, role, name] of [['#recover-preview', 'button', 'Recover committed view (discard staged change)'], ['#recovery-export', 'link', 'Download committed model JSON']]) {
        const {root} = await session.send('DOM.getDocument', {depth: 0});
        const {nodeId} = await session.send('DOM.querySelector', {nodeId: root.nodeId, selector});
        const {nodes} = await session.send('Accessibility.getPartialAXTree', {nodeId, fetchRelatives: false});
        const node = nodes.find(item => !item.ignored && item.role?.value === role);
        assert.ok(node); assert.equal(node.name?.value, name); accessibility.push({role, name});
      }
      await page.screenshot({path: path.join(output, `helper-refusal-${width}.png`)});
      await phase('native committed model download', async () => {
        const waiting = page.waitForEvent('download'); await page.locator('#recovery-export').focus(); await page.keyboard.press('Enter');
        const download = await waiting, file = path.join(output, `helper-committed-${width}.json`);
        await download.saveAs(file); assert.equal(await download.failure(), null);
        assert.equal(fs.readFileSync(file, 'utf8'), committed);
      });
      await page.locator('#recover-preview').focus();
      await phase('keyboard recover discards only pending scalar', () => page.keyboard.press('Enter'));
      const recoveredFocus = await focused(); assert.equal(recoveredFocus.id, 'name');
      assert.equal(await model(), committed); assert.equal(await page.locator('#name').inputValue(), 'Keep unrelated creation draft');
      assert.equal(await page.locator('#import-json').inputValue(), 'Keep unrelated import draft');
      assert.equal(await page.locator('#import-geometry').inputValue(), 'Keep unrelated geometry draft');
      assert.deepEqual(await page.locator('#name').evaluate(element => [element.selectionStart, element.selectionEnd, element.selectionDirection]), [2, 8, 'backward']);
      await page.locator('button[data-value="comment0"]').click();
      assert.equal(await page.locator('#scalar-declaredShortName').inputValue(), '');
      assert.equal(await page.locator('#scalar-body').inputValue(), displayedBody); assert.equal(await page.locator('#recovery-export').count(), 0);
      await phase('committed undo and redo survived helper refusal', async () => {
        await page.locator('#model-undo').focus(); await page.keyboard.press('Enter'); assert.equal(await model(), imported);
        await page.locator('#model-redo').focus(); await page.keyboard.press('Enter'); assert.equal(await model(), committed);
      });
      await page.locator('button[data-value="comment0"]').click();
      await phase('repeat helper refusal and keyboard confirmed discard', async () => {
        await stageShort();
        await page.getByRole('button', {name: 'Discard current document', exact: true}).focus(); await page.keyboard.press('Enter');
      });
      const confirmFocus = await focused(); assert.equal(confirmFocus.id, 'confirm-discard');
      await page.evaluate(() => document.fonts.dispatchEvent(new Event('loadingdone')));
      await page.keyboard.press('Enter');
      const empty = await model(); assert.equal(JSON.parse(empty).content.length, 0);
      await page.locator('#model-undo').focus(); await page.keyboard.press('Enter'); assert.equal(await model(), empty);
      assert.equal(await page.locator('#name').inputValue(), ''); assert.equal(await page.locator('#import-json').inputValue(), '');
      // Deliver native observations after the last action, then drain any tail.
      await page.evaluate(() => new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(() => setTimeout(resolve, 0)))));
      const observed = await page.evaluate(() => {window.__drainHelperTasks?.(); return {requests: window.__helperRequests, overflow: window.__helperRequestOverflow, tasks: window.__helperTasks, longTaskSupported: window.__helperLongTaskSupported};});
      assert.equal(observed.overflow, false); assert.ok(observed.requests.length > 0);
      for (const request of observed.requests) assert.ok(request.bytes < inputLimit, JSON.stringify(request));
      assert.deepEqual(errors, []); assert.deepEqual(warnings, []);
      results.push({width, expectedPendingPrettyBytes, inspectorLowerBound, committedBytes: Buffer.byteLength(committed), accessibility,
        selectedFocus, scalarFocus, bodyTabs, refusalFocus, recoveredFocus, confirmFocus, phases, requests: observed.requests, longTasks: observed.tasks, longTaskSupported: observed.longTaskSupported, errors, warnings,
        scope: 'Expected pending model is a fixture oracle; helper branch inferred from immutable producer and17M inspector lower bound. Whole rendering, viewport focus/glyph and<50ms acceptance remain OPEN.'});
      await session.detach(); await page.close();
      } catch (error) {
        // Failure-only native diagnostics; never replace the original assertion.
        try {
          const diagnostic = await page.evaluate(() => {
            const summary = text => {
              if (typeof text !== 'string') return null;
              let cr = 0, lf = 0;
              for (let i = 0; i < text.length; i++) { if (text.charCodeAt(i) === 13) cr++; if (text.charCodeAt(i) === 10) lf++; }
              return {utf16Units: text.length, carriageReturns: cr, lineFeeds: lf};
            };
            const fields = ['scalar-body', 'scalar-declaredShortName', 'name', 'import-json', 'import-geometry'].map(id => {
              const element = document.getElementById(id);
              return element ? {id, value: summary(element.value), defaultValue: summary(element.defaultValue),
                differsFromDefault: element.value !== element.defaultValue,
                matchesNormalizedDefault: typeof element.defaultValue === 'string' && element.value === element.defaultValue.replace(/\r\n?/g, '\n')} : {id, absent: true};
            });
            const status = document.querySelector('#editor-status')?.textContent ?? '';
            const platformStatus = document.querySelector('[data-platform-error]')?.textContent ?? '';
            return {fields, status: status.slice(0, 4096), statusTruncated: status.length > 4096,
              platformStatus: platformStatus.slice(0, 4096), platformStatusTruncated: platformStatus.length > 4096,
              requests: window.__helperRequests ?? [], requestOverflow: window.__helperRequestOverflow};
          });
          diagnostic.focus = await focused(); diagnostic.width = width;
          diagnostic.originalFailure = String(error).slice(0, 8192); diagnostic.phases = phases;
          diagnostic.errors = errors; diagnostic.warnings = warnings;
          diagnostic.requestCounts = diagnostic.requests.reduce((counts, request) => { counts[request.action] = (counts[request.action] ?? 0) + 1; return counts; }, {});
          try { await page.screenshot({path: path.join(output, `helper-failure-${width}.png`)}); }
          catch (screenshotError) { diagnostic.screenshotFailure = String(screenshotError).slice(0, 4096); }
          fs.writeFileSync(path.join(output, `helper-failure-${width}.json`), JSON.stringify(diagnostic, null, 2));
        } catch (diagnosticError) { console.error('Failure diagnostics unavailable:', diagnosticError); }
        throw error;
      }
    }
    assert.equal(results.length, 2);
    fs.writeFileSync(path.join(output, 'helper-refusal.json'), JSON.stringify({status: 'PASS_SELECTED_ACTUAL_HELPER_REFUSAL_CONSUMER', results}, null, 2));
  } finally {await browser.close();}
})().catch(error => {console.error(error); process.exitCode = 1;});
