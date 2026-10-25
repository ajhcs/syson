// SPDX-License-Identifier: EPL-2.0
// Actual keyboard/Chromium accessibility evidence; no screen-reader speech claim.
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const {chromium} = require(process.env.SYSON_PLAYWRIGHT_MODULE || 'playwright');
const url = process.env.SYSON_PREVIEW_URL, output = process.env.SYSON_BROWSER_ARTIFACT_DIR;
if (!url || !output) throw new Error('Set SYSON_PREVIEW_URL and SYSON_BROWSER_ARTIFACT_DIR.');
const origin = new URL(url).origin;
if (!['127.0.0.1', 'localhost', '[::1]'].includes(new URL(url).hostname)) throw new Error('Loopback preview required.');
fs.mkdirSync(output, {recursive:true});
(async () => {
  const browser = await chromium.launch({headless:true});
  try {
    const page = await browser.newPage({serviceWorkers:'block',viewport:{width:1365,height:1000}});
    const errors = [], warnings = [], focus = [], ax = [];
    page.on('pageerror', e => errors.push(e.message));
    page.on('console', m => { if (m.type() === 'error') errors.push(m.text()); if (m.type() === 'warning') warnings.push(m.text()); });
    await page.route('**/*', route => {
      if (new URL(route.request().url()).origin === origin) return route.continue();
      errors.push('External request: ' + route.request().url()); return route.abort();
    });
    await page.goto(url);
    await page.getByRole('heading',{name:'SysON model editor',exact:true}).waitFor();
    assert.equal(await page.title(),'SysON · Rust model editor');
    assert.equal(await page.getByRole('region',{name:'Element inspector',exact:true}).count(),1);
    const model = () => page.locator('#export-json').inputValue();
    const press = key => page.keyboard.press(key);
    const expectFocus = async (id, state) => {
      const actual = await page.evaluate(() => document.activeElement.id);
      assert.equal(actual,id,state); focus.push({state,id:actual});
    };
    await page.evaluate(() => {
      window.inspectorLive = document.getElementById('editor-status');
      window.inspectorAnnouncements = [];
      window.inspectorRegionRemoved = false;
      window.inspectorAttrsCleared = false;
      new MutationObserver(records => {
        for (const record of records) {
          for (const removed of record.removedNodes) {
            if (removed === window.inspectorLive || removed.contains?.(window.inspectorLive)) window.inspectorRegionRemoved = true;
          }
          if (record.type === 'attributes' && ['role','aria-live','aria-atomic'].includes(record.attributeName)
              && record.target === window.inspectorLive && (record.oldValue === null || !record.target.hasAttribute(record.attributeName))) window.inspectorAttrsCleared = true;
        }
        window.inspectorAnnouncements.push(window.inspectorLive.textContent);
      }).observe(document.getElementById('app'),{childList:true,subtree:true,attributes:true,attributeOldValue:true});
    });
    const cdp = await page.context().newCDPSession(page);
    await cdp.send('Accessibility.enable');
    const dom = await cdp.send('DOM.getDocument');
    const statusDom = await cdp.send('DOM.querySelector',{nodeId:dom.root.nodeId,selector:'#editor-status'});
    const {node:statusNode} = await cdp.send('DOM.describeNode',{nodeId:statusDom.nodeId});
    const statusBackend = statusNode.backendNodeId;
    const snapshot = async name => {
      const tree = await cdp.send('Accessibility.getFullAXTree');
      fs.writeFileSync(path.join(output,`ax-${name}.json`),JSON.stringify(tree,null,2));
      const status = tree.nodes.find(n=>n.backendDOMNodeId===statusBackend && !n.ignored);
      assert.ok(status,'Persistent status exposes the same backend node');
      assert.equal(status.role.value,'status');
      assert.equal(status.properties.find(p=>p.name==='live')?.value.value,'polite');
      assert.equal(status.properties.find(p=>p.name==='atomic')?.value.value,true);
      const focused = tree.nodes.find(n=>n.role?.value!=='RootWebArea' && n.properties?.some(p=>p.name==='focused' && p.value.value===true));
      ax.push({name,statusBackend,focusedRole:focused?.role?.value,focusedName:focused?.name?.value});
      return focused;
    };
    // Enter the creation form, then use actual native select/Tab/typing/Enter keys.
    await page.locator('#kind').focus();
    const create = async (kind,name,typed) => {
      await expectFocus('kind',`create ${name}: kind`);
      await press('Home');
      for (let i=0;i<kind;i++) await press('ArrowDown');
      await press('Tab'); await expectFocus('parent',`create ${name}: parent`);
      if (kind) { await press('Home'); await press('ArrowDown'); }
      await press('Tab'); await expectFocus('name',`create ${name}: name`);
      await page.keyboard.type(name);
      await press('Tab'); await expectFocus('definition',`create ${name}: definition`);
      if (typed) { await press('Home'); await press('ArrowDown'); }
      await press('Tab'); await expectFocus('stage-create',`create ${name}: stage`);
      const before = await model(); await press('Enter');
      await expectFocus('commit-staged',`create ${name}: staged`);
      assert.equal(await model(),before);
      await press('Enter'); await expectFocus('name',`create ${name}: committed`);
      assert.match(await page.getByRole('status').innerText(),/committed/);
      await press('Shift+Tab'); await press('Shift+Tab');
    };
    await create(0,'Vehicle',false);
    await create(1,'Wheel',false);
    await create(2,'wheel',true);
    assert.match(await model(),/sysml:FeatureTyping/);
    const committed = await model();
    await page.getByRole('button',{name:'wheel PartUsage',exact:true}).focus();
    await press('Enter'); await expectFocus('rename','select usage');
    let focused = await snapshot('selection');
    assert.equal(focused.role.value,'textbox'); assert.equal(focused.name.value,'Element name');
    // Stage/cancel, then stage/commit through the same keyboard path.
    await press('Control+A'); await page.keyboard.type('discarded name');
    await press('Tab'); await expectFocus('stage-rename','rename stage control');
    await press('Enter'); await expectFocus('commit-staged','rename staged');
    await press('Tab'); await expectFocus('cancel-staged','cancel control');
    await press('Enter'); await expectFocus('rename','rename cancelled');
    assert.equal(await page.locator('#rename').inputValue(),'wheel'); assert.equal(await model(),committed);
    await press('Control+A'); await page.keyboard.type('frontWheel');
    await press('Tab'); await press('Enter'); await expectFocus('commit-staged','rename restaged');
    await press('Enter'); await expectFocus('rename','rename committed');
    assert.equal(await page.locator('#rename').inputValue(),'frontWheel');
    const renamed = await model(); assert.match(renamed,/frontWheel/);
    await snapshot('rename-commit');
    // The typed scalar has its own label, description, origin and mode control.
    await page.getByRole('combobox',{name:'Is abstract',exact:true}).focus();
    await expectFocus('scalar-isAbstract','scalar origin');
    const original = await page.locator('#scalar-isAbstract').inputValue();
    await press('End'); await press('ArrowUp');
    assert.equal(await page.locator('#scalar-isAbstract').inputValue(),'true');
    await press('Tab'); await expectFocus('scalar-mode-isAbstract','scalar mode');
    await press('Tab'); await expectFocus('stage-attribute-isAbstract','scalar stage control');
    await press('Enter'); await expectFocus('commit-staged','scalar staged');
    await press('Tab'); await press('Enter'); await expectFocus('scalar-isAbstract','scalar cancelled');
    assert.equal(await page.locator('#scalar-isAbstract').inputValue(),original); assert.equal(await model(),renamed);
    await press('End'); await press('ArrowUp'); await press('Tab'); await press('Tab'); await press('Enter');
    await expectFocus('commit-staged','scalar restaged'); await press('Enter');
    await expectFocus('scalar-isAbstract','scalar committed');
    const doc = JSON.parse(await model());
    const find = rows => rows.flatMap(n=>[n,...find(n.data.ownedRelationship||[]),...find(n.data.ownedRelatedElement||[])]);
    assert.equal(find(doc.content).find(n=>n.data.declaredName==='frontWheel').data.isAbstract,true);
    focused = await snapshot('scalar-commit');
    assert.equal(focused.role.value,'combobox'); assert.equal(focused.name.value,'Is abstract');
    const live = await page.evaluate(() => ({same:window.inspectorLive===document.getElementById('editor-status'),connected:window.inspectorLive.isConnected,
      removed:window.inspectorRegionRemoved,attrsCleared:window.inspectorAttrsCleared,announcements:window.inspectorAnnouncements}));
    assert.equal(live.same,true); assert.equal(live.connected,true); assert.equal(live.removed,false); assert.equal(live.attrsCleared,false);
    for (const message of [/staged/,/cancelled/,/committed/]) assert.ok(live.announcements.some(t=>message.test(t)));
    fs.writeFileSync(path.join(output,'live-region.json'),JSON.stringify(live,null,2));
    await page.evaluate(()=>window.scrollTo(0,0));
    await page.screenshot({path:path.join(output,'inspector-desktop.png'),fullPage:false});
    await page.locator('form[data-attribute="isAbstract"]').screenshot({path:path.join(output,'inspector-scalar.png')});
    await page.setViewportSize({width:390,height:844});
    await page.locator('#rename').focus(); await press('Tab'); await press('Enter');
    await expectFocus('commit-staged','mobile rename staged'); await press('Tab'); await press('Enter');
    await expectFocus('rename','mobile rename cancelled');
    await snapshot('mobile-cancel');
    await page.locator('#rename').scrollIntoViewIfNeeded();
    await page.screenshot({path:path.join(output,'inspector-mobile.png'),fullPage:false});
    assert.ok(await page.evaluate(()=>document.documentElement.scrollWidth<=document.documentElement.clientWidth));
    assert.deepEqual(errors,[]); assert.deepEqual(warnings,[]);
    fs.writeFileSync(path.join(output,'focus.json'),JSON.stringify(focus,null,2));
    fs.writeFileSync(path.join(output,'ax-summary.json'),JSON.stringify(ax,null,2));
    console.log(JSON.stringify({result:'PASS',title:await page.title(),url:page.url(),viewports:[[1365,1000],[390,844]],consoleErrors:errors,consoleWarnings:warnings,
      checks:['keyboard create/select/rename/scalar edit','tab order','stage/cancel/commit isolation','origin focus return','continuously connected live region','Chromium full AX tree labels/live/focus','mobile focus/no overflow'],
      assistiveScope:'Chromium accessibility tree; screen-reader speech and cross-platform acceptance NOT_RUN',screenshots:['inspector-desktop.png','inspector-scalar.png','inspector-mobile.png']}));
  } finally { await browser.close(); }
})().catch(error=>{console.error(error);process.exitCode=1});
