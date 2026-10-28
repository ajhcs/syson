// SPDX-License-Identifier: EPL-2.0
// Fresh native artifact observation only: real UI save/reselect/reload, no mocked transport.
const assert = require('node:assert/strict'), fs = require('node:fs'), path = require('node:path');
const {chromium} = require(process.env.SYSON_PLAYWRIGHT_MODULE);
const base = process.env.SYSON_NATIVE_URL, output = process.env.SYSON_NATIVE_OUTPUT;
assert.ok(base && output && path.resolve(output).startsWith('/workspace/'));
assert.equal(new URL(base).hostname, '127.0.0.1');
const longName = 'Long <&> 雪🧭 ' + 'x'.repeat(4096) + ' END';
const geometry = element => {
  const r = element.getBoundingClientRect(), style = getComputedStyle(element);
  return {tag: element.tagName, role: element.getAttribute('role'), text: element.textContent, rect: {left:r.left, top:r.top, right:r.right, bottom:r.bottom, width:r.width, height:r.height},
    viewport: {width:innerWidth,height:innerHeight,pageLeft:scrollX,pageTop:scrollY,clientWidth:document.documentElement.clientWidth,scrollWidth:document.documentElement.scrollWidth},
    overflow: {x:style.overflowX,y:style.overflowY,text:style.textOverflow,whiteSpace:style.whiteSpace}, visible: r.width>0&&r.height>0&&r.right>0&&r.left<innerWidth&&r.bottom>0&&r.top<innerHeight};
};
(async () => {
  fs.mkdirSync(output,{recursive:true});
  const browser = await chromium.launch({headless:true, executablePath:process.env.SYSON_CHROMIUM_PATH});
  const results = [];
  try {
    for (const width of [320,1440]) {
      const context = await browser.newContext({viewport:{width:1440,height:1000},serviceWorkers:'block',acceptDownloads:true});
      const page = await context.newPage(), errors=[],warnings=[],blocked=[];
      let stage='setup', current;
      page.setDefaultTimeout(20000);
      page.on('pageerror',e=>errors.push(e.message));
      page.on('console',m=>{if(m.type()==='error')errors.push(m.text());if(m.type()==='warning')warnings.push(m.text());});
      await page.route('**/*',route=>{const u=new URL(route.request().url());if(u.origin===new URL(base).origin||['data:','blob:'].includes(u.protocol))return route.continue();blocked.push({url:u.href,method:route.request().method()});return route.abort();});
      await page.routeWebSocket('**/*',route=>{const u=new URL(route.url()), owned=new URL(base);if(u.hostname===owned.hostname&&u.port===owned.port&&u.protocol==='ws:')return route.connectToServer();blocked.push({url:u.href,method:'WEBSOCKET'});return route.close();});
      try {
        await page.goto(base,{waitUntil:'domcontentloaded'});
        assert.equal(await page.title(),'SysON');
        await page.locator('a[href="/new/project?templateId=sysmlv2-template"]').click();
        await page.getByPlaceholder('Enter project name').fill(`Batch13 Package ${width}`);
        await page.getByRole('button',{name:/create project/i}).click();
        await page.waitForURL(u=>/^\/projects\/[^/]+\/edit/.test(u.pathname),{timeout:90000});
        const projectURL=page.url(), explorer=()=>page.locator('[data-testid^="explorer://"]').first();
        const doc=()=>explorer().locator('[data-treeitemlabel="SysMLv2.sysml"]').first();
        await doc().waitFor({state:'visible',timeout:90000});
        await doc().dblclick();
        const packageItem=()=>explorer().locator('[data-treeitemlabel="Package1"]').first();
        await packageItem().waitFor();
        await page.setViewportSize({width,height:1000});
        await page.evaluate(()=>new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve))));
        stage='initial-package-selection';
        const initial=await packageItem().evaluate(geometry);
        await page.screenshot({path:path.join(output,`native-before-select-${width}.png`)});
        await packageItem().click();
        const field=()=>page.getByTestId('input-Declared Name').first();
        await field().waitFor();
        assert.equal(await field().inputValue(),'Package1');
        const beforeField=await field().evaluate(geometry);
        stage='save-long-name';
        const start=Date.now();
        await field().fill(longName);
        await field().press('Tab');
        await page.waitForFunction(expected=>Array.from(document.querySelectorAll('[data-treeitemlabel]')).some(e=>e.getAttribute('data-treeitemlabel')===expected||e.getAttribute('data-treeitemlabel')?.includes(expected)),longName,{timeout:60000});
        const saveObservationMs=Date.now()-start;
        const matching=()=>explorer().locator('[data-treeitemlabel]').filter({hasText:longName}).first();
        await matching().waitFor();
        assert.equal(await field().inputValue(),longName);
        const navigation=await matching().evaluate(geometry);
        await page.screenshot({path:path.join(output,`native-long-name-${width}.png`)});
        const cdp=await context.newCDPSession(page);
        const ax=await cdp.send('Accessibility.getFullAXTree');
        const axMatches=ax.nodes.filter(n=>n.name?.value?.includes(longName)).map(n=>({role:n.role?.value,name:n.name?.value,ignored:n.ignored,properties:n.properties}));
        stage='reselect';
        await doc().click();
        await matching().click();
        await field().waitFor();assert.equal(await field().inputValue(),longName);
        stage='reload';
        await page.reload({waitUntil:'domcontentloaded'});
        await doc().waitFor({state:'visible',timeout:90000});
        if(await matching().count()===0)await doc().dblclick();
        await matching().waitFor();
        const reloadBeforeSelect=await matching().evaluate(geometry);
        await page.screenshot({path:path.join(output,`native-reload-before-select-${width}.png`)});
        await matching().click();await field().waitFor();assert.equal(await field().inputValue(),longName);
        stage='native-full-control-access';
        const control=await field().evaluate(e=>({tag:e.tagName,type:e.type,value:e.value,readOnly:e.readOnly,disabled:e.disabled,maxLength:e.maxLength,scrollWidth:e.scrollWidth,clientWidth:e.clientWidth}));
        await field().click();await page.keyboard.press('Control+A');
        const fullSelection=await field().evaluate(e=>({start:e.selectionStart,end:e.selectionEnd,direction:e.selectionDirection,selected:e.value.slice(e.selectionStart,e.selectionEnd)}));
        assert.equal(fullSelection.selected,longName);
        stage='advanced-readonly-reference';
        const advanced=page.getByTestId('page-tab-Advanced');let readonlyReference={status:'ADVANCED_TAB_NOT_FOUND'};
        if(await advanced.count()){
          await advanced.first().click();
          await page.evaluate(()=>new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve))));
          const details=page.getByTestId('view-Details');
          readonlyReference={status:'OBSERVED_ARTIFACT_WIDGETS_ONLY',text:await details.innerText(),controls:await details.locator('input,textarea,[role="textbox"],label,[data-testid^="label-"]').evaluateAll(es=>es.map(e=>({tag:e.tagName,testid:e.dataset.testid,label:e.getAttribute('aria-label'),value:e.value??null,readOnly:e.readOnly??null,text:e.textContent}))),bounds:await details.evaluate(geometry)};
          await page.screenshot({path:path.join(output,`native-advanced-readonly-${width}.png`)});
        }
        current={width,title:await page.title(),projectURL,url:page.url(),longName,utf16Units:longName.length,utf8Bytes:Buffer.byteLength(longName),initial,beforeField,navigation,reloadBeforeSelect,control,fullSelection,axMatches,readonlyReference,saveObservationMs,errors,warnings,blocked,
          scope:'Actual Package.DeclaredName UI save/reselect/reload and native full-control selection on SysON2026.9.3 artifact. One-shot latency includes Playwright actions/transport, not distribution or cross-product performance ratio. Advanced readonly label is native-widget observation, not equivalence to Rust JSON disclosure or aliasIds.'};
        results.push(current);assert.equal(errors.length,0);assert.equal(blocked.length,0);
      } catch(error) {
        fs.writeFileSync(path.join(output,`native-failure-${width}.json`),JSON.stringify({width,stage,error:String(error),url:page.url(),title:await page.title().catch(()=>null),text:await page.locator('body').innerText().catch(()=>null),controls:await page.locator('input,textarea,[role="tab"],[data-treeitemlabel]').evaluateAll(es=>es.map(e=>({tag:e.tagName,testid:e.dataset.testid,label:e.getAttribute('aria-label'),treeLabel:e.getAttribute('data-treeitemlabel'),value:e.value??null,text:e.textContent}))).catch(()=>null),errors,warnings,blocked},null,2));
        await page.screenshot({path:path.join(output,`native-failure-${width}.png`)}).catch(()=>{});throw error;
      } finally {await context.close();}
    }
    fs.writeFileSync(path.join(output,'native-results.json'),JSON.stringify({status:'PASS_BOUNDED_NATIVE_ARTIFACT_OBSERVATION',results},null,2));
  } finally {await browser.close();}
})().catch(error=>{console.error(error);process.exitCode=1;});
