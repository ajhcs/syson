// SPDX-License-Identifier: EPL-2.0
// Actual WASM controls plus explicitly labelled DOM/decoder fault guard cases.
const assert=require('node:assert/strict'), fs=require('node:fs'), path=require('node:path');
const {chromium}=require(process.env.SYSON_PLAYWRIGHT_MODULE||'playwright');
const url=process.env.SYSON_PREVIEW_URL,output=process.env.SYSON_BROWSER_ARTIFACT_DIR;
assert.ok(url&&output&&path.resolve(output).startsWith('/workspace/'));
assert.ok(['127.0.0.1','localhost','[::1]'].includes(new URL(url).hostname));
fs.mkdirSync(output,{recursive:true});
(async()=>{
 const browser=await chromium.launch({headless:true}); const cases=[];
 try {
  async function fresh() {
   const page=await browser.newPage({viewport:{width:390,height:844},serviceWorkers:'block'});
   page.setDefaultTimeout(15000); const errors=[];
   page.on('pageerror',e=>errors.push(e.message));page.on('console',m=>{if(m.type()==='error')errors.push(m.text());});
   await page.route('**/*',route=>new URL(route.request().url()).origin===new URL(url).origin?route.continue():route.abort());
   await page.goto(url);await page.getByRole('heading',{name:'SysON model editor',exact:true}).waitFor();
   page.observedErrors=errors; return page;
  }
  async function finish(page,name) {assert.deepEqual(page.observedErrors,[]);cases.push(name);await page.close();}
  const model=page=>page.locator('#export-json').inputValue();
  const background=page=>page.evaluate(()=>document.fonts.dispatchEvent(new Event('loadingdone')));
  const importer=page=>page.locator('form[data-action="import"]');
  const openImport=page=>importer(page).locator('..').evaluate(e=>{e.open=true;});
  const submitImport=page=>importer(page).evaluate(form=>form.requestSubmit());
  // Independent metadata oracle: three explicitly retained creation selections,
  // the dirty/focused import control, and two stable disclosures.
  const metadata=[['kind','stage/','select-one','package'],['parent','stage/','select-one',''],
   ['definition','stage/','select-one',''],['import-json','import/','textarea',''],
   ['form:import/'],['id:geometry-details']].flat().join('');
  const room=1048576-Buffer.byteLength(metadata);
  for(const symbol of ['a','雪','🧭',String.fromCharCode(0xd800)]) {
   const page=await fresh();await openImport(page);
   assert.equal(await page.locator('input[id]:not([readonly]),select[id],textarea[id]:not([readonly])').count(),6);
   const size=Buffer.byteLength(symbol);const exact=symbol.repeat(Math.floor(room/size))+'a'.repeat(room%size);
   await page.locator('#import-json').fill(exact);
   await page.locator('#import-json').evaluate(e=>{e.focus();e.setSelectionRange(1,4,'backward');});
   const before=await model(page);await background(page);
   assert.equal(await page.locator('#import-json').inputValue(),exact);
   assert.deepEqual(await page.locator('#import-json').evaluate(e=>[e.selectionStart,e.selectionEnd,e.selectionDirection]),[1,4,'backward']);
   await page.locator('#import-json').fill(exact+'a');
   await page.evaluate(()=>{window.__ownedDraftNode=document.getElementById('import-json');});
   await background(page);
   assert.match(await page.locator('[data-platform-error]').innerText(),/Draft preservation limit exceeded/);
   assert.ok(await page.evaluate(()=>window.__ownedDraftNode===document.getElementById('import-json')));
   assert.equal(await page.locator('#import-json').inputValue(),exact+'a');assert.equal(await model(page),before);
   await page.locator('#import-json').fill('short again');await background(page);
   assert.equal(await page.locator('#import-json').inputValue(),'short again');
   await finish(page,`utf8-exact-and-plus-one-symbol-${symbol.charCodeAt(0)}`);
  }
  {
   const page=await fresh();await openImport(page);const before=await model(page);
   await page.locator('#geometry-details').evaluate(e=>{e.open=true;});
   await page.locator('#name').fill('ordinary draft');await page.locator('#import-geometry').fill('geometry draft');
   await page.locator('#import-json').fill('not JSON');await submitImport(page);
   assert.equal(await page.locator('#name').inputValue(),'ordinary draft');
   assert.equal(await page.locator('#import-geometry').inputValue(),'geometry draft');assert.equal(await model(page),before);
   // Source document replacement reuses identical stable IDs on later imports.
   await page.locator('#name').fill('A');await page.locator('#stage-create').click();await page.locator('#commit-staged').click();
   const first=JSON.parse(await model(page)),replacement=JSON.parse(await model(page));replacement.content[0].data.declaredName='B';
   await openImport(page);await page.locator('#import-json').fill(JSON.stringify(replacement));
   await page.locator('#geometry-details').evaluate(e=>{e.open=true;});
   await page.locator('#name').fill('old document secret');await page.locator('#import-geometry').fill('old geometry');
   await submitImport(page);assert.equal(JSON.parse(await model(page)).content[0].data.declaredName,'B');
   assert.equal(await page.locator('#name').inputValue(),'');assert.equal(await page.locator('#import-geometry').inputValue(),'');
   await page.locator('.tree button').filter({hasText:/^BPackage$/}).click();
   await page.locator('#rename').fill('');
   await openImport(page);await page.locator('#import-json').fill('bad JSON again');await submitImport(page);
   assert.equal(await page.locator('#rename').inputValue(),'','a deliberately cleared field must remain dirty/preserved');
   await page.locator('#rename').fill('old selection draft');
   await openImport(page);await page.locator('#import-json').fill(JSON.stringify(first));await submitImport(page);
   await page.locator('.tree button').filter({hasText:/^APackage$/}).click();
   assert.equal(await page.locator('#rename').inputValue(),'A');
   await finish(page,'ordinary-refusal-and-identical-id-document-replacement');
  }
  {
   const page=await fresh();await page.locator('#name').fill('Old');await page.locator('#stage-create').click();await page.locator('#commit-staged').click();
   const replacement=JSON.parse(await model(page));replacement.content[0].data.declaredName='Replacement';
   await openImport(page);await page.locator('#import-json').fill(JSON.stringify(replacement));await page.locator('#name').fill('must not leak');
   // Owned one-shot failure after real Rust import dispatch, before DOM publish.
   // No substituted WASM/model/HTML; restore the native decoder before throwing.
   await page.evaluate(()=>{const decode=TextDecoder.prototype.decode;TextDecoder.prototype.decode=function(...args){TextDecoder.prototype.decode=decode;throw new Error('owned post-dispatch decoder fault');};});
   await submitImport(page);
   assert.match(await page.locator('[data-platform-error]').innerText(),/output became uncertain/);
   await background(page);
   assert.equal(JSON.parse(await model(page)).content[0].data.declaredName,'Replacement');
   assert.equal(await page.locator('#name').inputValue(),'');assert.equal(await page.locator('#import-json').inputValue(),'');
   await finish(page,'actual-import-post-dispatch-decoder-fault-clears-untrusted-cache');
  }
  {
   const page=await fresh();
   await page.evaluate(()=>{window.__beforePristine=document.getElementById('export-json');for(let i=0;i<100;i++){const element=document.createElement('input');element.id=`owned-pristine-${i}`;document.getElementById('app').append(element);}});
   await background(page);assert.equal(await page.locator('[data-platform-error]').count(),0);
   assert.equal(await page.locator('#export-json').count(),1);
   assert.ok(await page.evaluate(()=>window.__beforePristine!==document.getElementById('export-json')));
   await finish(page,'owned-dom-more-than64-pristine-controls-do-not-deadlock');
  }
  // Explicit owned DOM guard probes, not normal product rendering evidence.
  // The exact-count input is deliberately rejected later by the independent ABI
  // byte limit. This proves cache admission without deleting fixture controls.
  for(const disclosures of [false,true]) {
   const page=await fresh();await openImport(page);await page.locator('#import-json').fill('\n'.repeat(600000));
   await page.evaluate(disclosures=>{
    const count=disclosures?30:60;
    for(let i=0;i<count;i++){const element=document.createElement(disclosures?'details':'input');element.id=`owned-boundary-${i}`;if(!disclosures)element.value='dirty';document.getElementById('app').append(element);}
   },disclosures);
   await submitImport(page);assert.match(await page.locator('[data-platform-error]').innerText(),/Input exceeds/);
   await page.evaluate(disclosures=>{const element=document.createElement(disclosures?'details':'input');element.id='owned-plus-one';if(!disclosures)element.value='dirty';document.getElementById('app').append(element);},disclosures);
   await submitImport(page);assert.match(await page.locator('[data-platform-error]').innerText(),/Draft preservation limit exceeded/);
   assert.equal(await page.locator('#import-json').inputValue(),'\n'.repeat(600000));
   await finish(page,disclosures?'owned-dom-32-and-33-disclosure-guard':'owned-dom-64-and-65-field-guard');
  }
  for(const probe of ['duplicate-field','duplicate-disclosure','metadata']) {
   const page=await fresh();await openImport(page);await page.locator('#import-json').fill('\n'.repeat(600000));
   await page.evaluate(probe=>{
    const element=document.createElement(probe==='duplicate-disclosure'?'details':'input');
    element.id=probe==='duplicate-field'?'name':probe==='duplicate-disclosure'?'geometry-details':'x'.repeat(128);
    if(probe!=='duplicate-disclosure')element.value='dirty';document.getElementById('app').append(element);
   },probe);
   await submitImport(page);
   assert.match(await page.locator('[data-platform-error]').innerText(),probe==='metadata'?/Input exceeds/:/Draft preservation limit exceeded/);
   if(probe==='metadata') {
    await page.evaluate(()=>{document.getElementById('x'.repeat(128)).id='x'.repeat(129);});
    await submitImport(page);assert.match(await page.locator('[data-platform-error]').innerText(),/Draft preservation limit exceeded/);
   }
   assert.equal(await page.locator('#import-json').inputValue(),'\n'.repeat(600000));
   await finish(page,`owned-dom-${probe}-guard`);
  }
  fs.writeFileSync(path.join(output,'draft-limits.json'),JSON.stringify({status:'PASS_BOUNDED_DRAFT_GUARDS',cases,limits:'Loader retention only; normal renderer allocation/performance remains RR18b3 OPEN'},null,2));
 } finally {await browser.close();}
})().catch(e=>{console.error(e);process.exitCode=1;});
