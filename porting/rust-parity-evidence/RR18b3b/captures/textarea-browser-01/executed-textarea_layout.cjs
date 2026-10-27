// SPDX-License-Identifier: EPL-2.0
// Actual WASM/native DOM/AX controls; no replacement model, HTML or input runtime.
const assert=require('node:assert/strict'),fs=require('node:fs'),path=require('node:path');
const {chromium}=require(process.env.SYSON_PLAYWRIGHT_MODULE||'playwright');
const url=process.env.SYSON_PREVIEW_URL,output=process.env.SYSON_BROWSER_ARTIFACT_DIR;
assert.ok(url&&output&&path.resolve(output).startsWith('/workspace/'));
assert.ok(['127.0.0.1','localhost','[::1]'].includes(new URL(url).hostname));
fs.mkdirSync(output,{recursive:true});
const line='long-line-'+ 'x'.repeat(4096);
const literal='\n\n leading &<>"\'\t雪🧭\r\nmiddle\tspaces  \n'+line+'\ntrailing \n';
const displayed=literal.replace(/\r\n?/g,'\n');
(async()=>{
 const browser=await chromium.launch({headless:true}),cases=[];
 try {
  for(const width of [320,1440]) {
   const page=await browser.newPage({viewport:{width,height:1000},serviceWorkers:'block'}),errors=[],warnings=[];
   page.setDefaultTimeout(15000);page.on('pageerror',e=>errors.push(e.message));
   page.on('console',m=>{if(m.type()==='error')errors.push(m.text());if(m.type()==='warning')warnings.push(m.text());});
   await page.route('**/*',route=>{
    const requested=new URL(route.request().url());
    if(requested.origin===new URL(url).origin||requested.protocol==='data:')return route.continue();
    errors.push(`Unexpected external request: ${requested.href}`);return route.abort();
   });
   await page.addInitScript(()=>{
    window.__textareaTasks=[];
    if(PerformanceObserver.supportedEntryTypes.includes('longtask'))new PerformanceObserver(list=>window.__textareaTasks.push(...list.getEntries().map(e=>({start:e.startTime,duration:e.duration})))).observe({type:'longtask',buffered:true});
   });
   await page.goto(url);await page.getByRole('heading',{name:'SysON model editor',exact:true}).waitFor();
   const started=Date.now(),model=()=>page.locator('#export-json').inputValue();
   const overflow=()=>page.evaluate(()=>({width:document.documentElement.clientWidth,scroll:document.documentElement.scrollWidth}));
   const importer=()=>page.locator('form[data-action="import"]');
   const openImport=()=>importer().locator('..').evaluate(e=>{e.open=true;});
   const checkStyle=async selector=>{
    const style=await page.locator(selector).evaluate(e=>{const s=getComputedStyle(e);return {whiteSpace:s.whiteSpace,overflowWrap:s.overflowWrap,wordBreak:s.wordBreak,overflowX:s.overflowX,overflowY:s.overflowY};});
    assert.deepEqual(style,{whiteSpace:'pre',overflowWrap:'normal',wordBreak:'normal',overflowX:'auto',overflowY:'auto'});
   };
   const session=await page.context().newCDPSession(page);await session.send('Accessibility.enable');
   const ax=async(selector,name,readonly)=>{
    const document=await session.send('DOM.getDocument',{depth:0});
    const {nodeId}=await session.send('DOM.querySelector',{nodeId:document.root.nodeId,selector});assert.ok(nodeId);
    const {nodes}=await session.send('Accessibility.getPartialAXTree',{nodeId,fetchRelatives:false});
    const node=nodes.find(n=>!n.ignored&&n.role?.value==='textbox');assert.ok(node,JSON.stringify(nodes));
    assert.equal(node.name?.value,name);assert.equal(node.value?.value,await page.locator(selector).inputValue());
    const properties=Object.fromEntries(node.properties.map(p=>[p.name,p.value.value]));
    assert.equal(properties.multiline,true);assert.equal(properties.readonly,readonly);
    return {name,role:node.role.value,multiline:properties.multiline,readonly:properties.readonly};
   };
   await page.locator('#name').fill('Before');await page.locator('#stage-create').click();await page.locator('#commit-staged').click();
   await page.getByRole('button',{name:'Layout canvas',exact:true}).click();
   await page.waitForFunction(()=>!document.querySelector('[data-canvas-ticket]')&&document.getElementById('export-geometry').value);
   await page.locator('#geometry-details').evaluate(e=>{e.open=true;});await openImport();
   const accessibility=[];
   for(const [selector,name,readonly] of [
    ['#export-json','Committed document export',true],
    ['#export-geometry','Canvas geometry export (separate from model)',true],
    ['#import-json','EMF JSON document',false],
    ['#import-geometry','Canvas companion JSON for this exact document',false]
   ]) {await checkStyle(selector);accessibility.push(await ax(selector,name,readonly));}
   const document=JSON.parse(await model());document.content[0].data.declaredName=literal;
   await page.locator('#import-json').fill(JSON.stringify(document));await importer().evaluate(form=>form.requestSubmit());
   const original=await model();assert.equal(JSON.parse(original).content[0].data.declaredName,literal);
   await page.locator('.tree button').first().click();await checkStyle('#rename');
   assert.equal(await page.locator('#rename').inputValue(),displayed);
   accessibility.push(await ax('#rename','Element name',false));
   const rename=page.locator('#rename');await rename.focus();await page.keyboard.press('Control+Home');
   assert.equal(await rename.evaluate(e=>e.selectionStart),0);
   await page.keyboard.press('Control+End');assert.equal(await rename.evaluate(e=>e.selectionEnd),displayed.length);
   assert.ok(await rename.evaluate(e=>e.scrollTop>0));
   await page.keyboard.press('ArrowUp');await page.keyboard.press('ArrowUp');await page.keyboard.press('End');
   assert.ok(await rename.evaluate(e=>e.scrollLeft>0));
   await page.keyboard.press('Shift+Home');
   const begin=displayed.indexOf(line);
   assert.deepEqual(await rename.evaluate(e=>[e.selectionStart,e.selectionEnd,e.selectionDirection]),[begin,begin+line.length,'backward']);
   assert.equal(await rename.evaluate(e=>e.value.slice(e.selectionStart,e.selectionEnd)),line);
   assert.equal(await rename.inputValue(),displayed);
   const local=await rename.evaluate(e=>({client:e.clientWidth,scroll:e.scrollWidth,height:e.clientHeight,scrollHeight:e.scrollHeight}));
   assert.ok(local.scroll>local.client);assert.ok(local.scrollHeight>local.height);
   const ring=await rename.evaluate(e=>parseFloat(getComputedStyle(e).outlineWidth));assert.ok(ring>=2);
   await page.keyboard.press('Control+Home');await page.keyboard.press('ArrowDown');await page.keyboard.press('ArrowDown');
   await page.screenshot({path:path.join(output,`textarea-rename-${width}.png`)});
   await rename.screenshot({path:path.join(output,`textarea-rename-control-${width}.png`)});
   // Unchanged native LF-normalized textarea submission must retain original CRLF.
   await page.locator('#stage-rename').click();assert.equal(await model(),original);
   await page.locator('#commit-staged').click();assert.equal(await model(),original);
   const changed=displayed+'changed\tend';await page.locator('#rename').fill(changed);await page.locator('#stage-rename').click();
   assert.equal(await model(),original);await page.locator('#cancel-staged').click();assert.equal(await model(),original);
   await page.locator('#rename').fill(changed);await page.locator('#stage-rename').click();await page.locator('#commit-staged').click();
   const committed=await model();assert.equal(JSON.parse(committed).content[0].data.declaredName,changed);
   await page.locator('#model-undo').click();assert.equal(await model(),original);
   await page.locator('#model-redo').click();assert.equal(await model(),committed);
   assert.ok(await page.locator('#export-json').evaluate(e=>e.scrollWidth>e.clientWidth));
   const draft='\n\n{ "literal": "雪🧭 &<>\\t" }\n'+line+'\n';
   await page.locator('#rename').fill('unsaved\ninspector draft');
   await page.locator('#geometry-details').evaluate(e=>{e.open=true;});await page.locator('#import-geometry').fill(draft);
   await openImport();await page.locator('#import-json').fill(draft);
   await page.locator('#import-json').evaluate(e=>{e.focus();e.setSelectionRange(2,7,'backward');});
   await importer().evaluate(form=>form.requestSubmit());
   assert.match(await page.locator('#editor-status').getAttribute('class'),/error/);
   const preserved=async()=>{
    assert.equal(await model(),committed);assert.equal(await page.locator('#import-json').inputValue(),draft);
    assert.equal(await page.locator('#import-geometry').inputValue(),draft);
    assert.equal(await page.locator('#rename').inputValue(),'unsaved\ninspector draft');
    assert.equal(await page.evaluate(()=>document.activeElement.id),'import-json');
    assert.deepEqual(await page.locator('#import-json').evaluate(e=>[e.selectionStart,e.selectionEnd,e.selectionDirection]),[2,7,'backward']);
   };
   await preserved();await page.evaluate(()=>document.fonts.dispatchEvent(new Event('loadingdone')));await preserved();
   accessibility.push(await ax('#import-json','EMF JSON document',false));
   for(const selector of ['#import-json','#import-geometry']) {
    await checkStyle(selector);assert.ok(await page.locator(selector).evaluate(e=>e.scrollWidth>e.clientWidth));
   }
   const dimensions=await overflow();assert.ok(dimensions.scroll<=dimensions.width,JSON.stringify(dimensions));
   assert.deepEqual(errors,[]);assert.deepEqual(warnings,[]);
   await page.screenshot({path:path.join(output,`textarea-drafts-${width}.png`)});
   await page.locator('#import-json').screenshot({path:path.join(output,`textarea-draft-control-${width}.png`)});
   cases.push({width,elapsedMs:Date.now()-started,local,dimensions,accessibility,errors,warnings,longTasks:await page.evaluate(()=>window.__textareaTasks),
    glyphCoverage:'DOM Unicode preserved; adequate CJK glyph font not present in inspected baseline, visual gate OPEN',
    performanceAcceptance:'Observed native controls only; parent RR18b3 whole-task <50ms and full bounded rendering remain required'});
   await session.detach();await page.close();
  }
  assert.equal(cases.length,2);fs.writeFileSync(path.join(output,'textarea-layout.json'),JSON.stringify({status:'PASS_SELECTED_NATIVE_TEXTAREA_BEHAVIOR',cases},null,2));
 } finally {await browser.close();}
})().catch(e=>{console.error(e);process.exitCode=1;});
