// SPDX-License-Identifier: EPL-2.0
// Owned scheduler guard: delay real application callbacks; never replace model/geometry.
const assert=require('node:assert/strict'),fs=require('node:fs'),path=require('node:path');
const {chromium}=require(process.env.SYSON_PLAYWRIGHT_MODULE);
(async()=>{const b=await chromium.launch({headless:true});try{const p=await b.newPage({viewport:{width:390,height:844}});
 await p.addInitScript(()=>{const original=window.setTimeout;window.__held=[];window.__hold=false;window.setTimeout=function(fn,ms,...args){if(window.__hold&&ms===0){window.__held.push(()=>fn(...args));return 0;}return original.call(this,fn,ms,...args);};});
 await p.goto(process.env.SYSON_PREVIEW_URL);await p.getByRole('heading',{name:'SysON model editor',exact:true}).waitFor();
 await p.locator('#name').fill('Before');await p.locator('#stage-create').click();await p.locator('#commit-staged').click();const before=await p.locator('#export-json').inputValue();
 await p.evaluate(()=>window.__hold=true);await p.getByRole('button',{name:'Layout canvas',exact:true}).click();await p.waitForFunction(()=>document.querySelector('[data-canvas-ticket]'));
 const ticket=await p.locator('[data-canvas-ticket]').getAttribute('data-canvas-ticket');
 await p.locator('form[data-action="import"]').locator('..').evaluate(e=>e.open=true);await p.locator('#import-json').fill('x'.repeat(1048576));
 await p.evaluate(()=>{const fn=window.__held.shift();if(!fn)throw Error('No actual queued canvas callback');fn();});
 await p.evaluate(()=>document.fonts.dispatchEvent(new Event('loadingdone')));
 const afterFont={pending:await p.locator('[data-canvas-ticket]').count(),notice:await p.locator('[data-platform-error]').innerText()};
 await p.locator('#import-json').fill('short');await p.locator('#model-redo').click();
 const afterRedo={pending:await p.locator('[data-canvas-ticket]').count(),queued:await p.evaluate(()=>window.__held.length)};
 await p.evaluate(()=>{for(let n=0;n<4&&window.__held.length;n++)window.__held.shift()();window.__hold=false;});
 const result={ticket,afterFont,afterRedo,publishedGeometryBytes:Buffer.byteLength(await p.locator('#export-geometry').inputValue()),modelUnchanged:await p.locator('#export-json').inputValue()===before};
 fs.writeFileSync(path.join(process.env.SYSON_BROWSER_ARTIFACT_DIR,'invalidation-observation.json'),JSON.stringify(result,null,2));await p.screenshot({path:path.join(process.env.SYSON_BROWSER_ARTIFACT_DIR,'invalidation.png')});console.log(JSON.stringify(result));
 assert.equal(afterRedo.pending,0,'mandatory font cancellation must prevent the prior canvas ticket being requeued');assert.equal(result.publishedGeometryBytes,0,'cancelled work must not publish after quota is reduced');assert.equal(result.modelUnchanged,true);
}finally{await b.close()}})().catch(e=>{console.error(e);process.exitCode=1});
