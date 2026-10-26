// SPDX-License-Identifier: EPL-2.0
// Actual browser regression for unrelated drafts during canvas and error paints.
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const {chromium} = require(process.env.SYSON_PLAYWRIGHT_MODULE || 'playwright');
(async () => {
  const url = process.env.SYSON_PREVIEW_URL;
  const output = process.env.SYSON_BROWSER_ARTIFACT_DIR;
  assert.ok(url && output);
  assert.ok(['127.0.0.1','localhost'].includes(new URL(url).hostname));
  const browser = await chromium.launch({headless:true});
  try {
    const page = await browser.newPage({viewport:{width:1440,height:1000},serviceWorkers:'block'});
    const errors = [];
    page.on('pageerror', error => errors.push(error.message));
    await page.route('**/*', route => new URL(route.request().url()).origin === new URL(url).origin
      ? route.continue() : route.abort());
    await page.goto(url);
    await page.locator('#name').fill('Vehicle');
    await page.locator('#stage-create').click();
    await page.locator('#commit-staged').click();
    await page.locator('.tree button').filter({hasText:/^VehiclePackage$/}).click();
    await page.locator('#rename').fill('Keep this rename');
    await page.locator('#rename').evaluate(e => { e.focus(); e.setSelectionRange(2,6); });
    await page.locator('#geometry-details').evaluate(e => { e.open = true; });
    const before = await page.locator('#export-json').inputValue();
    await page.getByRole('button',{name:'Layout canvas',exact:true}).click();
    await page.waitForFunction(() => !document.querySelector('[data-canvas-ticket]'));
    for (const action of ['Canvas undo','Canvas redo','Cancel canvas work']) {
      await page.getByRole('button',{name:action,exact:true}).click();
      assert.equal(await page.locator('#rename').inputValue(),'Keep this rename');
      assert.equal(await page.locator('#geometry-details').evaluate(e => e.open),true);
      assert.equal(await page.locator('#export-json').inputValue(),before);
    }
    await page.locator('#kind').selectOption('part_definition');
    await page.locator('#parent').selectOption('');
    await page.locator('#name').fill('Retry this name');
    await page.locator('#stage-create').click();
    await page.getByRole('alert').waitFor();
    assert.equal(await page.locator('#name').inputValue(),'Retry this name');
    assert.equal(await page.locator('#rename').inputValue(),'Keep this rename');
    assert.equal(await page.locator('#geometry-details').evaluate(e => e.open),true);
    assert.equal(await page.locator('#export-json').inputValue(),before);
    await page.screenshot({path:path.join(output,'draft-recovery.png')});
    assert.deepEqual(errors,[]);
    fs.writeFileSync(path.join(output,'result.json'),JSON.stringify({status:'PASS',checks:[
      'canvas undo/redo/cancel retain rename and disclosure',
      'rejected creation retains both independent form drafts',
      'committed document unchanged', 'no page errors'
    ]},null,2));
  } finally { await browser.close(); }
})().catch(error => { console.error(error); process.exitCode = 1; });
