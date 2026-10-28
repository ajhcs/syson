// SPDX-License-Identifier: EPL-2.0
// Root-only actual default WASM caller; no replaced model/HTML/layout result.
const assert=require('node:assert/strict'),fs=require('node:fs'),path=require('node:path');
const {chromium}=require(process.env.SYSON_PLAYWRIGHT_MODULE||'playwright');
const url=process.env.SYSON_PREVIEW_URL,output=process.env.SYSON_BROWSER_ARTIFACT_DIR;
assert.ok(url&&output&&path.resolve(output).startsWith('/workspace/'));
const origin=new URL(url).origin;
assert.ok(['127.0.0.1','localhost','[::1]'].includes(new URL(url).hostname));
const fixture=fs.readFileSync(path.join(__dirname,'fixtures/ownership-amplification.json'),'utf8');
assert.equal(Buffer.byteLength(fixture),192234);
const original=JSON.parse(fixture),owner=original.content[0].data.elementId;
assert.equal(owner.length,65542);assert.equal(original.content[0].data.ownedRelationship.length,512);
fs.mkdirSync(output,{recursive:true});
(async()=>{
 const browser=await chromium.launch({headless:true}),results=[];
 try {
  for(const width of [320,1440]) {
   const page=await browser.newPage({viewport:{width,height:1000},serviceWorkers:'block'}),errors=[],warnings=[];
   page.setDefaultTimeout(30000);page.on('pageerror',e=>errors.push(e.message));
   page.on('console',m=>{if(m.type()==='error')errors.push(m.text());if(m.type()==='warning')warnings.push(m.text());});
   await page.route('**/*',route=>{if(new URL(route.request().url()).origin===origin)return route.continue();errors.push(`Unexpected external request: ${route.request().url()}`);return route.abort();});
   await page.addInitScript(()=>{window.__summaryTasks=[];if(PerformanceObserver.supportedEntryTypes.includes('longtask'))new PerformanceObserver(list=>window.__summaryTasks.push(...list.getEntries().map(e=>({start:e.startTime,duration:e.duration})))).observe({type:'longtask',buffered:true});});
   await page.goto(url);await page.getByRole('heading',{name:'SysON model editor',exact:true}).waitFor();
   const model=()=>page.locator('#export-json').inputValue();
   await page.locator('form[data-action="import"]').locator('..').evaluate(e=>{e.open=true;});
   await page.locator('#import-json').fill(fixture);
   await page.locator('form[data-action="import"]').evaluate(form=>form.requestSubmit());
   const before=await model();assert.deepEqual(JSON.parse(before),original);
   assert.equal(await page.locator('#recovery-export').count(),0);
   const hierarchy=page.locator('ul[aria-label="Model hierarchy"]');
   assert.equal(await hierarchy.locator('button[data-value^="child-"]').count(),512);
   assert.equal(await hierarchy.locator('button[data-value^="membership-"]').count(),0);
   assert.equal(await hierarchy.locator('ul.tree-children').count(),1);
   const relationships=page.locator('#model-relationships');
   assert.equal(await relationships.locator('button[data-value^="membership-"]').count(),512);
   await relationships.evaluate(e=>{e.open=true;});
   await page.locator('button[data-value="membership-0000"]').click();
   await page.getByRole('heading',{name:'OwningMembership',exact:true}).waitFor();assert.equal(await model(),before);
   await page.locator('button[data-value="child-0511"]').click();
   assert.equal(await page.locator('#rename').inputValue(),'part0511');
   assert.equal(await page.locator('#scalar-declaredName').count(),1);
   await page.locator('#rename').fill('Changed 雪');await page.locator('#stage-rename').click();
   assert.equal(await model(),before);
   await page.locator('#commit-staged').click();
   const expected=JSON.parse(before);expected.content[0].data.ownedRelationship[511].data.ownedRelatedElement[0].data.declaredName='Changed 雪';
   const changed=await model();assert.deepEqual(JSON.parse(changed),expected);
   await page.locator('#model-undo').click();assert.equal(await model(),before);
   await page.locator('#model-redo').click();assert.equal(await model(),changed);
   // Select the actual long-ID owner through its rendered control; do not
   // synthesize a different model or invoke an internal handler.
   await page.locator('button[data-action="select"]').filter({hasText:/^OwnerPackage$/}).click();
   assert.equal(await page.locator('#rename').inputValue(),'Owner');assert.equal(await model(),changed);
   await page.locator('#element-identity > summary').click();
   assert.equal(await page.locator('#element-identity').evaluate(element=>element.open),true);
   assert.equal(await page.locator('#element-identity .element-id').isVisible(),true);
   assert.equal(await page.locator('#element-identity .element-id').innerText(),owner);
   await page.locator('#name').fill('Unsubmitted creation draft');
   await page.locator('form[data-action="import"]').locator('..').evaluate(e=>{e.open=true;});
   await page.locator('#import-json').fill('invalid document');
   await page.locator('form[data-action="import"]').evaluate(form=>form.requestSubmit());
   assert.equal(await model(),changed);assert.equal(await page.locator('#name').inputValue(),'Unsubmitted creation draft');
   assert.match(await page.locator('#editor-status').getAttribute('class'),/error/);
   assert.deepEqual(errors,[]);assert.deepEqual(warnings,[]);
   await page.evaluate(()=>window.scrollTo(0,0));
   await page.screenshot({path:path.join(output,`borrowed-summary-${width}.png`)}); // Viewport only, not a50k-pixel page.
   results.push({width,fixtureBytes:Buffer.byteLength(fixture),ownerCodeUnits:owner.length,children:512,relationships:512,modelBytes:Buffer.byteLength(before),errors,warnings,longTasks:await page.evaluate(()=>window.__summaryTasks)});
   await page.close();
  }
  fs.writeFileSync(path.join(output,'borrowed-summaries.json'),JSON.stringify({status:'PASS_SELECTED_BORROWED_SUMMARY_CALLER',results,
   limits:'Borrowed summary/render-navigation foundation only. Complete HTML admission, all owned-summary consumers, physical mobile/cross-browser and required <50ms composed acceptance remain OPEN RR18b3c/d.'},null,2));
 } finally {await browser.close();}
})().catch(error=>{console.error(error);process.exitCode=1;});
