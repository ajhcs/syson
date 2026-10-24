// SPDX-License-Identifier: EPL-2.0
// Validation only. Serves frozen Rust/WASM artifacts on an ephemeral loopback port.
const assert = require('node:assert/strict');
const fs = require('node:fs');
const http = require('node:http');
const path = require('node:path');
const crypto = require('node:crypto');
const {chromium} = require(process.env.SYSON_PLAYWRIGHT_MODULE || 'playwright');
const candidate = process.env.SYSON_PREVIEW_DIR;
const standard = process.env.SYSON_DEFAULT_PREVIEW_DIR;
const output = process.env.SYSON_BROWSER_ARTIFACT_DIR;
if (!candidate || !standard || !output || !path.resolve(output).startsWith('/workspace/')) {
  throw new Error('Set candidate/default preview directories and a new artifact directory on /workspace.');
}
fs.mkdirSync(output, {recursive: false});
const types = {'index.html':'text/html','loader.js':'text/javascript','style.css':'text/css','syson_web.wasm':'application/wasm'};
const server = http.createServer((request, response) => {
  const url = new URL(request.url, 'http://localhost');
  const isDefault = url.pathname.startsWith('/default/');
  const name = url.pathname.replace(/^\/(default\/)?/, '') || 'index.html';
  if (!Object.hasOwn(types, name)) {response.writeHead(404); response.end(); return;}
  response.writeHead(200, {'Content-Type':types[name]});
  response.end(fs.readFileSync(path.join(isDefault ? standard : candidate, name)));
});
const nodes = document => {
  const found = [];
  function visit(node) {
    found.push(node);
    for (const feature of ['ownedRelationship','ownedRelatedElement']) {
      for (const child of node.data[feature] || []) visit(child);
    }
  }
  for (const root of document.content) visit(root);
  return found;
};
(async () => {
  await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
  const url = `http://127.0.0.1:${server.address().port}/`;
  const started = new Date().toISOString();
  const browser = await chromium.launch({headless:true});
  const errors = [];
  try {
    const page = await browser.newPage({viewport:{width:1365,height:1000}});
    page.on('pageerror', error => errors.push(error.message));
    page.on('console', message => {if (message.type()==='error') errors.push(message.text());});
    await page.goto(url);
    await page.getByRole('heading', {name:'SysON model editor',exact:true}).waitFor();
    assert.equal(await page.title(), 'SysON · Rust model editor');
    assert.equal(await page.locator('#kind option').count(), 7);
    const exported = () => page.locator('#export-json').inputValue();
    const committed = () => page.getByRole('button',{name:'Commit staged change',exact:true}).click();
    const stage = async (kind,name,parent='') => {
      await page.locator('#kind').selectOption(kind);
      await page.locator('#parent').selectOption(parent);
      await page.locator('#name').fill(name);
      await page.getByRole('button',{name:'Stage creation',exact:true}).click();
      assert.match(await page.getByRole('status').innerText(), /staged/);
    };
    await stage('package','Vehicle'); await committed();
    const parent = nodes(JSON.parse(await exported())).find(n => n.eClass==='sysml:Package').data.elementId;
    for (const [kind,name,className,children] of [
      ['connection_definition','Link','ConnectionDefinition',2],
      ['interface_definition','Signal','InterfaceDefinition',4],
      ['accept_action_usage','Receive','AcceptActionUsage',2],
      ['port_definition','IO','PortDefinition',1],
    ]) {
      const before = await exported();
      await stage(kind,name,parent);
      assert.equal(await exported(), before);
      await page.getByRole('button',{name:`${name} ${className}`,exact:true}).waitFor();
      await page.getByRole('button',{name:'Cancel staged change',exact:true}).click();
      assert.equal(await exported(), before);
      assert.equal(await page.getByRole('button',{name:`${name} ${className}`,exact:true}).count(), 0);
      await stage(kind,name,parent); await committed();
      const after = await exported();
      const root = nodes(JSON.parse(after)).find(n => n.eClass===`sysml:${className}` && n.data.declaredName===name);
      assert.equal(root.data.ownedRelationship.length, children);
      if (className==='InterfaceDefinition') {
        assert.deepEqual(root.data.ownedRelationship.map(m => m.data.ownedRelatedElement[0].eClass),
          ['sysml:PortUsage','sysml:PortUsage','sysml:ReferenceUsage','sysml:ReferenceUsage']);
      }
      if (className==='PortDefinition') {
        const conjugated = root.data.ownedRelationship[0].data.ownedRelatedElement[0];
        const reference = conjugated.data.ownedRelationship[0];
        assert.equal(reference.data.originalPortDefinition, root.data.elementId);
        assert.equal(reference.data.conjugatedType, conjugated.data.elementId);
      }
      await page.getByRole('button',{name:'Undo',exact:true}).click(); assert.equal(await exported(),before);
      await page.getByRole('button',{name:'Redo',exact:true}).click(); assert.equal(await exported(),after);
    }
    const model = await exported();
    fs.writeFileSync(path.join(output,'initialized-model.emf.json'), model);
    // Invalid parent goes through the real form and cannot change the committed model.
    await page.locator('#kind').selectOption('port_definition');
    await page.locator('#parent').selectOption('');
    await page.locator('#name').fill('Invalid');
    await page.getByRole('button',{name:'Stage creation',exact:true}).click();
    await page.getByRole('alert').waitFor(); assert.equal(await exported(),model);
    // Reload actual exported bytes through a fresh document, then confirm history resets.
    await page.getByText('Import another document',{exact:true}).click();
    await page.locator('#import-json').fill(model);
    await page.getByRole('button',{name:'Import as new document and clear history',exact:true}).click();
    assert.equal(await exported(),model);
    await page.getByRole('button',{name:'Undo',exact:true}).click(); assert.equal(await exported(),model);
    assert.match(await page.getByRole('status').innerText(), /No history/);
    await page.getByRole('button',{name:'IO PortDefinition',exact:true}).click();
    await page.evaluate(() => window.scrollTo(0,0));
    await page.screenshot({path:path.join(output,'desktop.png')});
    await page.setViewportSize({width:390,height:844});
    assert.equal(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth),true);
    await page.locator('#kind').selectOption('port_definition');
    await page.locator('#parent').selectOption(parent);
    await page.locator('form[data-action="stage"]').scrollIntoViewIfNeeded();
    await page.screenshot({path:path.join(output,'mobile.png')});
    // A fresh default-feature artifact must still advertise exactly the original three constructors.
    await page.goto(`${url}default/`);
    await page.getByRole('heading',{name:'SysON model editor',exact:true}).waitFor();
    assert.equal(await page.locator('#kind option').count(),3);
    assert.equal(await page.locator('#kind option[value="port_definition"]').count(),0);
    assert.deepEqual(errors,[]);
    const artifactHashes = {};
    for (const directory of [candidate,standard]) {
      for (const file of Object.keys(types)) {
        artifactHashes[`${directory===candidate?'candidate':'default'}/${file}`] = crypto.createHash('sha256').update(fs.readFileSync(path.join(directory,file))).digest('hex');
      }
    }
    const receipt = {result:'PASS',started_utc:started,finished_utc:new Date().toISOString(),
      browser_plugin:'absent; installed isolated Playwright Chromium used',
      browser_version:browser.version(), node_version:process.version,
      viewports:[[1365,1000],[390,844]], console_errors:errors, artifact_hashes:artifactHashes,
      checks:['identity/nonblank/no runtime overlay','all four native graphs','Interface inherited ends',
        'staging/export isolation','cancel','single undo/redo','Port local stable references',
        'invalid parent rollback','export/import/history reset','mobile horizontal overflow','default-off capability'],
      upstream_runtime:'NOT_RUN', parity_verified:false};
    fs.writeFileSync(path.join(output,'receipt.json'),JSON.stringify(receipt,null,2)+'\n');
    console.log(JSON.stringify(receipt));
  } finally {await browser.close(); server.close();}
})().catch(error => {console.error(error);server.close();process.exitCode=1;});
