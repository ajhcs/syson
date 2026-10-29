// Development-only original dispatch-method observation; no callbacks execute.
// This trusted-byte VM has real builtins/process objects, not security isolation.
// Usage: node dispatch_oracle.cjs ABS_BUNDLE ABS_CORPUS ABS_NEW_OUTPUT
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
assert(outputPath.startsWith('/workspace/'), 'output belongs on Samsung workspace');
assert(!fs.existsSync(outputPath), 'output must be a new explicitly owned file');
const started = new Date().toISOString();
const bundle = fs.readFileSync(bundlePath), corpusBytes = fs.readFileSync(corpusPath);
assert.equal(sha(bundle), PINNED, 'pinned original bundle drift');
const corpus = JSON.parse(corpusBytes.toString('utf8'));
assert.equal(corpus.schema, 1);
assert(Array.isArray(corpus.cases) && corpus.cases.length);
const ids = new Set();
for (const item of corpus.cases) {
  assert(typeof item.id === 'string' && item.id && !ids.has(item.id));
  ids.add(item.id);
  assert(['TypesIndex', 'SysMLCstNodeBuilder'].includes(item.constructor));
  assert(['all', 'lookup'].includes(item.mode));
  assert(Array.isArray(item.entries) && typeof item.reverse === 'boolean');
  assert(['subtype-first', 'supertype-first'].includes(item.order));
  assert(Object.hasOwn(item, 'default') && Object.hasOwn(item, 'input'));
}
const stop = bundle.indexOf(BOUNDARY);
assert(stop > 0 && bundle.indexOf(BOUNDARY, stop + 1) === -1, 'CLI boundary drift');
const prefix = bundle.subarray(0, stop), source = bundle.toString('utf8');
const headers = Array.from(source.matchAll(/^\/\/ (.+)$/gm)), moduleHashes = {};
for (const [label, expected] of [
  ['../syside-languageserver/lib/model/types.js', 'afd39d9ebedb6ff1a6cca2d4f42341d5883c9509e81f3b89981f1cc0bea79440'],
  ['../syside-languageserver/lib/services/parser/parser.js', 'ced8f9cab928606d13a323bcaca8693025c2ae381fd9ed2b0bcf27b45707991e'],
  ['../../node_modules/.pnpm/langium@1.2.1/node_modules/langium/lib/parser/cst-node-builder.js', '5515bf68b0f3d79a26feebbef259c513879c6043aff06ae0e866885564dd3b4a'],
]) {
  const matches = headers.flatMap((h, i) => h[1] === label ? [i] : []);
  assert.equal(matches.length, 1, 'module marker missing or ambiguous: ' + label);
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
}, { name: 'pinned-original-dispatch-current-runtime' });
new vm.Script(prefix.toString('utf8'), { filename: bundlePath + ':exact-prefix' })
  .runInContext(context, { timeout: 30000 });
const probe = String.raw`(() => {
  const index = require_types2().typeIndex;
  const generated = new Set(index.base.getAllTypes());
  // The original constructor stores this argument without reading it. Only its
  // original map construction runs; construct(item) and callbacks never run.
  const builder = new (require_parser3().SysMLCstNodeBuilder)(undefined);
  if (index.constructor.name !== 'TypesIndex' || builder.constructor.name !== 'SysMLCstNodeBuilder')
    throw Error('original constructor identity drift');
  function number(bits) {
    if (!/^[0-9a-f]{16}$/.test(bits)) throw Error('invalid number bits');
    const view = new DataView(new ArrayBuffer(8));
    view.setUint32(0, parseInt(bits.slice(0, 8), 16), false);
    view.setUint32(4, parseInt(bits.slice(8), 16), false);
    return view.getFloat64(0, false);
  }
  function decode(d) {
    switch (d.kind) {
      case 'undefined': return undefined;
      case 'null': return null;
      case 'boolean': if (typeof d.value !== 'boolean') throw Error('boolean descriptor'); return d.value;
      case 'number': return number(d.bits);
      case 'string': if (typeof d.value !== 'string') throw Error('string descriptor'); return d.value;
      case 'array': if (!Array.isArray(d.items)) throw Error('array descriptor'); return d.items.map(decode);
      case 'symbol': return Symbol(d.description);
      default: throw Error('unsupported descriptor');
    }
  }
  function encode(value) {
    if (value === undefined) return {kind:'undefined'};
    if (value === null) return {kind:'null'};
    if (typeof value === 'boolean') return {kind:'boolean',value};
    if (typeof value === 'string') return {kind:'string',value};
    if (typeof value === 'number') {
      const view = new DataView(new ArrayBuffer(8)); view.setFloat64(0,value,false);
      const hex = n => view.getUint32(n,false).toString(16).padStart(8,'0');
      return {kind:'number',bits:hex(0)+hex(4)};
    }
    if (typeof value === 'function') return {kind:'function',name:value.name,arity:value.length};
    if (Array.isArray(value)) return {kind:'array',items:value.map(encode)};
    if (value instanceof Map) return {kind:'map',entries:Array.from(value,([key,item])=>[key,encode(item)])};
    throw Error('unhandled original return kind');
  }
  const observations = [];
  for (const item of JSON.parse(corpusJson).cases) {
    const registry = Object.create(null), names = new Set();
    for (const entry of item.entries) {
      if (!generated.has(entry.name) || names.has(entry.name)) throw Error('unadmitted registry key');
      names.add(entry.name); registry[entry.name] = decode(entry.value);
    }
    registry.default = decode(item.default);
    let map;
    if (item.constructor === 'SysMLCstNodeBuilder') {
      if (item.method !== 'postprocessingMap' || item.entries.length || item.default.kind !== 'undefined')
        throw Error('pinned registration case must use original map');
      map = builder.postprocessingMap;
    } else if (item.method === 'expandToDerivedTypes') map = index.expandToDerivedTypes(registry);
    else if (item.method === 'expandAndMerge') map = index.expandAndMerge(registry,item.reverse);
    else if (item.method === 'chain') map = index.chain(registry,item.order);
    else throw Error('unadmitted original method');
    observations.push({id:item.id,value:encode(item.mode === 'all' ? map : map.get(decode(item.input)))});
  }
  return JSON.stringify({observations,native_constructors:[index.constructor.name,builder.constructor.name]});
})()`;
const capture = JSON.parse(new vm.Script(probe, { filename: 'original-dispatch-method-probe' })
  .runInContext(context, { timeout: 30000 }));
const result = {schema:1,...capture,provenance:{
  qualification:'PINNED_BUNDLE_CURRENT_RUNTIME_DISPATCH_OBSERVATION_ONLY',
  bundle_path:fs.realpathSync(bundlePath),bundle_sha256:sha(bundle),bundle_bytes:bundle.length,
  evaluated_prefix_sha256:sha(prefix),evaluated_prefix_bytes:prefix.length,
  excluded_entry_boundary:BOUNDARY.toString(),embedded_module_sha256:moduleHashes,
  corpus_path:fs.realpathSync(corpusPath),corpus_sha256:sha(corpusBytes),corpus_cases:corpus.cases.length,
  oracle_sha256:sha(fs.readFileSync(__filename)),probe_sha256:sha(probe),
  node:process.version,v8:process.versions.v8,versions:process.versions,
  exec_path:fs.realpathSync(process.execPath),executable_sha256:sha(fs.readFileSync(process.execPath)),
  platform:process.platform,arch:process.arch,requested_builtins:Array.from(requestedBuiltins).sort(),
  started_utc:started,finished_utc:new Date().toISOString(),vm_timeout_ms_per_script:30000,
  calls:['original typeIndex.expandToDerivedTypes/expandAndMerge/chain',
    'original SysMLCstNodeBuilder(undefined) constructor and its postprocessingMap',
    'Map.get selection only; no construct(item), callback, parser or server execution'],
  limits:['Trusted pinned-byte VM has real process/builtin objects and is not a security sandbox.',
    'Registry inputs are own-property null-prototype objects in the admitted typed domain; inherited properties and dynamic mutation are unqualified.',
    'Function descriptors are native observations, not Rust function/callback equivalence. Bind selection identities separately.',
    'Current Node/V8 does not bind configured native Node20 or whole source/artifact/runtime closure.',
    'No F12 callback application, AST/cache mutation, real parser/importer/UI, performance or feature acceptance follows.'],
}};
const serialized = JSON.stringify(result,null,2)+'\n';
fs.writeFileSync(outputPath,serialized,{encoding:'utf8',flag:'wx',mode:0o600});
console.log(JSON.stringify({output_path:outputPath,output_sha256:sha(serialized),cases:capture.observations.length,
  qualification:result.provenance.qualification}));
