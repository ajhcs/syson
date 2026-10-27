// SPDX-License-Identifier: EPL-2.0
// Real WASM rendering and native keyboard scroll; no physical-touch qualification.
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const {chromium} = require(process.env.SYSON_PLAYWRIGHT_MODULE || 'playwright');
const url = process.env.SYSON_PREVIEW_URL, output = process.env.SYSON_BROWSER_ARTIFACT_DIR;
assert.ok(url && output, 'Set preview URL and browser artifact directory.');
const origin = new URL(url).origin;
assert.ok(['127.0.0.1','localhost','[::1]'].includes(new URL(url).hostname));
fs.mkdirSync(output, {recursive:true});
const uuid = (prefix, n) => `${prefix}0000000-0000-4000-8000-${String(n).padStart(12,'0')}`;
const node = (n, eClass, data) => ({id:uuid('1',n),eClass:`sysml:${eClass}`,data:{elementId:uuid('2',n),...data}});
const fixture = {json:{version:'1.0',encoding:'utf-8'},ns:{sysml:'http://www.eclipse.org/syson/sysml'},content:[
  node(1,'Package',{declaredName:'Assembly',ownedRelationship:Array.from({length:31},(_,n)=>
    node(100+n,'OwningMembership',{ownedRelatedElement:[node(2+n,'PartDefinition',{declaredName:`Part ${String(n+1).padStart(2,'0')}`})]}))})
]};
(async () => {
  const browser = await chromium.launch({headless:true});
  const results = [];
  try {
    for (const width of [320,1440]) {
      const page = await browser.newPage({viewport:{width,height:1000},serviceWorkers:'block'});
      const errors = [], warnings = [];
      page.on('pageerror', e=>errors.push(e.message));
      page.on('console', m=>{if(m.type()==='error')errors.push(m.text());if(m.type()==='warning')warnings.push(m.text());});
      await page.route('**/*', route=>{
        if (new URL(route.request().url()).origin===origin) return route.continue();
        errors.push(`External request: ${route.request().url()}`); return route.abort();
      });
      // Count calls transparently: original bytes instantiate and every model
      // dispatch invokes the real exported WASM function, with no mocked state.
      await page.addInitScript(()=>{
        const instantiate = WebAssembly.instantiate.bind(WebAssembly);
        window.__modelDispatches = 0;
        WebAssembly.instantiate = async (...args)=>{
          const result = await instantiate(...args);
          if (!result.instance) return result;
          const exports = result.instance.exports;
          return {...result,instance:{exports:{...exports,dispatch:(...input)=>{
            window.__modelDispatches++; return exports.dispatch(...input);
          }}}};
        };
      });
      await page.goto(url);
      await page.getByRole('heading',{name:'SysON model editor',exact:true}).waitFor();
      assert.equal(await page.title(),'SysON · Rust model editor');
      await page.evaluate(()=>document.fonts.ready);
      await page.locator('form[data-action="import"]').locator('..').locator('summary').click();
      await page.locator('#import-json').fill(JSON.stringify(fixture));
      await page.getByRole('button',{name:'Import as new document and clear history',exact:true}).click();
      assert.equal(await page.locator('[data-canvas-label]').count(),32);
      await page.getByRole('button',{name:'Layout canvas',exact:true}).click();
      await page.waitForFunction(()=>{
        const raw = document.getElementById('export-geometry')?.value;
        if (!raw || document.querySelector('[data-canvas-ticket]')) return false;
        const companion = JSON.parse(raw);
        return companion.version===1 && Array.isArray(companion.nodes) && companion.nodes.length===32;
      });
      const viewport = page.getByRole('region',{name:'Diagram viewport',exact:true});
      const actual = page.getByRole('button',{name:'Actual size',exact:true});
      const fit = page.getByRole('button',{name:'Fit overview',exact:true});
      const model = ()=>page.locator('#export-json').inputValue();
      const geometry = ()=>page.locator('#export-geometry').inputValue();
      const noBodyOverflow = async()=>assert.ok(await page.evaluate(()=>document.documentElement.scrollWidth<=document.documentElement.clientWidth));
      const measurements = ()=>viewport.locator('svg').evaluate(svg=>{
        const matrix=svg.getScreenCTM(), scale=Math.hypot(matrix.a,matrix.b);
        const name=svg.querySelector('[data-canvas-label]'), kind=svg.querySelector('text');
        return {scale,nameHeight:name.getBoundingClientRect().height,
          namePixels:parseFloat(getComputedStyle(name).fontSize)*scale,
          kindPixels:parseFloat(getComputedStyle(kind).fontSize)*scale,
          labels:svg.querySelectorAll('[data-canvas-label]').length};
      });
      const readable = m=>{
        assert.ok(Math.abs(m.scale-1)<0.005,`Actual size is 1:1: ${JSON.stringify(m)}`);
        assert.ok(m.namePixels>=13.9 && m.kindPixels>=11.9 && m.nameHeight>=23.8);
        assert.equal(m.labels,32);
      };
      const initial=await measurements(); readable(initial); await noBodyOverflow();
      assert.equal(await actual.getAttribute('aria-pressed'),'true');
      for (const control of [actual,fit]) {
        const box=await control.boundingBox(); assert.ok(box.width>=44&&box.height>=44);
      }
      await page.locator('#name').fill(`Keep this draft ${width}`);
      await page.locator('#name').evaluate(e=>e.setSelectionRange(2,7,'backward'));
      await page.locator('#geometry-details summary').click();
      await page.locator('#import-geometry').fill('Unsubmitted geometry draft');
      const state = async()=>({model:await model(),geometry:await geometry(),
        name:await page.locator('#name').inputValue(),draft:await page.locator('#import-geometry').inputValue(),
        details:await page.locator('#geometry-details').evaluate(e=>e.open),
        caret:await page.locator('#name').evaluate(e=>[e.selectionStart,e.selectionEnd,e.selectionDirection])});
      const before=await state();
      await page.locator('.canvas-view-controls').evaluate(e=>e.scrollIntoView({block:'start'}));
      await page.screenshot({path:path.join(output,`canvas-actual-${width}.png`),fullPage:false});
      // Native arrow-key panning operates inside the region, not the page.
      await viewport.focus();
      assert.equal(await page.evaluate(()=>document.activeElement.id),'canvas-viewport');
      const axis=await viewport.evaluate(e=>e.scrollHeight>e.clientHeight+1?'y':'x');
      await page.keyboard.press(axis==='y'?'ArrowDown':'ArrowRight');
      await page.waitForFunction(axis=>{
        const e=document.getElementById('canvas-viewport'); return axis==='y'?e.scrollTop>0:e.scrollLeft>0;
      },axis);
      const pan=await viewport.evaluate(e=>{e.scrollTo(240,180);return {left:e.scrollLeft,top:e.scrollTop};});
      assert.ok(pan.left>0||pan.top>0);
      const dispatches=await page.evaluate(()=>window.__modelDispatches);
      await page.evaluate(()=>{window.__viewNodes={draft:document.getElementById('name'),details:document.getElementById('geometry-details'),fit:document.getElementById('canvas-fit')};});
      await fit.focus(); await page.keyboard.press('Enter');
      assert.equal(await fit.getAttribute('aria-pressed'),'true');
      assert.equal(await page.evaluate(()=>document.activeElement===window.__viewNodes.fit),true);
      assert.equal(await page.evaluate(()=>document.getElementById('name')===window.__viewNodes.draft && document.getElementById('geometry-details')===window.__viewNodes.details),true);
      assert.equal(await page.evaluate(()=>window.__modelDispatches),dispatches);
      assert.deepEqual(await state(),before);
      const overview=await measurements(); assert.ok(overview.scale<1);
      assert.ok(await viewport.evaluate(e=>e.scrollWidth<=e.clientWidth+1 && e.scrollHeight<=e.clientHeight+1));
      await noBodyOverflow();
      await page.locator('.canvas-view-controls').evaluate(e=>e.scrollIntoView({block:'start'}));
      await page.screenshot({path:path.join(output,`canvas-fit-${width}.png`),fullPage:false});
      // A simulated font event invokes the real background WASM handler while
      // the draft input stays focused: this is the existing focused-caret contract.
      // A blurred input's selection is not promised across DOM replacement.
      await page.locator('#name').focus();
      await page.evaluate(()=>document.fonts.dispatchEvent(new Event('loading')));
      assert.equal(await fit.getAttribute('aria-pressed'),'true');
      assert.equal(await page.evaluate(()=>document.activeElement.id),'name');
      assert.deepEqual(await state(),before);
      const afterPaint=await page.evaluate(()=>window.__modelDispatches);
      await actual.focus(); await page.keyboard.press('Enter');
      assert.equal(await actual.getAttribute('aria-pressed'),'true');
      assert.equal(await page.evaluate(()=>document.activeElement.id),'canvas-actual-size');
      assert.equal(await page.evaluate(()=>window.__modelDispatches),afterPaint);
      readable(await measurements()); await noBodyOverflow();
      assert.deepEqual(await state(),before);
      const restored=await viewport.evaluate(e=>({left:e.scrollLeft,top:e.scrollTop}));
      assert.ok(Math.abs(restored.left-pan.left)<=1 && Math.abs(restored.top-pan.top)<=1);
      await page.locator('.canvas-view-controls').evaluate(e=>e.scrollIntoView({block:'start'}));
      await page.screenshot({path:path.join(output,`canvas-restored-${width}.png`),fullPage:false});
      // Imported document history is empty; presentation toggles add no entry.
      await page.getByRole('button',{name:'Undo',exact:true}).click();
      assert.match(await page.getByRole('status').innerText(),/No history available/);
      assert.equal(await model(),before.model); assert.equal(await geometry(),before.geometry);
      assert.deepEqual(errors,[]); assert.deepEqual(warnings,[]);
      results.push({width,initial,overview,restored,errors,warnings});
      await page.close();
    }
    const result={status:'PASS',fixtureNodes:32,results,checks:[
      'actual-size native label dimensions at320/1440px','explicit Fit overview and return',
      'native focusable keyboard scroll','local overflow without document horizontal overflow',
      'no WASM dispatch on presentation toggles','draft/caret/disclosure/DOM identities retained',
      'mode/pan and focused caret retained across simulated-font-event repaint','model/geometry bytes and model history unchanged'
    ],limits:'Chromium keyboard and viewport emulation; physical touch, screen-reader speech and other browsers unqualified'};
    fs.writeFileSync(path.join(output,'result.json'),JSON.stringify(result,null,2));
    console.log(JSON.stringify(result));
  } finally {await browser.close();}
})().catch(error=>{console.error(error);process.exitCode=1;});
