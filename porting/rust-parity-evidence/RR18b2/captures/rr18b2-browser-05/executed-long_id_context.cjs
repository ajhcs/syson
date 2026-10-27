// SPDX-License-Identifier: EPL-2.0
// Real imported model identities and rendered controls; no model/HTML substitute.
const assert=require('node:assert/strict'),fs=require('node:fs'),path=require('node:path');
const {chromium}=require(process.env.SYSON_PLAYWRIGHT_MODULE||'playwright');
const url=process.env.SYSON_PREVIEW_URL,output=process.env.SYSON_BROWSER_ARTIFACT_DIR;
assert.ok(url&&output&&path.resolve(output).startsWith('/workspace/'));
assert.ok(['127.0.0.1','localhost','[::1]'].includes(new URL(url).hostname));fs.mkdirSync(output,{recursive:true});
(async()=>{
 const browser=await chromium.launch({headless:true}),cases=[];
 try {
  for(const width of [320,1440]) for(const length of [200,200000,600000]) {
   const page=await browser.newPage({viewport:{width,height:900},serviceWorkers:'block'}),errors=[];
   page.setDefaultTimeout(15000);page.on('pageerror',e=>errors.push(e.message));page.on('console',m=>{if(m.type()==='error')errors.push(m.text());});
   await page.route('**/*',route=>new URL(route.request().url()).origin===new URL(url).origin?route.continue():route.abort());
   await page.addInitScript(()=>{window.__longIdTasks=[];if(PerformanceObserver.supportedEntryTypes.includes('longtask'))new PerformanceObserver(list=>window.__longIdTasks.push(...list.getEntries().map(e=>({start:e.startTime,duration:e.duration})))).observe({type:'longtask',buffered:true});});
   await page.goto(url);await page.getByRole('heading',{name:'SysON model editor',exact:true}).waitFor();
   const started=Date.now(),model=()=>page.locator('#export-json').inputValue();
   const importer=()=>page.locator('form[data-action="import"]');
   const rawImport=async raw=>{await importer().locator('..').evaluate(e=>{e.open=true;});await page.locator('#import-json').fill(raw);await importer().evaluate(form=>form.requestSubmit());};
   await page.locator('#name').fill('LongID');await page.locator('#stage-create').click();await page.locator('#commit-staged').click();
   const document=JSON.parse(await model()),identity='owned-long-id-'+'a'.repeat(length);
   document.content[0].data.elementId=identity;await rawImport(JSON.stringify(document));
   await page.locator('.tree button').filter({hasText:/^LongIDPackage$/}).click();
   assert.equal(await page.locator('#rename').inputValue(),'LongID');
   const original=await model();await page.locator('#name').fill('document-scoped draft');
   await page.locator('#rename').fill('unsaved inspector draft');await rawImport('bad JSON');
   if(length===600000) assert.match(await page.locator('[data-platform-error]').innerText(),/Draft preservation limit exceeded/);
   else assert.match(await page.locator('#editor-status').getAttribute('class'),/error/);
   assert.equal(await page.locator('#rename').inputValue(),'unsaved inspector draft');
   assert.equal(await page.locator('#name').inputValue(),'document-scoped draft');assert.equal(await model(),original);
   await page.locator('#rename').fill('LongID');
   if(length===600000) {
    // An opened mutable disclosure needs exact inspector context and exceeds
    // quota together with the600k parent value; closing it is a native escape.
    await page.locator('#advanced-properties').evaluate(e=>{e.open=true;});await rawImport('bad JSON');
    assert.match(await page.locator('[data-platform-error]').innerText(),/Draft preservation limit exceeded/);
    await page.locator('#advanced-properties').evaluate(e=>{e.open=false;});await rawImport('bad JSON');
    assert.match(await page.locator('#editor-status').getAttribute('class'),/error/);
    assert.equal(await page.locator('#name').inputValue(),'document-scoped draft');assert.equal(await model(),original);
   } else {
    // Actual advanced attribute staging makes Rust's native disclosure default
    // OPEN. Explicitly closing it must survive a refused form/background repaint.
    await page.locator('#advanced-properties').evaluate(e=>{e.open=true;});
    await page.locator('#scalar-declaredName').fill('LongID');
    await page.locator('form[data-action="attribute"][data-attribute="declaredName"]').evaluate(form=>form.requestSubmit());
    await page.locator('#cancel-staged').click();assert.ok(await page.locator('#advanced-properties').evaluate(e=>e.open));
    await page.locator('#advanced-properties').evaluate(e=>{e.open=false;});
    await page.locator('#name').fill('document-scoped draft');await rawImport('bad JSON');
    assert.equal(await page.locator('#advanced-properties').evaluate(e=>e.open),false);
    await page.evaluate(()=>document.fonts.dispatchEvent(new Event('loadingdone')));
    assert.equal(await page.locator('#advanced-properties').evaluate(e=>e.open),false);
    assert.equal(await page.locator('#name').inputValue(),'document-scoped draft');
    // Restore the native default, eliminating an unnecessary inspector context.
    await page.locator('#advanced-properties').evaluate(e=>{e.open=true;});
   }
   const replacement=JSON.parse(original);replacement.content[0].data.declaredName='Replacement';
   if(length===600000) {
    await rawImport(JSON.stringify(replacement));
    assert.match(await page.locator('[data-platform-error]').innerText(),/Draft preservation limit exceeded/);
    assert.equal(await model(),original);
    await page.locator('#parent').selectOption(''); // Document root is mutable.
   }
   await rawImport(JSON.stringify(replacement));
   assert.equal(JSON.parse(await model()).content[0].data.declaredName,'Replacement');
   assert.equal(JSON.parse(await model()).content[0].data.elementId,identity);
   assert.equal(await page.locator('#name').inputValue(),'');assert.deepEqual(errors,[]);
   if(length===200) await page.screenshot({path:path.join(output,`long-id-${width}.png`),fullPage:true});
   cases.push({width,idCodeUnits:identity.length,modelReplaced:true,errors,elapsedMs:Date.now()-started,longTasks:await page.evaluate(()=>window.__longIdTasks),performanceAcceptance:'RR18b3 OPEN'});
   await page.close();
  }
  assert.equal(cases.length,6);fs.writeFileSync(path.join(output,'long-id-context.json'),JSON.stringify({status:'PASS_SELECTED_REAL_LONG_ID_CONTEXT',cases},null,2));
 } finally {await browser.close();}
})().catch(e=>{console.error(e);process.exitCode=1;});
