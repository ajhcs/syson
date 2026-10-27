// SPDX-License-Identifier: EPL-2.0
// Actual default WASM UI; no model/ABI/HTML replacement or synthetic model engine.
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const {chromium} = require(process.env.SYSON_PLAYWRIGHT_MODULE || 'playwright');
const url = process.env.SYSON_PREVIEW_URL, output = process.env.SYSON_BROWSER_ARTIFACT_DIR;
assert.ok(url && output && path.resolve(output).startsWith('/workspace/'));
assert.ok(['127.0.0.1','localhost','[::1]'].includes(new URL(url).hostname));
const fixture = fs.readFileSync(path.join(__dirname,'../RR18a/fixtures/escaped-import.json'),'utf8');
assert.equal(Buffer.byteLength(fixture), 700258);
const hugeName = JSON.parse(fixture).content[0].data.declaredName;
fs.mkdirSync(output,{recursive:true});
(async () => {
 const browser = await chromium.launch({headless:true});
 const results = [];
 try {
  for (const width of [320,1440]) {
   const page = await browser.newPage({viewport:{width,height:1000},serviceWorkers:'block'});
   page.setDefaultTimeout(15000);
   const errors = [];
   page.on('pageerror',error=>errors.push(error.message));
   page.on('console',message=>{if(message.type()==='error') errors.push(message.text());});
   await page.route('**/*',route=>{
    const requested = new URL(route.request().url());
    if(requested.origin===new URL(url).origin || requested.protocol==='data:') return route.continue();
    errors.push(`Unexpected external request: ${requested.href}`); return route.abort();
   });
   await page.addInitScript(()=>{
    window.__draftLongTasks=[];
    if(PerformanceObserver.supportedEntryTypes.includes('longtask')) new PerformanceObserver(list=>{
     window.__draftLongTasks.push(...list.getEntries().map(entry=>({start:entry.startTime,duration:entry.duration})));
    }).observe({type:'longtask',buffered:true});
   });
   await page.goto(url);
   await page.getByRole('heading',{name:'SysON model editor',exact:true}).waitFor();
   await page.locator('#name').fill('Before');
   await page.locator('#stage-create').click(); await page.locator('#commit-staged').click();
   await page.locator('.tree button').filter({hasText:/^BeforePackage$/}).click();
   const model=()=>page.locator('#export-json').inputValue();
   const before=await model();
   await page.locator('#name').fill('Keep creation 雪 draft');
   await page.locator('form[data-action="import"]').locator('..').evaluate(element=>{element.open=true;});
   await page.locator('#import-json').fill('Keep import draft');
   await page.locator('#geometry-details').evaluate(element=>{element.open=true;});
   await page.locator('#import-geometry').fill('Keep geometry draft');
   await page.locator('#rename').fill(hugeName);
   await page.locator('#name').evaluate(element=>{element.focus();element.setSelectionRange(2,5,'backward');});
   const start=Date.now();
   // Native submit listener while an unrelated field retains focus/caret.
   await page.locator('form[data-action="rename"]').evaluate(form=>form.requestSubmit());
   await page.locator('#recovery-export').waitFor();
   const href=await page.locator('#recovery-export').getAttribute('href');
   assert.equal(decodeURIComponent(href.split(',').slice(1).join(',')),before);
   await page.locator('#recover-preview').focus(); await page.keyboard.press('Enter');
   assert.equal(await model(),before);
   assert.equal(await page.locator('#name').inputValue(),'Keep creation 雪 draft','unrelated creation draft must survive fallback and recover');
   assert.equal(await page.locator('#import-json').inputValue(),'Keep import draft');
   assert.equal(await page.locator('#import-geometry').inputValue(),'Keep geometry draft');
   assert.ok(await page.locator('#geometry-details').evaluate(element=>element.open));
   assert.ok(await page.locator('form[data-action="import"]').locator('..').evaluate(element=>element.open));
   assert.equal(await page.evaluate(()=>document.activeElement.id),'name');
   assert.deepEqual(await page.locator('#name').evaluate(element=>[element.selectionStart,element.selectionEnd,element.selectionDirection]),[2,5,'backward']);
   assert.equal(await page.locator('#rename').count(),0,'cancelled huge rename must not be restored');
   await page.locator('.tree button').filter({hasText:/^BeforePackage$/}).click();
   assert.equal(await page.locator('#rename').inputValue(),'Before');
   assert.equal(await page.locator('#recovery-export').count(),0,'recovery must not loop');
   assert.deepEqual(errors,[]);
   await page.screenshot({path:path.join(output,`draft-recovered-${width}.png`),fullPage:true});
   results.push({width,elapsedMs:Date.now()-start,errors,longTasks:await page.evaluate(()=>window.__draftLongTasks)});
   await page.close();
  }
  fs.writeFileSync(path.join(output,'draft-recovery.json'),JSON.stringify({status:'PASS_BOUNDED_DRAFT_RECOVERY',results,remaining:'Full HTML allocation and <50ms performance acceptance remain RR18b3 OPEN'},null,2));
 } finally {await browser.close();}
})().catch(error=>{console.error(error);process.exitCode=1;});
