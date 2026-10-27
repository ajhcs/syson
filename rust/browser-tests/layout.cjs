// SPDX-License-Identifier: EPL-2.0
// Actual WASM canvas journey; no native SysON/runtime-oracle claim.
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
    const page = await browser.newPage({serviceWorkers:'block', viewport:{width:1365,height:1000}});
    const errors = [], warnings = [];
    page.on('pageerror', e => errors.push(e.message));
    page.on('console', m => { if (m.type() === 'error') errors.push(m.text()); if (m.type() === 'warning') warnings.push(m.text()); });
    await page.route('**/*', route => {
      if (new URL(route.request().url()).origin === origin) return route.continue();
      errors.push('External request: ' + route.request().url()); return route.abort();
    });
    await page.goto(url);
    await page.getByRole('heading', {name:'SysON model editor',exact:true}).waitFor();
    assert.equal(await page.title(),'SysON · Rust model editor');
    await page.evaluate(() => document.fonts.ready);
    const button = name => page.getByRole('button', {name,exact:true});
    const model = () => page.locator('#export-json').inputValue();
    const text = () => page.locator('#export-geometry').inputValue();
    const geometry = async () => JSON.parse(await text());
    const phase = () => page.locator('[data-canvas-phase]').getAttribute('data-canvas-phase');
    const create = async (kind,name,parent='',definition='') => {
      await page.locator('#kind').selectOption(kind); await page.locator('#parent').selectOption(parent);
      await page.locator('#definition').selectOption(definition); await page.locator('#name').fill(name);
      await button('Stage creation').click(); await button('Commit staged change').click();
    };
    await create('package','Vehicle');
    const parent = await page.locator('#parent option').filter({hasText:'Vehicle'}).getAttribute('value');
    await create('part_definition','Wheel',parent);
    const definition = await page.locator('#definition option').filter({hasText:'Wheel'}).getAttribute('value');
    await create('part_usage','wheel',parent,definition);
    await button('wheel PartUsage').click();
    const committed = await model();
    await page.clock.install({time:new Date('2026-10-25T12:00:00Z')});
    await page.clock.pauseAt(new Date('2026-10-25T12:00:01Z'));
    await page.locator('#name').fill('draft before layout');
    await button('Layout canvas').click();
    assert.equal(await phase(),'compute');
    assert.equal(await page.locator('#name').inputValue(),'draft before layout');
    await page.locator('#rename').fill('unsaved wheel draft');
    if (!await page.locator('#geometry-details').evaluate(e=>e.open))
      await page.locator('#geometry-details summary').click();
    await page.locator('#import-geometry').fill('unsaved JSON draft');
    await page.locator('#rename').focus();
    await page.locator('#rename').evaluate(e => e.setSelectionRange(3,9,'backward'));
    await page.clock.runFor(50);
    assert.equal(await page.locator('[data-canvas-phase]').count(),0);
    assert.equal(await page.locator('#name').inputValue(),'draft before layout');
    assert.equal(await page.locator('#rename').inputValue(),'unsaved wheel draft');
    assert.equal(await page.locator('#import-geometry').inputValue(),'unsaved JSON draft');
    assert.equal(await page.locator('#geometry-details').evaluate(e=>e.open),true);
    assert.deepEqual(await page.locator('#rename').evaluate(e=>[document.activeElement.id,e.selectionStart,e.selectionEnd,e.selectionDirection]),['rename',3,9,'backward']);
    assert.equal(await model(),committed);
    const originalText = await text(), original = await geometry();
    assert.equal(original.nodes.length,3); assert.equal(original.edges.length,1);
    assert.equal(await page.locator('[data-canvas-edge]').count(),1);
    assert.equal(original.nodes.find(n=>n.id===parent).header,44);
    // Cancel before computation and invalidate a queued publication by selection.
    await button('Layout canvas').click(); await button('Cancel canvas work').click();
    await page.clock.runFor(50); assert.equal(await text(),originalText);
    await button('Layout canvas').click();
    await page.clock.runFor(0); // Runs compute; the subsequent zero-delay callback is a later tick.
    assert.equal(await phase(),'apply');
    await button('Wheel PartDefinition').click();
    await page.clock.runFor(50); assert.equal(await text(),originalText);
    await button('Layout canvas').click();
    // Simulated font-generation event validates the event→ticket-cancel binding.
    await page.evaluate(()=>document.fonts.dispatchEvent(new Event('loading')));
    await page.clock.runFor(50); assert.equal(await text(),originalText);
    await button('wheel PartUsage').click();
    await button('Rotate preview').click(); await page.clock.runFor(50);
    const rotatedText = await text(), rotated = await geometry();
    const usage = rotated.nodes.find(n=>n.id!==parent && n.id!==definition);
    assert.deepEqual([usage.width,usage.height],[70,180]);
    for (const n of original.nodes.filter(n=>n.id!==usage.id)) assert.deepEqual(rotated.nodes.find(x=>x.id===n.id),n);
    assert.equal(rotated.edges[0].id,original.edges[0].id);
    assert.notDeepEqual(rotated.edges[0].points,original.edges[0].points);
    assert.equal(await model(),committed);
    await button('Canvas undo').click(); assert.equal(await text(),originalText);
    await button('Canvas redo').click(); assert.equal(await text(),rotatedText);
    await button('Layout canvas').click(); await page.clock.runFor(50);
    const full = await geometry(), box = full.nodes.find(n=>n.id===parent);
    for (const n of full.nodes.filter(n=>n.parent===parent)) {
      assert.ok(n.x>=box.x && n.y>=box.y+box.header && n.x+n.width<=box.x+box.width && n.y+n.height<=box.y+box.height);
    }
    // Invalid import is atomic even while a valid computation is queued.
    const beforeImport = await text();
    await button('Layout canvas').click();
    if (!await page.locator('#geometry-details').evaluate(e=>e.open))
      await page.locator('#geometry-details summary').click();
    await page.locator('#import-geometry').fill('{bad');
    await button('Import canvas geometry').click();
    assert.equal(await text(),beforeImport); assert.equal(await phase(),'compute');
    await page.clock.runFor(50);
    const foreign = await geometry(); foreign.model += ' ';
    if (!await page.locator('#geometry-details').evaluate(e=>e.open))
      await page.locator('#geometry-details summary').click();
    await page.locator('#import-geometry').fill(JSON.stringify(foreign));
    await button('Import canvas geometry').click(); assert.equal(await text(),beforeImport);
    // Structurally valid file data may contain outside/negative routes; fit all points.
    const imported = await geometry(), edge = imported.edges[0];
    const first = edge.points[0], last = edge.points.at(-1);
    edge.points = [first,[first[0],-100],[-200,-100],[-200,last[1]],last];
    if (!await page.locator('#geometry-details').evaluate(e=>e.open))
      await page.locator('#geometry-details summary').click();
    await page.locator('#import-geometry').fill(JSON.stringify(imported));
    await button('Import canvas geometry').click();
    assert.match(await page.getByRole('status').innerText(),/imported/);
    const restored = await geometry(); assert.deepEqual(restored,imported);
    const view = await page.locator('.canvas svg').getAttribute('viewBox');
    const [x,y,w,h] = view.split(' ').map(Number);
    for (const [px,py] of edge.points) assert.ok(px>=x && px<=x+w && py>=y && py<=y+h);
    assert.ok(x<=-224 && y<=-124); assert.equal(await model(),committed);
    await button('Canvas undo').click(); assert.equal(await text(),beforeImport);
    await button('Canvas redo').click(); assert.deepEqual(await geometry(),imported);
    // A staged semantic change cancels queued canvas work without changing model bytes.
    await button('Layout canvas').click(); await page.locator('#rename').fill('staged wheel');
    await button('Stage rename').click(); await page.clock.runFor(50);
    assert.equal(await page.locator('[data-canvas-phase]').count(),0); assert.equal(await model(),committed);
    await button('Cancel staged change').click(); assert.deepEqual(await geometry(),imported);
    // Importing identical model bytes still invalidates the previous generation/history.
    await page.getByText('Import another document',{exact:true}).click();
    await page.locator('#import-json').fill(committed); await button('Import as new document and clear history').click();
    assert.equal(await text(),''); await button('Canvas undo').click(); assert.equal(await text(),'');
    await button('Layout canvas').click(); await page.clock.runFor(50);
    await page.screenshot({path:path.join(output,'layout-desktop.png'),fullPage:true});
    await page.setViewportSize({width:390,height:844});
    await page.screenshot({path:path.join(output,'layout-mobile.png'),fullPage:true});
    const size = await page.evaluate(()=>[document.documentElement.scrollWidth,document.documentElement.clientWidth]);
    assert.ok(size[0]<=size[1]); assert.deepEqual(errors,[]); assert.deepEqual(warnings,[]);
    fs.writeFileSync(path.join(output,'canvas-geometry.json'),await text());
    console.log(JSON.stringify({result:'PASS',title:await page.title(),url:page.url(),viewports:[[1365,1000],[390,844]],checks:['real DOM font measurement','cooperative publication','draft/focus/caret/details preservation','cancel/stale selection','simulated font-generation event invalidation','first-leaf rotation and incident edge','canvas undo/redo','full-layout bounds','atomic import','negative route viewport','staged-model invalidation','same-model import generation','mobile no overflow'],screenshots:['layout-desktop.png','layout-mobile.png'],consoleErrors:errors,consoleWarnings:warnings}));
  } finally { await browser.close(); }
})().catch(error=>{console.error(error);process.exitCode=1});
