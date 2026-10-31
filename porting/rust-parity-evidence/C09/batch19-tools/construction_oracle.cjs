// Development-only original-parser observation. This trusted-code VM is not a
// security sandbox. It deliberately evaluates no CLI entry or server startup.
// Usage: node construction_oracle.cjs ABS_BUNDLE ABS_CORPUS ABS_NEW_OUTPUT
'use strict';
const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');
const vm = require('node:vm');
const { builtinModules } = require('node:module');
const assert = require('node:assert/strict');
const sha = bytes => crypto.createHash('sha256').update(bytes).digest('hex');
const PINNED = '47d8fe11fda1b58b6257463087f663be953c2ab6c2b5e1e2ccc44161ed5cd0bd';
const BOUNDARY = Buffer.from('// src/index.ts\nvar src_exports = {};');
assert.equal(process.argv.length, 5, 'bundle, corpus and new output paths required');
const [bundlePath, corpusPath, outputPath] = process.argv.slice(2);
assert([bundlePath, corpusPath, outputPath].every(path.isAbsolute));
assert(outputPath.startsWith('/workspace/'), 'working output belongs on /workspace');
assert(!fs.existsSync(outputPath), 'refusing to replace an existing output');
const started = new Date().toISOString();
const bundle = fs.readFileSync(bundlePath);
const corpusBytes = fs.readFileSync(corpusPath);
assert.equal(sha(bundle), PINNED, 'pinned bundle drift');
const corpus = JSON.parse(corpusBytes.toString('utf8'));
assert.equal(corpus.schema, 1);
assert(Array.isArray(corpus.cases) && corpus.cases.length > 0);
const ids = new Set();
for (const item of corpus.cases) {
  assert(typeof item.id === 'string' && item.id.length && !ids.has(item.id));
  assert.equal(typeof item.source, 'string');
  assert.equal(Buffer.from(item.source, 'utf8').toString('utf8'), item.source, 'unpaired input surrogate');
  ids.add(item.id);
}
const stop = bundle.indexOf(BOUNDARY);
assert(stop > 0, 'missing CLI entry boundary');
assert.equal(bundle.indexOf(BOUNDARY, stop + 1), -1, 'ambiguous CLI entry boundary');
const prefix = bundle.subarray(0, stop);
const source = bundle.toString('utf8');
const headers = Array.from(source.matchAll(/^\/\/ (.+)$/gm));
const moduleHashes = {};
for (const [label, expected] of [
  ['../syside-languageserver/lib/sysml-module.js', '4dbbd7a940fd07186c3a8b9312b4fd3d616f03a5cebb204ce7be579398fe4186'],
  ['../syside-languageserver/lib/services/shared/workspace/file-system-provider.js', '9cb3100ef0a2db99d65741ce66f90f96295a29affa268a1fdc0dea362883e0e4'],
  ['../syside-languageserver/lib/services/parser/parser.js', 'ced8f9cab928606d13a323bcaca8693025c2ae381fd9ed2b0bcf27b45707991e'],
]) {
  const matches = headers.flatMap((header, i) => header[1] === label ? [i] : []);
  assert.equal(matches.length, 1, 'missing/ambiguous original module: ' + label);
  const i = matches[0];
  assert.equal(sha(source.slice(headers[i].index, headers[i + 1].index)), expected);
  moduleHashes[label] = expected;
}
const requestedBuiltins = new Set();
const builtins = new Set(builtinModules.map(name => name.replace(/^node:/, '')));
function builtinOnlyRequire(name) {
  assert(builtins.has(name.replace(/^node:/, '')), 'non-builtin import: ' + name);
  requestedBuiltins.add(name);
  return require(name);
}
const context = vm.createContext({
  require: builtinOnlyRequire, module: { exports: {} }, exports: {},
  process, console, Buffer, URL, URLSearchParams,
  setTimeout, clearTimeout, setImmediate, clearImmediate,
  corpusJson: JSON.stringify(corpus),
}, { name: 'pinned-syside-original-parser-current-runtime' });
new vm.Script(prefix.toString('utf8'), { filename: bundlePath + ':exact-prefix' })
  .runInContext(context, { timeout: 30000 });
const probe = String.raw`(() => {
  const filesystem = require_file_system_provider2();
  const services = require_sysml_module().createSysMLServices(filesystem.SysMLEmptyFileSystem);
  const provider = services.shared.workspace.FileSystemProvider;
  if (!(provider instanceof filesystem.SysMLEmptyFileSystemProvider)) throw Error('unexpected filesystem provider');
  if (provider.standardLibrary !== undefined || provider.extensionDir !== undefined) throw Error('unexpected filesystem roots');
  const parser = services.SysML.parser.LangiumParser;
  const keyName = key => typeof key === 'symbol' ? { symbol: key.description } : key;
  const keys = value => Reflect.ownKeys(value).map(keyName);
  const own = (value, key) => Object.getOwnPropertyDescriptor(value, key);
  function bits(value) {
    const bytes = new DataView(new ArrayBuffer(8)); bytes.setFloat64(0, value, false);
    return bytes.getUint32(0, false).toString(16).padStart(8, '0') +
      bytes.getUint32(4, false).toString(16).padStart(8, '0');
  }
  function scalar(value) {
    if (value === undefined) return { kind: 'undefined' };
    if (typeof value === 'number' && !Number.isFinite(value)) return { kind: 'number', bits: bits(value) };
    if (typeof value === 'number' && Object.is(value, -0)) return { kind: 'number', bits: bits(value) };
    if (typeof value === 'symbol') return { kind: 'symbol', description: value.description };
    if (typeof value === 'bigint') return { kind: 'bigint', decimal: value.toString() };
    if (typeof value === 'function') return { kind: 'function', name: value.name, arity: value.length };
    return value;
  }
  function exception(error) {
    return { name: error.name, message: error.message, stack: error.stack };
  }
  function graph(root, lexerErrors, parserErrors) {
    const objects = [], cst = [], seen = new WeakMap(), cstSeen = new WeakMap();
    function descriptor(d) {
      const result = { enumerable: d.enumerable, configurable: d.configurable };
      if ('value' in d) result.writable = d.writable;
      else { result.accessor = true; result.getter = Boolean(d.get); result.setter = Boolean(d.set); }
      return result;
    }
    function grammarContext(feature) {
      if (!feature || typeof feature !== 'object') return scalar(feature);
      // Grammar context is not recursively expanded into all grammar rules.
      // Every own key remains inventoried; AST data never takes this branch.
      return { kind: 'grammar_context', property_keys: keys(feature), properties:
        Reflect.ownKeys(feature).map(key => {
          const d = own(feature, key), row = { key: keyName(key), descriptor: descriptor(d) };
          if ('value' in d) row.value = d.value && typeof d.value === 'object'
            ? { kind: 'grammar_context_object', property_keys: keys(d.value) } : scalar(d.value);
          return row;
        }) };
    }
    function cstNode(node) {
      if (!node || typeof node !== 'object') return scalar(node);
      if (cstSeen.has(node)) return { $cst: cstSeen.get(node) };
      const id = 'c' + cst.length;
      cstSeen.set(node, id);
      const row = { id, property_keys_before_span_read: keys(node), span: {}, properties: [] };
      cst.push(row);
      // These original Langium CST getters only traverse CST/range caches.
      // Never read CST.element or AST reference-resolution getters.
      for (const key of ['offset', 'end', 'length', 'range', 'text', 'hidden']) {
        try { row.span[key] = encode(node[key]); }
        catch (error) { row.span[key] = { kind: 'getter_error', ...exception(error) }; }
      }
      row.property_keys = keys(node);
      for (const key of Reflect.ownKeys(node)) {
        const d = own(node, key), field = { key: keyName(key), descriptor: descriptor(d) };
        if ('value' in d) {
          if (key === 'feature') field.value = grammarContext(d.value);
          else if (key === 'parent' || key === 'root') field.value = cstNode(d.value);
          else if (key === 'children' && Array.isArray(d.value)) {
            field.value = { kind: 'cst_children', property_keys: keys(d.value),
              items: Array.from(d.value, child => cstNode(child)) };
            const parent = own(d.value, 'parent');
            if (parent && 'value' in parent) field.value.parent = cstNode(parent.value);
          } else field.value = encode(d.value);
        }
        row.properties.push(field);
      }
      return { $cst: id };
    }
    function encode(value) {
      if (value === null || typeof value !== 'object') return scalar(value);
      if (seen.has(value)) return { $object: seen.get(value) };
      const id = 'o' + objects.length;
      seen.set(value, id);
      const type = own(value, '$type'), reference = own(value, '$refText');
      const row = { id, kind: Array.isArray(value) ? 'array' : type && 'value' in type
        ? 'ast' : reference && 'value' in reference ? 'reference' : 'object',
        property_keys: keys(value), properties: [] };
      if (type && 'value' in type) row.node_type = scalar(type.value);
      objects.push(row);
      if (value instanceof RegExp) row.regexp = { source: value.source, flags: value.flags };
      for (const key of Reflect.ownKeys(value)) {
        const d = own(value, key), field = { key: keyName(key), descriptor: descriptor(d) };
        if ('value' in d) {
          if (key === '$meta') field.value = { kind: 'omitted_metadata', reason: 'external metamodel graph; key and descriptor retained' };
          else if (key === '$cstNode' || key === '$refNode') field.value = cstNode(d.value);
          else field.value = encode(d.value);
        }
        row.properties.push(field);
      }
      return { $object: id };
    }
    function diagnostics(errors) {
      return errors.map(error => {
        const scalarFields = {};
        for (const key of Object.getOwnPropertyNames(error)) {
          const d = own(error, key);
          if ('value' in d && (d.value === null || typeof d.value !== 'object')) scalarFields[key] = scalar(d.value);
        }
        return { property_keys: keys(error), scalar_fields: scalarFields, record: encode(error) };
      });
    }
    const rootRecord = encode(root);
    const lexer = diagnostics(lexerErrors), parser = diagnostics(parserErrors);
    return { tree: { root: rootRecord, objects, cst }, lexer_errors: lexer, parser_errors: parser };
  }
  const cases = JSON.parse(corpusJson).cases.map(item => {
    const row = { id: item.id, source: item.source };
    if (item.category) row.category = item.category;
    if (item.expected_restriction) row.expected_restriction = item.expected_restriction;
    try {
      const result = parser.parse(item.source);
      return { ...row, ...graph(result.value, result.lexerErrors, result.parserErrors) };
    } catch (error) {
      // A thrown parser/serialization failure is preserved, never disguised as
      // an empty successful AST or an original lexer/parser diagnostic.
      return { ...row, parse_exception: exception(error), lexer_errors: null, parser_errors: null, tree: null };
    }
  });
  return JSON.stringify({ cases, provider: { class: 'SysMLEmptyFileSystemProvider',
    standard_library: 'undefined', extension_directory: 'undefined', connection_supplied: false } });
})()`;
const captured = JSON.parse(new vm.Script(probe, { filename: 'construction-original-parser-probe' })
  .runInContext(context, { timeout: 60000 }));
assert.equal(captured.cases.length, corpus.cases.length);
const result = { cases: captured.cases, provenance: {
  qualification: 'PINNED_BUNDLE_CURRENT_RUNTIME_ORIGINAL_PARSER_OBSERVATION_ONLY',
  bundle_path: fs.realpathSync(bundlePath), bundle_sha256: sha(bundle), bundle_bytes: bundle.length,
  embedded_module_sha256: moduleHashes, evaluated_prefix_sha256: sha(prefix), evaluated_prefix_bytes: prefix.length,
  excluded_entry_boundary: BOUNDARY.toString(), boundary_byte_offset: stop,
  corpus_path: fs.realpathSync(corpusPath), corpus_sha256: sha(corpusBytes), corpus_cases: corpus.cases.length,
  oracle_sha256: sha(fs.readFileSync(__filename)), probe_sha256: sha(probe),
  source_baseline: '6ba2ac64e6b16ee2b05752ad55395eebb87d4910',
  source_candidate: '1326f234cae23b0e4777d5a48a069b17e9e332f1',
  node: process.version, v8: process.versions.v8, versions: process.versions,
  exec_path: process.execPath, real_exec_path: fs.realpathSync(process.execPath),
  executable_sha256: sha(fs.readFileSync(process.execPath)), platform: process.platform, arch: process.arch,
  requested_builtins: Array.from(requestedBuiltins).sort(), filesystem: captured.provider,
  started_utc: started, finished_utc: new Date().toISOString(),
  vm_timeout_ms: { prefix: 30000, services_parse_and_capture: 60000 },
  method: 'createSysMLServices(SysMLEmptyFileSystem).SysML.parser.LangiumParser.parse(source)',
  ast_metadata_skiplist: ['$meta'],
  serialization_policy: ['Every own AST property and descriptor is inventoried, including nonenumerable keys, undefined, false, empty arrays, $children and containers.',
    'Object identity/cycles/aliases use per-case $object/$cst references; IDs are capture-local, not persistent/native semantic IDs.',
    'Reference accessor descriptors are recorded without invoking ref, error or $nodeDescription; $refText and $refNode data are retained.',
    'CST span/text/range getters are read; inventories before/after the read preserve cache side effects. CST.element is never read.',
    'CST grammar features retain key/descriptor inventories and primitive values; nested grammar-context objects are inventoried without expanding the whole grammar.',
    'No property whitelist projects structural AST fields away; every original lexer/parser diagnostic gets a full own-data object record.'],
  io_policy: ['Original empty filesystem provider is supplied without a connection; standardLibrary and extensionDir must be absent.',
    'Original configuration updateStandardLibrary is a no-op on this provider; readFile/readFileSync/loadScript throw, readDirectory returns [], exists returns false.',
    'No server startup, workspace initialization, file loading, reference resolution, document building, semantic validation or package installation is invoked.',
    'Builtin-only require limits package imports; the VM runs trusted pinned code and is not an OS I/O or security sandbox.'],
  limits: ['Current Node/V8 is not bound to the deployed Docker Node20 runtime or whole native artifact.',
    'Parser initialization creates original service objects; parse-only observation does not qualify linking, metamodel construction or importer/UI workflows.',
    'Malformed/recovered native ASTs are preserved; they are not accepted models. Rust finite-family refusals remain separate outcomes.',
    'No cancellation/resource-capacity, performance, persistence or feature acceptance follows.'],
} };
const serialized = JSON.stringify(result, null, 2) + '\n';
fs.writeFileSync(outputPath, serialized, { encoding: 'utf8', flag: 'wx', mode: 0o600 });
console.log(JSON.stringify({ output_path: outputPath, output_sha256: sha(serialized), cases: result.cases.length,
  parse_exceptions: result.cases.filter(item => item.parse_exception).length, qualification: result.provenance.qualification }));
