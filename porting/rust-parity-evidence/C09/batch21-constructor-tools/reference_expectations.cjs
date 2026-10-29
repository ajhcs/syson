// Original-method development oracle. Trusted pinned code, not a security sandbox.
// Usage: node reference_expectations.cjs ABS_BUNDLE ABS_CORPUS ABS_NEW_OUTPUT
'use strict';
const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');
const crypto = require('node:crypto');
const { builtinModules } = require('node:module');
const assert = require('node:assert/strict');
const sha = bytes => crypto.createHash('sha256').update(bytes).digest('hex');
const PINNED = '47d8fe11fda1b58b6257463087f663be953c2ab6c2b5e1e2ccc44161ed5cd0bd';
const BOUNDARY = Buffer.from('// src/index.ts\nvar src_exports = {};');
assert.equal(process.argv.length, 5);
const [bundlePath, corpusPath, requestedOutputPath] = process.argv.slice(2);
assert([bundlePath, corpusPath, requestedOutputPath].every(path.isAbsolute));
const workspace = fs.realpathSync('/workspace');
const outputParent = fs.realpathSync(path.dirname(requestedOutputPath));
assert(outputParent === workspace || outputParent.startsWith(workspace + path.sep));
const outputPath = path.join(outputParent, path.basename(requestedOutputPath));
assert(!fs.existsSync(outputPath));
const started = new Date().toISOString();
const bundle = fs.readFileSync(bundlePath), corpusBytes = fs.readFileSync(corpusPath);
assert.equal(sha(bundle), PINNED);
assert.equal(sha(corpusBytes), 'a4fb319e8b30b3c50504bdcc42f38b35acee7ae179550874e66bf113c35d05f4');
const corpus = JSON.parse(corpusBytes.toString('utf8'));
assert.equal(corpus.schema, 1);
assert(Array.isArray(corpus.cases) && corpus.cases.length === 42);
const ids = new Set(), counts = { finite: 0, malformed: 0, 'outside-whitelist': 0 };
for (const item of corpus.cases) {
  assert(typeof item.id === 'string' && item.id.length && !ids.has(item.id));
  assert.equal(typeof item.source, 'string');
  assert(Object.hasOwn(counts, item.category));
  const restriction = { malformed: 'Malformed', 'outside-whitelist': 'Unsupported' }[item.category];
  assert.equal(item.expected_restriction, restriction);
  counts[item.category]++;
  assert.equal(Buffer.from(item.source, 'utf8').toString('utf8'), item.source);
  ids.add(item.id);
}
assert.deepEqual(counts, { finite: 22, malformed: 8, 'outside-whitelist': 12 });
const stop = bundle.indexOf(BOUNDARY);
assert(stop > 0 && bundle.indexOf(BOUNDARY, stop + 1) === -1);
const prefix = bundle.subarray(0, stop), source = bundle.toString('utf8');
const headers = Array.from(source.matchAll(/^\/\/ (.+)$/gm)), moduleHashes = {};
for (const [label, expected] of [
  ['../syside-languageserver/lib/services/sysml-ast-reflection.js', 'feba10097b686ba078add908e48f4543bb6a5353f4bb7ae69b9f03867f2fd382'],
  ['../../node_modules/.pnpm/langium@1.2.1/node_modules/langium/lib/utils/grammar-util.js', 'ea3ba01b690c9fa19df6fd9321ecba911bb5eab3feeb1ba757b453c94134531c'],
  ['../syside-languageserver/lib/sysml-module.js', '4dbbd7a940fd07186c3a8b9312b4fd3d616f03a5cebb204ce7be579398fe4186'],
]) {
  const matches = headers.flatMap((h, i) => h[1] === label ? [i] : []);
  assert.equal(matches.length, 1);
  const i = matches[0];
  assert.equal(sha(source.slice(headers[i].index, headers[i + 1].index)), expected);
  moduleHashes[label] = expected;
}
const requestedBuiltins = new Set();
const builtins = new Set(builtinModules.map(name => name.replace(/^node:/, '')));
function builtinOnlyRequire(name) {
  assert(builtins.has(name.replace(/^node:/, '')));
  requestedBuiltins.add(name);
  return require(name);
}
const context = vm.createContext({ require: builtinOnlyRequire, module: { exports: {} }, exports: {},
  process, console, Buffer, URL, URLSearchParams, setTimeout, clearTimeout, setImmediate, clearImmediate,
  corpusJson: JSON.stringify(corpus) }, { name: 'original-parser-reference-expectations' });
new vm.Script(prefix.toString('utf8'), { filename: bundlePath + ':exact-prefix' })
  .runInContext(context, { timeout: 30000 });
const probe = String.raw`(() => {
  const filesystem = require_file_system_provider2();
  const services = require_sysml_module().createSysMLServices(filesystem.SysMLEmptyFileSystem);
  const provider = services.shared.workspace.FileSystemProvider;
  if (!(provider instanceof filesystem.SysMLEmptyFileSystemProvider)) throw Error('unexpected provider');
  if (provider.standardLibrary !== undefined || provider.extensionDir !== undefined) throw Error('unexpected filesystem roots');
  const parser = services.SysML.parser.LangiumParser, reflection = services.shared.AstReflection;
  const streamReferences = require_lib2().streamReferences;
  const findNodesForProperty = require_grammar_util().findNodesForProperty;
  const own = (value, key) => Object.getOwnPropertyDescriptor(value, key);
  const data = (value, key) => { const d = own(value, key); return d && 'value' in d ? d.value : undefined; };
  const ast = value => value && typeof value === 'object' && typeof data(value, '$type') === 'string';
  const escape = key => key.replaceAll('~', '~0').replaceAll('/', '~1');
  const error = e => ({ name: e.name, message: e.message, stack: e.stack });
  function span(cst) {
    if (!cst) return null;
    return { offset: cst.offset, end: cst.end, length: cst.length, text: cst.text, range: cst.range };
  }
  const cases = JSON.parse(corpusJson).cases.map(item => {
    const row = { id: item.id, source: item.source, references: [], property_spans: [], metadata: [], aliases: [] };
    try {
      const parsed = parser.parse(item.source);
      row.lexer_error_count = parsed.lexerErrors.length; row.parser_error_count = parsed.parserErrors.length;
      const paths = new WeakMap(), nodes = [];
      function visit(value, path) {
        if (Array.isArray(value)) { value.forEach((v, i) => visit(v, path + '/' + i)); return; }
        if (!ast(value)) return;
        if (paths.has(value)) { row.aliases.push({ path, canonical: paths.get(value) }); return; }
        paths.set(value, path); nodes.push(value);
        for (const key of Object.getOwnPropertyNames(value).sort()) {
          const d = own(value, key);
          if (!key.startsWith('$') && 'value' in d) visit(d.value, path + '/' + escape(key));
        }
      }
      visit(parsed.value, 'root');
      for (const node of nodes) {
        const owner_path = paths.get(node), cst = data(node, '$cstNode');
        const index = own(node, '$childIndex');
        row.metadata.push({ path: owner_path, node_type: data(node, '$type'), own_keys: Object.getOwnPropertyNames(node),
          mandatory: reflection.getTypeMetaData(data(node, '$type')).mandatory,
          child_index: index ? { present: true, descriptor_kind: 'value' in index ? 'data' : 'accessor',
            value: 'value' in index ? (index.value === undefined ? { kind: 'undefined' } : index.value)
              : { kind: 'accessor' } } : { present: false } });
        for (const property of Object.getOwnPropertyNames(node).filter(k => !k.startsWith('$') || k === '$children').sort()) {
          const entry = { owner_path, property };
          try { entry.nodes = findNodesForProperty(cst, property).map(span); }
          catch (e) { entry.error = error(e); }
          row.property_spans.push(entry);
        }
        for (const info of streamReferences(node)) {
          const entry = { owner_path, property: info.property, index: info.index === undefined ? null : info.index,
            ref_text: data(info.reference, '$refText'), own_keys: Object.getOwnPropertyNames(info.reference) };
          try { entry.target_type = reflection.getReferenceType(info); }
          catch (e) { entry.target_error = error(e); }
          try { entry.cst = span(data(info.reference, '$refNode')); }
          catch (e) { entry.cst_error = error(e); }
          row.references.push(entry);
        }
      }
    } catch (e) { row.capture_error = error(e); }
    return row;
  });
  return JSON.stringify({ cases });
})()`;
const captured = JSON.parse(new vm.Script(probe, { filename: 'original-reference-and-property-cst-probe' })
  .runInContext(context, { timeout: 60000 }));
const result = { cases: captured.cases, provenance: {
  qualification: 'PINNED_BUNDLE_CURRENT_RUNTIME_ORIGINAL_METHOD_EXPECTATIONS_ONLY',
  bundle_sha256: sha(bundle), bundle_path: fs.realpathSync(bundlePath), embedded_module_sha256: moduleHashes,
  evaluated_prefix_sha256: sha(prefix), evaluated_prefix_bytes: prefix.length, excluded_entry_boundary: BOUNDARY.toString(),
  corpus_sha256: sha(corpusBytes), corpus_path: fs.realpathSync(corpusPath),
  oracle_sha256: sha(fs.readFileSync(__filename)), probe_sha256: sha(probe),
  node: process.version, v8: process.versions.v8, executable_sha256: sha(fs.readFileSync(process.execPath)),
  exec_path: process.execPath, versions: process.versions, requested_builtins: [...requestedBuiltins].sort(),
  started_utc: started, finished_utc: new Date().toISOString(),
  calls: ['createSysMLServices(SysMLEmptyFileSystem).SysML.parser.LangiumParser.parse(source)',
    'original streamReferences(node), custom services.shared.AstReflection.getReferenceType(realReferenceInfo)',
    'original AstReflection.getTypeMetaData(node.$type)', 'original findNodesForProperty(node.$cstNode, property)'],
  accessor_policy: 'No AST reference ref/error/$nodeDescription getter is read. The original property-CST helper reads CST.element associations and grammar containers; safe CST span getters are read.',
  limits: ['Builtin-only trusted VM, not an OS I/O/security sandbox; original empty filesystem, no server/workspace/link/validation invocation.',
    'Structural-path identities are case-local; cache arrays do not mint containment paths. Reference targets are original method results, never guessed from Rust.',
    'Current Node/V8 does not bind deployed Docker Node20, the whole native artifact, semantic resolution, cancellation/resource bounds or feature acceptance.'],
} };
assert.equal(result.cases.length, corpus.cases.length);
const serialized = JSON.stringify(result, null, 2) + '\n';
fs.writeFileSync(outputPath, serialized, { encoding: 'utf8', flag: 'wx', mode: 0o600 });
console.log(JSON.stringify({ output_path: outputPath, output_sha256: sha(serialized), cases: result.cases.length,
  capture_errors: result.cases.filter(c => c.capture_error).length }));
