// SPDX-License-Identifier: EPL-2.0
// Default-feature native scalar controls; no model/HTML/WASM substitution.
const assert=require('node:assert/strict'),fs=require('node:fs'),path=require('node:path');
const {chromium}=require(process.env.SYSON_PLAYWRIGHT_MODULE||'playwright');
const url=process.env.SYSON_PREVIEW_URL,out=process.env.SYSON_BROWSER_ARTIFACT_DIR;
assert.ok(url&&out&&path.resolve(out).startsWith('/workspace/'));
assert.ok(['127.0.0.1','localhost','[::1]'].includes(new URL(url).hostname));
const literal='\n\nfirst\r\nsecond\r&<>"\'\t\0雪🧭'+'&'.repeat(16384);
const displayed=literal.replace(/\r\n?/g,'\n').replace(/\0/g,'\ufffd');
const expanding=fs.readFileSync(path.join(__dirname,'../RR18a/fixtures/escaped-import.json'),'utf8');
assert.equal(Buffer.byteLength(expanding),700258);fs.mkdirSync(out,{recursive:true});
(async()=>{
  const browser=await chromium.launch({headless:true}),results=[];
  try {
    for(const width of [320,1440]) {
      const page=await browser.newPage({viewport:{width,height:1000},serviceWorkers:'block'}),errors=[],warnings=[];
      page.on('pageerror',error=>errors.push(error.message));
      page.on('console',message=>{if(message.type()==='error')errors.push(message.text());if(message.type()==='warning')warnings.push(message.text());});
      await page.route('**/*',route=>new URL(route.request().url()).origin===new URL(url).origin?route.continue():route.abort());
      await page.addInitScript(()=>{window.__scalarTasks=[];if(PerformanceObserver.supportedEntryTypes.includes('longtask'))new PerformanceObserver(list=>window.__scalarTasks.push(...list.getEntries().map(e=>({start:e.startTime,duration:e.duration})))).observe({type:'longtask',buffered:true});});
      await page.goto(url);await page.getByRole('heading',{name:'SysON model editor',exact:true}).waitFor();
      const model=()=>page.locator('#export-json').inputValue();
      const importer=()=>page.locator('form[data-action="import"]');
      const importText=async text=>{await importer().locator('..').evaluate(e=>{e.open=true;});await page.locator('#import-json').fill(text);await importer().evaluate(form=>form.requestSubmit());};
      const document=JSON.parse(await model());
      document.content=[{id:'comment-resource',eClass:'sysml:Comment',data:{elementId:'comment0',declaredName:'Comment helper',body:literal,aliasIds:['a&','b<'],qualifiedName:'Readonly&<>'}}];
      await importText(JSON.stringify(document));const original=await model();
      assert.deepEqual(JSON.parse(original),document);
      await page.locator('button[data-value="comment0"]').click();
      assert.equal(await page.locator('#scalar-body').inputValue(),displayed);
      await page.locator('#unavailable-properties > summary').click();
      assert.match(await page.locator('#unavailable-properties').innerText(),/Readonly&<>/);
      for(const attribute of ['qualifiedName','aliasIds'])assert.equal(await page.locator(`form[data-attribute="${attribute}"]`).count(),0);
      const session=await page.context().newCDPSession(page);
      await session.send('Accessibility.enable');
      const {root:dom}=await session.send('DOM.getDocument',{depth:0});
      const {nodeId}=await session.send('DOM.querySelector',{nodeId:dom.nodeId,selector:'#scalar-body'});
      const {nodes}=await session.send('Accessibility.getPartialAXTree',{nodeId,fetchRelatives:false});
      const ax=nodes.find(node=>!node.ignored&&node.role?.value==='textbox');
      assert.ok(ax);assert.equal(ax.name?.value,'Body');assert.equal(ax.value?.value,displayed);
      const properties=Object.fromEntries(ax.properties.map(p=>[p.name,p.value.value]));
      assert.equal(properties.multiline,true);assert.equal(properties.readonly,false);
      await page.locator('#scalar-body').focus();
      await page.keyboard.press('Tab');
      assert.equal(await page.evaluate(()=>document.activeElement.id),'scalar-mode-body');
      await page.keyboard.press('Tab');assert.equal(await page.evaluate(()=>document.activeElement.id),'stage-attribute-body');
      await page.keyboard.press('Enter');assert.equal(await model(),original);
      assert.equal(await page.evaluate(()=>document.activeElement.id),'commit-staged');
      await page.keyboard.press('Enter');assert.equal(await model(),original);
      const changed=displayed+'changed';await page.locator('#scalar-body').fill(changed);
      await page.locator('#stage-attribute-body').click();
      await page.locator('#cancel-staged').click();
      assert.equal(await model(),original);
      await page.locator('#scalar-body').fill(changed);
      await page.locator('#stage-attribute-body').click();
      await page.locator('#commit-staged').click();
      const committed=await model();assert.equal(JSON.parse(committed).content[0].data.body,changed);
      await page.locator('#model-undo').click();
      assert.equal(await model(),original);
      await page.locator('#model-redo').click();
      assert.equal(await model(),committed);
      assert.match(await page.locator('#scalar-state-declaredShortName').innerText(),/Not set/);
      await page.locator('#scalar-mode-declaredShortName').selectOption('null');
      await page.locator('#stage-attribute-declaredShortName').click();
      await page.locator('#commit-staged').click();
      assert.equal(JSON.parse(await model()).content[0].data.declaredShortName,null);
      assert.match(await page.locator('#scalar-state-declaredShortName').innerText(),/Explicitly empty/);
      await page.locator('#model-undo').click();assert.equal(Object.hasOwn(JSON.parse(await model()).content[0].data,'declaredShortName'),false);
      const beforeRefusal=await model();
      await page.locator('#scalar-body').fill('unsubmitted scalar draft');
      await page.locator('#name').fill('unsubmitted creation draft');
      await importText(expanding);
      assert.equal(await model(),beforeRefusal);
      assert.equal(await page.locator('#scalar-body').inputValue(),'unsubmitted scalar draft');
      assert.equal(await page.locator('#name').inputValue(),'unsubmitted creation draft');
      const focus=await page.evaluate(()=>{const e=document.activeElement,r=e.getBoundingClientRect();return {id:e.id,top:r.top,bottom:r.bottom,visible:r.bottom>0&&r.top<innerHeight};});
      // Record focus visibility BEFORE screenshot auto-scroll; known viewport
      // recovery visibility and complete rendering/performance remain d OPEN.
      await page.screenshot({path:path.join(out,`scalar-${width}.png`)});
      assert.deepEqual(errors,[]);assert.deepEqual(warnings,[]);
      results.push({width,errors,warnings,bodyUtf8Bytes:Buffer.byteLength(literal),ax:{name:ax.name.value,role:ax.role.value,multiline:true,readonly:false},focus,longTasks:await page.evaluate(()=>window.__scalarTasks)});
      await session.detach();await page.close();
    }
    fs.writeFileSync(path.join(out,'scalar-helpers.json'),JSON.stringify({status:'PASS_SELECTED_DEFAULT_SCALAR_HELPERS',results,limits:'Helper output admission only; owned inspection copies, global renderer materialization, focus visibility, glyph coverage and <50ms acceptance remain OPEN.'},null,2));
  } finally {await browser.close();}
})().catch(error=>{console.error(error);process.exitCode=1;});
