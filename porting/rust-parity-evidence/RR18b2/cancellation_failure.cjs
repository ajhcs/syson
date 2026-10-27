// SPDX-License-Identifier: EPL-2.0
// Owned scheduler/platform faults around actual WASM actions; no model substitute.
const assert=require('node:assert/strict'),fs=require('node:fs'),path=require('node:path');
const {chromium}=require(process.env.SYSON_PLAYWRIGHT_MODULE||'playwright');
const url=process.env.SYSON_PREVIEW_URL,output=process.env.SYSON_BROWSER_ARTIFACT_DIR;
assert.ok(url&&output&&path.resolve(output).startsWith('/workspace/'));
assert.ok(['127.0.0.1','localhost','[::1]'].includes(new URL(url).hostname));
fs.mkdirSync(output,{recursive:true});
(async()=>{
 const browser=await chromium.launch({headless:true}),results=[];
 try {
  for(const fault of ['encode','decode']) {
   const page=await browser.newPage({viewport:{width:390,height:844},serviceWorkers:'block'}),errors=[];
   page.setDefaultTimeout(15000);page.on('pageerror',e=>errors.push(e.message));page.on('console',m=>{if(m.type()==='error')errors.push(m.text());});
   await page.route('**/*',route=>new URL(route.request().url()).origin===new URL(url).origin?route.continue():route.abort());
   await page.addInitScript(()=>{
    const timeout=window.setTimeout;window.__ownedHeld=[];window.__ownedHold=false;
    window.setTimeout=function(callback,ms,...args){if(window.__ownedHold&&ms===0){window.__ownedHeld.push(()=>callback(...args));return 0;}return timeout.call(this,callback,ms,...args);};
   });
   await page.goto(url);await page.getByRole('heading',{name:'SysON model editor',exact:true}).waitFor();
   await page.locator('#name').fill('Before');await page.locator('#stage-create').click();await page.locator('#commit-staged').click();
   const before=await page.locator('#export-json').inputValue();
   await page.evaluate(()=>{window.__ownedHold=true;});
   await page.getByRole('button',{name:'Layout canvas',exact:true}).click();await page.waitForFunction(()=>document.querySelector('[data-canvas-ticket]'));
   assert.ok(await page.evaluate(()=>window.__ownedHeld.length>0));
   await page.locator('form[data-action="import"]').locator('..').evaluate(e=>{e.open=true;});
   await page.locator('#import-json').fill('x'.repeat(1048576));
   await page.evaluate(()=>document.fonts.dispatchEvent(new Event('loadingdone')));
   assert.match(await page.locator('[data-platform-error]').innerText(),/cancellation is pending/);
   await page.locator('#import-json').fill('short');
   await page.evaluate(fault=>{
    if(fault==='decode') {
     const decode=TextDecoder.prototype.decode;TextDecoder.prototype.decode=function(...args){TextDecoder.prototype.decode=decode;throw new Error('owned cancel decode fault');};
    } else {
     const encode=TextEncoder.prototype.encode;TextEncoder.prototype.encode=function(text){if(text.includes('"action":"canvas_cancel"')){TextEncoder.prototype.encode=encode;throw new Error('owned cancel encode fault');}return encode.call(this,text);};
    }
   },fault);
   await page.locator('#model-redo').click();
   assert.match(await page.locator('[data-platform-error]').innerText(),fault==='decode'?/output became uncertain/:/owned cancel encode fault/);
   await page.evaluate(()=>{while(window.__ownedHeld.length)window.__ownedHeld.shift()();});
   await page.locator('#model-redo').click();
   if(fault==='decode') {
    assert.match(await page.locator('[data-platform-error]').innerText(),/Canvas cancellation is pending/);
    await page.getByRole('button',{name:'Cancel canvas work',exact:true}).click();
   }
   assert.equal(await page.locator('[data-canvas-ticket]').count(),0);
   assert.equal(await page.locator('#export-geometry').inputValue(),'');
   assert.equal(await page.locator('#export-json').inputValue(),before);
   await page.evaluate(()=>{window.__ownedHold=false;});
   await page.getByRole('button',{name:'Layout canvas',exact:true}).click();
   await page.waitForFunction(()=>!document.querySelector('[data-canvas-ticket]')&&document.getElementById('export-geometry').value);
   assert.equal(await page.locator('#export-json').inputValue(),before);
   assert.deepEqual(errors,[]);results.push({fault,modelUnchanged:true,newLayoutAvailable:true,errors});await page.close();
  }
  fs.writeFileSync(path.join(output,'cancellation-failure.json'),JSON.stringify({status:'PASS_ACTUAL_CANCELLATION_BARRIER_FAULTS',results},null,2));
 } finally {await browser.close();}
})().catch(error=>{console.error(error);process.exitCode=1;});
