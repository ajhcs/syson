// Development evidence only; never a shipped application dependency.
// Usage: node reflection_oracle.cjs ABSOLUTE_PINNED_BUNDLE ABSOLUTE_NEW_OUTPUT
// Reads the exact retained bundle; evaluates its unmodified module-definition
// prefix, stopping before the CLI entry. Original methods execute in one VM.
'use strict';
const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');
const vm = require('node:vm');
const { builtinModules } = require('node:module');
const assert = require('node:assert/strict');
const PINNED = '47d8fe11fda1b58b6257463087f663be953c2ab6c2b5e1e2ccc44161ed5cd0bd';
const BOUNDARY = Buffer.from('// src/index.ts\nvar src_exports = {};');
const sha = bytes => crypto.createHash('sha256').update(bytes).digest('hex');
assert.equal(process.argv.length, 4, 'two explicit paths are required');
const [bundlePath, outputPath] = process.argv.slice(2);
assert(path.isAbsolute(bundlePath) && path.isAbsolute(outputPath), 'absolute paths required');
assert(!fs.existsSync(outputPath), 'output must be a new, explicitly owned file');
assert.notEqual(fs.realpathSync(bundlePath), path.resolve(outputPath));
const started = new Date().toISOString();
const bundle = fs.readFileSync(bundlePath);
assert.equal(sha(bundle), PINNED, 'pinned bundle bytes drifted');
const stop = bundle.indexOf(BOUNDARY);
assert(stop > 0, 'CLI entry boundary missing');
assert.equal(bundle.indexOf(BOUNDARY, stop + 1), -1, 'CLI entry boundary is not unique');
const prefix = bundle.subarray(0, stop);
const sourceText = bundle.toString('utf8');
const headers = Array.from(sourceText.matchAll(/^\/\/ (.+)$/gm));
const embeddedModuleHashes = {};
for (const [label, expected] of [
  ['../../node_modules/.pnpm/langium@1.2.1/node_modules/langium/lib/syntax-tree.js', 'ba5342f44ed73ec6636e8ce98db3ac5bf37b74af75da17b088196e012e344f4f'],
  ['../syside-languageserver/lib/generated/ast.js', '2cbb9199c741eb4f8a67904cce2ae58f1eab21f7c0c5d8ac15f40a1d4da6868d'],
  ['../syside-languageserver/lib/model/types.js', 'afd39d9ebedb6ff1a6cca2d4f42341d5883c9509e81f3b89981f1cc0bea79440'],
  ['../syside-languageserver/lib/services/sysml-ast-reflection.js', 'feba10097b686ba078add908e48f4543bb6a5353f4bb7ae69b9f03867f2fd382'],
]) {
  const matches = headers.flatMap((header, i) => header[1] === label ? [i] : []);
  assert.equal(matches.length, 1, `module marker missing/ambiguous: ${label}`);
  const i = matches[0];
  const text = sourceText.slice(headers[i].index, headers[i + 1].index);
  assert.equal(sha(text), expected, `embedded module drift: ${label}`);
  embeddedModuleHashes[label] = expected;
}
const requestedBuiltins = new Set();
const builtins = new Set(builtinModules.map(name => name.replace(/^node:/, '')));
// No external packages, network operations, subprocesses or module writes are
// requested by this harness. The original barrel modules import Node builtins;
// importing those modules is separate from invoking their I/O operations.
function builtinOnlyRequire(name) {
  assert(builtins.has(name.replace(/^node:/, '')), `non-builtin import: ${name}`);
  requestedBuiltins.add(name);
  return require(name);
}
const context = vm.createContext({
  require: builtinOnlyRequire, module: { exports: {} }, exports: {},
  process, console, Buffer, URL, URLSearchParams,
  setTimeout, clearTimeout, setImmediate, clearImmediate,
}, { name: 'pinned-syside-reflection-current-runtime' });
const initializationStart = process.hrtime.bigint();
new vm.Script(prefix.toString('utf8'), { filename: bundlePath + ':exact-prefix' })
  .runInContext(context, { timeout: 30000 });

// This probe contains calls and serialization only. It neither replaces the
// sort comparator nor reconstructs inheritance from retained Rust tables.
const probe = String.raw`(() => {
  const ast = require_ast2();
  const base = new ast.SysMlAstReflection();
  const index = require_types2().typeIndex;
  const reflection = new (require_sysml_ast_reflection().SysMLAstReflection)();
  const names = base.getAllTypes();
  if (names.length !== 189 || new Set(names).size !== 189) throw Error('type inventory drift');
  const local = new Map(names.map(name => [name, base.getTypeMetaData(name)]));
  function outcome(action) {
    try {
      const value = action();
      if (value === undefined) return { status: 'return', value_kind: 'undefined' };
      return { status: 'return', value };
    } catch (error) {
      return { status: 'throw', name: error.name, message: error.message };
    }
  }
  const rows = names.map(name => {
    const ancestors = Array.from(index.getInheritanceChain(name));
    const metadata = reflection.getTypeMetaData(name);
    const mandatory = metadata.mandatory.map(property => ({
      name: property.name, kind: property.type,
      // Rust-only declared_by is reconstructed from original LOCAL metadata
      // in the original effective chain, not asserted as an upstream field.
      declared_by: property.name === '$children' ? null : [name, ...ancestors]
        .find(owner => local.get(owner).mandatory.some(p => p.name === property.name)),
    }));
    return {
      name, ancestors, descendants: Array.from(index.getSubtypes(name)),
      local_mandatory: local.get(name).mandatory,
      metadata, mandatory,
      subtype_row: names.map(supertype => index.isSubtype(name, supertype)),
      base_subtype_row: names.map(supertype => base.isSubtype(name, supertype)),
      metadata_cache_identity: reflection.getTypeMetaData(name) === metadata,
    };
  });
  const referenceNames = [
    'ClassifierReference', 'ConjugatedPortReference', 'ElementReference',
    'FeatureReference', 'MembershipReference', 'MetaclassReference',
    'NamespaceReference', 'TypeReference',
  ];
  const generated_references = referenceNames.map(name => ({
    property_id: name + ':parts',
    result: outcome(() => base.getReferenceType({ container: { $type: name }, property: 'parts' })),
  }));
  const positions = [];
  for (const name of [...referenceNames, 'Package']) {
    for (let length = 1; length <= 4; ++length) {
      for (let position = 0; position < length; ++position) {
        const container = { $type: name, parts: new Array(length) };
        positions.push({ container: name, index: position, length,
          result: outcome(() => reflection.getReferenceType({ container, property: 'parts', index: position })) });
      }
    }
  }
  const unsupported_positions = [];
  for (const [label, position, length] of [
    ['empty-zero', 0, 0], ['empty-minus-one', -1, 0], ['negative', -1, 2],
    ['past-end', 2, 2], ['fractional', 0.5, 2], ['missing-index', undefined, 2],
    ['string-last', '1', 2], ['nan', NaN, 2], ['infinite', Infinity, 2],
  ]) {
    unsupported_positions.push({ label, container: 'TypeReference', length,
      result: outcome(() => reflection.getReferenceType({ container: { $type: 'TypeReference', parts: new Array(length) }, property: 'parts', index: position })) });
  }
  unsupported_positions.push({ label: 'ignored-property',
    result: outcome(() => reflection.getReferenceType({ container: { $type: 'TypeReference', parts: [null] }, property: 'other', index: 0 })) });
  unsupported_positions.push({ label: 'missing-parts',
    result: outcome(() => reflection.getReferenceType({ container: { $type: 'TypeReference' }, property: 'parts', index: 0 })) });
  unsupported_positions.push({ label: 'null-container',
    result: outcome(() => reflection.getReferenceType({ container: null, property: 'parts', index: 0 })) });
  const unknown_names = ['', 'NotAType', '__proto__', 'Type\0', 'type'].map(name => ({
    name,
    subtype_self: outcome(() => index.isSubtype(name, name)),
    subtype_element: outcome(() => index.isSubtype(name, 'Element')),
    ancestors: outcome(() => Array.from(index.getInheritanceChain(name))),
    descendants: outcome(() => Array.from(index.getSubtypes(name))),
    generated_metadata: outcome(() => base.getTypeMetaData(name)),
    effective_metadata: outcome(() => reflection.getTypeMetaData(name)),
  }));
  const invalid_generated_references = ['', 'TypeReference:other', 'Package:parts', '__proto__:parts'].map(id => {
    const separator = id.indexOf(':');
    const name = separator < 0 ? id : id.slice(0, separator);
    const property = separator < 0 ? '' : id.slice(separator + 1);
    return { property_id: id,
      result: outcome(() => base.getReferenceType({ container: { $type: name }, property })) };
  });
  return JSON.stringify({
    all_generated_types: names, ast_types_filtered: index.getAllTypes(), rows,
    generated_references, positions, unsupported_positions, unknown_names,
    invalid_generated_references,
  });
})()`;
const captureStart = process.hrtime.bigint();
const observation = JSON.parse(new vm.Script(probe, { filename: 'reflection-oracle-probe' })
  .runInContext(context, { timeout: 30000 }));
const captureEnd = process.hrtime.bigint();
const result = {
  qualification: 'PINNED_BUNDLE_CURRENT_RUNTIME_OBSERVATION_ONLY',
  limits: [
    'No binding to the actual native Docker Node20.15.1 binary/V8, process or whole deployed artifact.',
    'Candidate TypeScript and embedded bundle are separately retained provenance; no whole-source equivalence is inferred.',
    'The byte-identical prefix contains original module definitions; the CLI entry is never evaluated.',
    'No AST creation/default assignment/linker/parser/project workflow executes.',
    'Unknown names and malformed dynamic positions document differences outside the strict owner-qualified Rust domain.',
    'Initialization/capture duration is diagnostic setup cost, not native/Rust product performance evidence.',
  ],
  source: {
    bundle_path: fs.realpathSync(bundlePath), bundle_sha256: sha(bundle), bundle_bytes: bundle.length,
    baseline: '6ba2ac64e6b16ee2b05752ad55395eebb87d4910',
    candidate: '1326f234cae23b0e4777d5a48a069b17e9e332f1',
    candidate_sources: {
      'generated/ast.ts': '982a5b2415abbc4ad8f29cab3facbf5fbe29a9dc7cc7bde42a58ad0849de9c02',
      'model/types.ts': '61c221ebe23bee6f61757a72f515ce2aeafb111a8bbf3cc6fce9c3a9415c4f3c',
      'services/sysml-ast-reflection.ts': '9f43aba6051f1ef68b36730212e5ebdbb906816fac746a644be7cd6e89c9607b',
    },
    evaluated_prefix_sha256: sha(prefix), evaluated_prefix_bytes: prefix.length,
    embedded_module_source_sha256: embeddedModuleHashes,
    excluded_entry_boundary: BOUNDARY.toString(), boundary_byte_offset: stop,
    probe_sha256: sha(probe), oracle_sha256: sha(fs.readFileSync(__filename)),
  },
  runtime: {
    node: process.version, v8: process.versions.v8, versions: process.versions,
    exec_path: process.execPath, real_exec_path: fs.realpathSync(process.execPath),
    executable_sha256: sha(fs.readFileSync(process.execPath)),
    platform: process.platform, arch: process.arch,
    requested_builtins: Array.from(requestedBuiltins).sort(),
  },
  calls: [
    'new require_ast2().SysMlAstReflection().getAllTypes()/isSubtype()/getTypeMetaData()/getReferenceType()',
    'require_types2().typeIndex.getAllTypes()/getInheritanceChain()/getSubtypes()/isSubtype()',
    'new require_sysml_ast_reflection().SysMLAstReflection().getTypeMetaData()/getReferenceType()',
  ],
  execution: {
    started_utc: started, finished_utc: new Date().toISOString(),
    prefix_initialization_ns: (captureStart - initializationStart).toString(),
    observation_ns: (captureEnd - captureStart).toString(),
    vm_timeout_ms_per_script: 30000,
  },
  observation,
};
const serialized = JSON.stringify(result, null, 2) + '\n';
fs.writeFileSync(outputPath, serialized, { encoding: 'utf8', flag: 'wx', mode: 0o600 });
console.log(JSON.stringify({ output_path: outputPath, output_sha256: sha(serialized),
  rows: observation.rows.length, positions: observation.positions.length,
  qualification: result.qualification }));
