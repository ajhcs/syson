// SPDX-License-Identifier: EPL-2.0
// Observation only: real WASM exports/arguments and DOM setters are delegated unchanged.
const assert=require('node:assert/strict'),fs=require('node:fs'),path=require('node:path');
const {chromium}=require(process.env.SYSON_PLAYWRIGHT_MODULE);
const url=process.env.SYSON_PREVIEW_URL,out=process.env.SYSON_BROWSER_ARTIFACT_DIR;
const fixture=fs.readFileSync(path.join(__dirname,'RR18a/fixtures/escaped-import.json'),'utf8');
const hugeName=JSON.parse(fixture).content[0].data.declaredName;
(async()=>{const browser=await chromium.launch({headless:true});const results=[];try{
 for(const width of [320,1440]){
  const p=await browser.newPage({viewport:{width,height:1000},serviceWorkers:'block',acceptDownloads:true});
  const cdp=await p.context().newCDPSession(p);await cdp.send('Tracing.start',{categories:'devtools.timeline,blink.user_timing,v8,disabled-by-default-devtools.timeline',transferMode:'ReturnAsStream'});
  const errors=[],warnings=[];p.on('pageerror',e=>errors.push(e.message));p.on('console',m=>{if(m.type()==='error')errors.push(m.text());if(m.type()==='warning')warnings.push(m.text())});
  await p.route('**/*',r=>new URL(r.request().url()).origin===new URL(url).origin||r.request().url().startsWith('data:')?r.continue():r.abort());
  await p.addInitScript(()=>{
   window.__phase={events:[],tasks:[],scenario:'initialize',action:'initialize'};
   const observe=(phase,started,units)=>{const q=window.__phase;if(q.events.length<20000)q.events.push({scenario:q.scenario,action:q.action,phase,start:started,duration:performance.now()-started,units});};
   const instantiate=WebAssembly.instantiate;
   WebAssembly.instantiate=async function(...args){const original=await instantiate.apply(this,args);const actual=original.instance.exports,api={...actual};let started=0;
    api.input_reset=(...v)=>{const r=actual.input_reset(...v);started=performance.now();return r;};
    api.dispatch=(...v)=>{observe('ABI byte transfer',started);const t=performance.now();try{return actual.dispatch(...v);}finally{observe('Rust dispatch including parse/handle/render',t);}};
    return {module:original.module,instance:{exports:api}};
   };
   const encode=TextEncoder.prototype.encode;TextEncoder.prototype.encode=function(value){const match=typeof value==='string'&&/^\{"action":"([^"]{1,64})"/.exec(value.slice(0,128));if(match)window.__phase.action=match[1];const t=performance.now();try{return encode.call(this,value);}finally{observe('UTF8 input encoding',t,typeof value==='string'?value.length:null);}};
   const decode=TextDecoder.prototype.decode;TextDecoder.prototype.decode=function(value,...args){const t=performance.now();try{return decode.call(this,value,...args);}finally{observe('UTF8 output decoding',t,value?.byteLength);}};
   const inner=Object.getOwnPropertyDescriptor(Element.prototype,'innerHTML');Object.defineProperty(Element.prototype,'innerHTML',{...inner,set(value){const t=performance.now();try{return inner.set.call(this,value);}finally{if(this instanceof HTMLTemplateElement)observe('Template HTML parse',t,value.length);}}});
   const insert=Node.prototype.insertBefore;Node.prototype.insertBefore=function(...args){const t=performance.now();try{return insert.apply(this,args);}finally{if(this.id==='app')observe('DOM insertion',t);}};
   if(PerformanceObserver.supportedEntryTypes.includes('longtask'))new PerformanceObserver(list=>window.__phase.tasks.push(...list.getEntries().map(x=>({start:x.startTime,duration:x.duration})))).observe({type:'longtask',buffered:true});
  });
  await p.goto(url);await p.getByRole('heading',{name:'SysON model editor',exact:true}).waitFor();
  assert.equal(await p.title(),'SysON · Rust model editor');assert.equal(p.url(),url);
  const scenario=async name=>p.evaluate(n=>{window.__phase.scenario=n;performance.mark('syson-scenario:'+n)},name);
  const model=()=>p.locator('#export-json').inputValue();
  const layout=async()=>{await p.getByRole('button',{name:'Layout canvas',exact:true}).click();await p.waitForFunction(()=>!document.querySelector('[data-canvas-ticket]')&&document.getElementById('export-geometry')?.value);};
  await scenario('small model setup');await p.locator('#name').fill('Before');await p.locator('#stage-create').click();await p.locator('#commit-staged').click();await layout();
  const before=await model();await p.locator('form[data-action="import"]').locator('..').evaluate(e=>e.open=true);
  await scenario('700258-byte expanding import refusal');await p.locator('#import-json').fill(fixture);await p.locator('form[data-action="import"]').evaluate(f=>f.requestSubmit());assert.equal(await model(),before);
  const admitted=JSON.parse(before);admitted.opaque='x'.repeat(900000);
  await scenario('900k opaque import and layout');await p.locator('#import-json').fill(JSON.stringify(admitted));await p.locator('form[data-action="import"]').evaluate(f=>f.requestSubmit());const committed=await model();assert.equal(Buffer.byteLength(committed),900381);await layout();const geometry=await p.locator('#export-geometry').inputValue();
  await scenario('staged escaped-name fallback');await p.locator('.tree button').filter({hasText:/^BeforePackage$/}).click();await p.locator('#rename').fill(hugeName);await p.locator('form[data-action="rename"]').evaluate(f=>f.requestSubmit());await p.locator('#recovery-export').waitFor();
  const href=await p.locator('#recovery-export').getAttribute('href');assert.equal(decodeURIComponent(href.split(',').slice(1).join(',')),committed);
  await scenario('native recovery download');const pending=p.waitForEvent('download');await p.locator('#recovery-export').focus();await p.keyboard.press('Enter');const download=await pending;const file=path.join(out,`phase-export-${width}.json`);await download.saveAs(file);assert.equal(fs.readFileSync(file,'utf8'),committed);
  await p.screenshot({path:path.join(out,`phase-recovery-${width}.png`)});
  await scenario('recover committed view');await p.locator('#recover-preview').focus();await p.keyboard.press('Enter');assert.equal(await model(),committed);assert.equal(await p.locator('#export-geometry').inputValue(),geometry);
  assert.ok(await p.evaluate(()=>document.documentElement.scrollWidth<=document.documentElement.clientWidth));assert.deepEqual(errors,[]);assert.deepEqual(warnings,[]);
  const phases=await p.evaluate(()=>window.__phase);const summary={};for(const e of phases.events){const key=e.scenario+' / '+e.phase;const q=summary[key]??={count:0,total:0,max:0};q.count++;q.total+=e.duration;q.max=Math.max(q.max,e.duration);}
  const complete=new Promise(resolve=>cdp.once('Tracing.tracingComplete',resolve));await cdp.send('Tracing.end');const {stream}=await complete;const chunks=[];while(true){const part=await cdp.send('IO.read',{handle:stream});chunks.push(Buffer.from(part.data,part.base64Encoded?'base64':'utf8'));if(part.eof)break;}await cdp.send('IO.close',{handle:stream});fs.writeFileSync(path.join(out,`trace-${width}.json`),Buffer.concat(chunks));await cdp.detach();
  results.push({width,title:await p.title(),downloadBytes:Buffer.byteLength(committed),uriBytes:Buffer.byteLength(href),errors,warnings,phases,summary});await p.close();
 }
 fs.writeFileSync(path.join(out,'phase-results.json'),JSON.stringify({status:'PASS_INSTRUMENTED_SELECTED_UI_OBSERVATIONS',scope:'Real delegates, no model/HTML substitution. Instrumented phase observations; Rust parse/handle/render remain one opaque export. Not an uninstrumented <50ms acceptance result.',results},null,2));
 console.log(JSON.stringify(results.map(x=>({width:x.width,tasks:x.phases.tasks,summary:x.summary}))));
}finally{await browser.close();}})().catch(e=>{console.error(e);process.exitCode=1});
