// SPDX-License-Identifier: EPL-2.0
// Real native defaults/forms and unchanged WASM; no substituted model or HTML.
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const {chromium} = require(process.env.SYSON_PLAYWRIGHT_MODULE || 'playwright');
const url = process.env.SYSON_PREVIEW_URL, output = process.env.SYSON_BROWSER_ARTIFACT_DIR;
assert.ok(url && output && path.resolve(output).startsWith('/workspace/'));
assert.ok(['127.0.0.1', 'localhost', '[::1]'].includes(new URL(url).hostname));
fs.mkdirSync(output, {recursive: true});
const amp = '&'.repeat(530000);
const cases = [
  ['lf', '\n\nfirst\n', '\nlast\n'],
  ['crlf', '\r\nfirst\r\n', '\r\nlast\r\n'],
  ['lone-cr', '\rfirst\r', '\rlast\r'],
  ['mixed', '\n\r\nfirst\r', '\r\nlast\n\r'],
  ['trailing-cr', 'first', '\r'],
  ['unicode', '\n雪🧭\r\n', '\tEnd雪🧭'],
];
(async () => {
  const browser = await chromium.launch({headless: true}), results = [];
  try {
    for (const width of [320, 1440]) for (const [name, prefix, suffix] of cases) {
      const page = await browser.newPage({viewport: {width, height: 1000}, serviceWorkers: 'block'});
      page.setDefaultTimeout(30000);
      const errors = [], warnings = [];
      page.on('pageerror', error => errors.push(error.message));
      page.on('console', message => {
        if (message.type() === 'error') errors.push(message.text());
        if (message.type() === 'warning') warnings.push(message.text());
      });
      await page.route('**/*', route => {
        if (new URL(route.request().url()).origin === new URL(url).origin) return route.continue();
        errors.push(`Unexpected external request: ${route.request().url()}`); return route.abort();
      });
      await page.addInitScript(() => {
        window.__draftActions = []; window.__draftActionOverflow = false;
        const encode = TextEncoder.prototype.encode;
        TextEncoder.prototype.encode = function (text) {
          const bytes = encode.call(this, text);
          const action = typeof text === 'string' && /^\{"action":"([^"\n]{1,64})"/.exec(text.slice(0, 128));
          if (action) {
            if (window.__draftActions.length === 128) window.__draftActionOverflow = true;
            else window.__draftActions.push({action: action[1], bytes: bytes.length});
          }
          return bytes;
        };
      });
      await page.goto(url);
      await page.getByRole('heading', {name: 'SysON model editor', exact: true}).waitFor();
      const model = () => page.locator('#export-json').inputValue();
      const importer = () => page.locator('form[data-action="import"]');
      const importText = async text => {
        await importer().locator('..').evaluate(element => {element.open = true;});
        await page.locator('#import-json').fill(text);
        await importer().evaluate(form => form.requestSubmit());
      };
      const body = prefix + amp + suffix, displayed = body.replace(/\r\n?/g, '\n');
      const fixture = JSON.parse(await model());
      fixture.content = [{id: 'comment-resource', eClass: 'sysml:Comment', data: {elementId: 'comment0', declaredName: 'Note', body}}];
      const raw = JSON.stringify(fixture); assert.ok(Buffer.byteLength(raw) < 1048576);
      await importText(raw);
      const committed = await model(); assert.deepEqual(JSON.parse(committed), fixture);
      await page.locator('button[data-value="comment0"]').click();
      const field = page.locator('#scalar-body');
      assert.equal(await field.inputValue(), displayed);
      assert.equal(await field.evaluate(element => element.defaultValue), body);
      const short = page.locator('#scalar-declaredShortName');
      const shortForm = () => page.locator('form[data-action="attribute"][data-attribute="declaredShortName"]');
      const requests = () => page.evaluate(() => window.__draftActions.filter(entry => entry.action === 'attribute').length);
      await short.fill(amp);
      const before = await requests();
      await shortForm().evaluate(form => form.requestSubmit());
      assert.equal(await requests(), before + 1, 'pristine normalized body must not consume draft quota');
      await page.locator('#commit-staged').waitFor();
      assert.equal(await page.locator('#recovery-export').count(), 0);
      assert.equal(await model(), committed);
      await page.locator('#cancel-staged').click();
      assert.equal(await model(), committed);
      // A focused pristine field is deliberately retained by the original rule.
      await short.fill(amp);
      await field.evaluate(element => {element.focus(); element.setSelectionRange(2, 8, 'backward');});
      let blocked = await requests();
      await shortForm().evaluate(form => form.requestSubmit());
      assert.equal(await requests(), blocked);
      assert.match(await page.locator('[data-platform-error]').innerText(), /Draft preservation limit exceeded/);
      assert.equal(await field.inputValue(), displayed); assert.equal(await short.inputValue(), amp);
      assert.equal(await page.locator('#commit-staged').count(), 0);
      // Two genuinely dirty large values still exceed the unchanged1MiB quota.
      await field.fill(displayed + 'edited'); await short.fill(amp);
      blocked = await requests();
      await shortForm().evaluate(form => form.requestSubmit());
      assert.equal(await requests(), blocked);
      assert.match(await page.locator('[data-platform-error]').innerText(), /Draft preservation limit exceeded/);
      assert.equal(await field.inputValue(), displayed + 'edited'); assert.equal(await short.inputValue(), amp);
      await short.fill('');
      await importText('bad document');
      assert.equal(await field.inputValue(), displayed + 'edited', 'true edit must survive refusal');
      assert.equal(await model(), committed);
      await field.fill('');
      await importText('bad document');
      assert.equal(await field.inputValue(), '', 'deliberately cleared value must survive refusal');
      assert.equal(await model(), committed);
      await field.fill(displayed);
      await field.evaluate(element => {element.focus(); element.setSelectionRange(2, 8, 'backward');});
      await page.evaluate(() => document.fonts.dispatchEvent(new Event('loadingdone')));
      assert.equal(await field.inputValue(), displayed);
      assert.equal(await page.evaluate(() => document.activeElement.id), 'scalar-body');
      assert.deepEqual(await field.evaluate(element => [element.selectionStart, element.selectionEnd, element.selectionDirection]), [2, 8, 'backward']);
      assert.equal(await model(), committed);
      const observed = await page.evaluate(() => ({actions: window.__draftActions, overflow: window.__draftActionOverflow}));
      assert.equal(observed.overflow, false);
      for (const request of observed.actions) assert.ok(request.bytes < 1048576);
      assert.deepEqual(errors, []); assert.deepEqual(warnings, []);
      if (name === 'crlf') await page.screenshot({path: path.join(output, `draft-normalization-${width}.png`)});
      results.push({width, name, bodyCodeUnits: body.length, nativeValueCodeUnits: displayed.length, errors, warnings, actions: observed.actions});
      await page.close();
    }
    assert.equal(results.length, 12);
    fs.writeFileSync(path.join(output, 'draft-normalization.json'), JSON.stringify({status: 'PASS_NATIVE_TEXTAREA_DEFAULT_COMPARISON', results,
      limits: 'Textarea draft classification only; whole HTML/helper performance, visible focus, glyphs and<50ms acceptance remain separate. Original850k helper-refusal caller is independently required.'}, null, 2));
  } finally {await browser.close();}
})().catch(error => {console.error(error); process.exitCode = 1;});
