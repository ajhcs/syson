const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { chromium } = require(process.env.SYSON_RESTART_PLAYWRIGHT || 'playwright');
const [phase, dir, origin] = process.argv.slice(2);
const base = new URL(origin);
assert.equal(base.hostname, '127.0.0.1');
const project = 'RestartDemo';
const diagram = 'Archive overview';
const documentation = 'Verified project restart documentation';
const addressFile = path.join(dir, 'journey.json');

async function mutate(page, action, route, name, payload, status) {
  const [reply] = await Promise.all([
    page.waitForResponse(response =>
      new URL(response.url()).pathname === route &&
      (route.endsWith('/upload') ? response.request().method() === 'POST' :
        response.request().postDataJSON()?.operationName === name)),
    action(),
  ]);
  assert.equal(reply.status(), status);
  const body = await reply.json();
  assert.ok(!body.errors?.length, 'GraphQL errors');
  assert.equal(body.data[name].__typename, payload);
  if (name === 'createProject') {
    const input = reply.request().postDataJSON().variables.input;
    assert.equal(input.name, project);
    assert.equal(input.templateId, 'blank-project');
  }
  return body.data[name];
}

async function setUp(page) {
  await page.goto(base.origin + '/new/project?templateId=blank-project', { waitUntil: 'domcontentloaded' });
  const name = page.getByTestId('name');
  await page.waitForFunction(() => document.querySelector('[data-testid="name"]')?.value === 'Blank Project');
  await name.click();
  await name.press('Control+a');
  await page.keyboard.insertText(project);
  assert.equal(await name.inputValue(), project);
  const created = await mutate(page, () => page.getByTestId('create-project').click(),
    '/api/graphql', 'createProject', 'CreateProjectSuccessPayload', 200);
  const projectId = created.project.id;
  await page.waitForURL(url => url.pathname === '/projects/' + projectId + '/edit');
  assert.ok((await page.getByTestId('navbar-title').innerText()).includes(project));
  await page.getByTestId('upload-document-icon').click();
  const dialog = page.getByRole('dialog');
  await dialog.locator('input[type=file]').setInputFiles({
    name: 'ArchiveDemoSource.sysml', mimeType: 'text/plain',
    buffer: Buffer.from('package ArchiveDemo {\n    part def Wheel;\n    part wheel : Wheel;\n}\n'),
  });
  const uploaded = await mutate(page, () => dialog.getByTestId('upload-document-split-button').click(),
    '/api/graphql/upload', 'uploadDocument', 'UploadDocumentSuccessPayload', 201);
  assert.equal(uploaded.report, '');
  await dialog.getByText('The document has been successfully uploaded').waitFor();
  await dialog.getByTestId('upload-document-close').click();
  const explorer = page.getByTestId('explorer://');
  await page.getByTestId('tree-filter-menu-icon').click();
  await page.getByTestId('tree-filter-menu-checkbox-Hide Root Namespaces').getByRole('checkbox').uncheck();
  await page.keyboard.press('Escape');
  const source = explorer.locator('[data-treeitemlabel^="ArchiveDemoSource"]').first();
  async function expand(row) {
    await row.waitFor();
    if (await row.getAttribute('data-expanded') === 'false')
      await explorer.getByTestId((await row.getAttribute('data-treeitemlabel')) + '-toggle').click();
  }
  await expand(source);
  const root = explorer.locator('[data-treeitemkind="siriusComponents://semantic?domain=sysml&entity=Namespace"]').first();
  await root.getByRole('button').click();
  await page.getByTestId('new-representation').click();
  await dialog.getByTestId('name').fill(diagram);
  const description = dialog.getByTestId('representationDescription');
  if (!(await description.innerText()).includes('General View')) {
    await description.click();
    await page.getByTestId('General View').click();
  }
  await dialog.getByTestId('create-representation').click();
  await page.getByTestId('representation-tab-' + diagram).waitFor();
  const canvas = page.locator('[data-representation-kind="diagram"][data-representation-label="Archive overview"]');
  await canvas.locator('.react-flow').waitFor();
  await expand(root);
  await explorer.getByTestId('ArchiveDemo-fullrow').dragTo(canvas.locator('.react-flow__pane'));
  await canvas.locator('.react-flow__node').filter({ hasText: 'ArchiveDemo' }).waitFor();
  const details = page.getByRole('region', { name: 'Details' });
  const label = details.getByTestId('input-Label');
  const field = details.getByTestId('input-Documentation');
  assert.equal(await label.inputValue(), diagram);
  assert.equal(await field.inputValue(), '');
  await mutate(page, async () => {
    await field.fill(documentation);
    await label.focus();
  }, '/api/graphql', 'editTextfield', 'SuccessPayload', 200);
  const url = new URL(page.url());
  assert.match(url.pathname, /^\/projects\/[^/]+\/edit\/[^/]+$/);
  assert.ok(url.searchParams.has('selection'));
  return { projectId, route: url.pathname + url.search };
}

async function read(page, saved) {
  assert.equal(new URL(page.url()).pathname, new URL(saved.route, base).pathname);
  await page.getByTestId('representation-tab-' + diagram).waitFor();
  const canvas = page.locator('[data-representation-kind="diagram"][data-representation-label="Archive overview"]');
  const node = canvas.locator('.react-flow__node').filter({ hasText: 'ArchiveDemo' });
  await node.waitFor();
  assert.equal(await node.count(), 1);
  const details = page.getByRole('region', { name: 'Details' });
  const label = await details.getByTestId('input-Label').inputValue();
  const text = await details.getByTestId('input-Documentation').inputValue();
  assert.equal(label, diagram);
  assert.equal(text, documentation);
  await page.screenshot({ path: path.join(dir, phase + '.png') });
  const dom = { title: await page.title(), diagram: label, documentation: text,
    renderedPackageNodes: await node.count(), detailsAria: await details.ariaSnapshot() };

  await page.getByTestId('more').click();
  const link = page.getByTestId('download-link');
  const href = new URL(await link.getAttribute('href'), page.url());
  assert.equal(href.origin, base.origin);
  assert.equal(href.pathname, '/api/projects/' + saved.projectId);
  const [response, file] = await Promise.all([
    page.waitForResponse(response => response.url() === href.href),
    page.waitForEvent('download'),
    link.click(),
  ]);
  assert.equal(response.status(), 200);
  assert.equal((await response.headerValue('content-type')).split(';')[0], 'application/zip');
  assert.equal(file.suggestedFilename(), project + '.zip');
  const archive = path.join(dir, phase + '.zip');
  await file.saveAs(archive);
  const download = { status: response.status(), filename: file.suggestedFilename(), bytes: fs.statSync(archive).size };
  fs.writeFileSync(path.join(dir, phase + '.json'), JSON.stringify({ dom, download }, null, 2) + '\n');
}

async function main() {
  const browser = await chromium.launch({ headless: true });
  const errors = [];
  try {
    const context = await browser.newContext({ acceptDownloads: true, viewport: { width: 1440, height: 960 }, locale: 'en-US' });
    const page = await context.newPage();
    page.setDefaultTimeout(15000);
    page.on('pageerror', error => errors.push({ type: 'pageerror', message: error.message }));
    page.on('console', message => {
      if (message.type() === 'error') errors.push({ type: 'console', message: message.text() });
    });
    let saved;
    if (phase === 'before') {
      saved = await setUp(page);
      fs.writeFileSync(addressFile, JSON.stringify(saved, null, 2) + '\n');
    } else {
      assert.equal(phase, 'after');
      saved = JSON.parse(fs.readFileSync(addressFile));
      assert.match(saved.route, /^\/projects\/[^/]+\/edit\/[^/]+\?/);
      await page.goto(base.origin + saved.route, { waitUntil: 'domcontentloaded' });
    }
    await read(page, saved);
    assert.deepEqual(errors.filter(error => error.type === 'pageerror'), []);
  } finally {
    fs.writeFileSync(path.join(dir, phase + '-browser-errors.json'), JSON.stringify(errors, null, 2) + '\n');
    await browser.close();
  }
}

main().catch(error => { console.error(error); process.exitCode = 1; });
