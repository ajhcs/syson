// SPDX-License-Identifier: EPL-2.0
// Validation only. All editing uses rendered controls; no model/ABI calls.
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { chromium } = require(process.env.SYSON_PLAYWRIGHT_MODULE || 'playwright');
const url = process.env.SYSON_PREVIEW_URL;
const output = process.env.SYSON_BROWSER_ARTIFACT_DIR;
if (!url || !output) throw Error('Explicit preview URL and owned artifact directory required');
fs.mkdirSync(output, {recursive:true});
(async () => {
 const browser = await chromium.launch({headless:true});
 const receipts = [];
 try {
  for (const width of [1365, 390]) {
   const page = await browser.newPage({viewport:{width,height:width===390?844:1000}, serviceWorkers:'block'});
   page.setDefaultTimeout(10000);
   const errors = [];
   page.on('pageerror', error => errors.push(error.message));
   page.on('console', message => { if (message.type()==='error') errors.push(message.text()); });
   await page.route('**/*', route => new URL(route.request().url()).origin===new URL(url).origin ? route.continue() : route.abort());
   await page.addInitScript(() => {
    window.historyLongTasks = [];
    if (PerformanceObserver.supportedEntryTypes.includes('longtask')) new PerformanceObserver(list => {
     for (const entry of list.getEntries()) window.historyLongTasks.push(entry.duration);
    }).observe({entryTypes:['longtask']});
   });
   await page.goto(url);
   await page.getByRole('heading',{name:'SysON model editor',exact:true}).waitFor();
   const exported = () => page.locator('#export-json').inputValue();
   const currentName = async () => JSON.parse(await exported()).content[0].data.declaredName;
   await page.locator('#kind').selectOption('package');
   await page.locator('#name').fill('Revision0');
   await page.locator('#stage-create').click();
   await page.locator('#commit-staged').click();
   await page.getByRole('button',{name:'Revision0 Package',exact:true}).click();
   const times = [];
   for (let n=1; n<=70; n++) {
    const start = performance.now();
    const before = await exported();
    await page.locator('#rename').fill(`Revision${n}`);
    await page.locator('#stage-rename').click();
    assert.equal(await exported(), before);
    await page.locator('#commit-staged').click();
    assert.equal(await currentName(), `Revision${n}`);
    await page.getByRole('button',{name:`Revision${n} Package`,exact:true}).waitFor();
    times.push(performance.now()-start);
   }
   const final = await exported();
   for (let n=69; n>=6; n--) {
    await page.locator('#model-undo').focus(); await page.keyboard.press('Enter');
    assert.equal(await currentName(), `Revision${n}`);
   }
   const boundary = await exported();
   await page.locator('#model-undo').focus(); await page.keyboard.press('Enter');
   assert.equal(await exported(), boundary);
   assert.match(await page.locator('#editor-status').innerText(), /No history available/);
   await page.screenshot({path:path.join(output,`history-${width}-boundary.png`),fullPage:true});
   for (let n=7; n<=70; n++) {
    await page.locator('#model-redo').focus(); await page.keyboard.press('Enter');
    assert.equal(await currentName(), `Revision${n}`);
   }
   assert.equal(await exported(), final);
   await page.locator('#model-redo').click(); assert.equal(await exported(), final);
   const dimensions = await page.evaluate(() => ({width:document.documentElement.clientWidth,scroll:document.documentElement.scrollWidth}));
   assert.ok(dimensions.scroll<=dimensions.width, JSON.stringify(dimensions));
   assert.deepEqual(errors, []);
   receipts.push({width, renames:70, undo:64, redo:64, boundaryName:'Revision6', errors, dimensions,
    observedCommitMilliseconds:times, observedLongTaskMilliseconds:await page.evaluate(() => window.historyLongTasks),
    claim:'Bounded rendered control exercise; timings are observations, not an SLA; visible truncation policy remains RR19 OPEN'});
   await page.close();
  }
  fs.writeFileSync(path.join(output,'history-ui.json'),JSON.stringify(receipts,null,2));
  console.log(JSON.stringify({result:'PASS_BOUNDED_HISTORY_UI',receipts}));
 } finally { await browser.close(); }
})().catch(error => { console.error(error); process.exitCode=1; });
