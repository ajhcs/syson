// Development-only original-method oracle; never a shipped application path.
// Usage: node values_oracle.cjs ABSOLUTE_BUNDLE ABSOLUTE_CORPUS ABSOLUTE_NEW_OUTPUT
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
assert([bundlePath, corpusPath, outputPath].every(path.isAbsolute), 'absolute paths required');
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
  assert(typeof item.id === 'string' && item.id.length && !ids.has(item.id), 'invalid/duplicate case id');
  ids.add(item.id);
  assert(['convert', 'sanitize', 'unsanitize'].includes(item.op), 'unsupported operation');
  if (Object.hasOwn(item, 'text')) {
    assert(typeof item.text === 'string', 'text must be a Unicode string');
    assert.equal(Buffer.from(item.text, 'utf8').toString('utf8'), item.text, 'input text has an unpaired surrogate');
  } else {
    assert.equal(item.op, 'sanitize', 'only sanitize(undefined) may omit text');
  }
  if (item.op === 'convert') {
    assert(['raw', 'unresolved', 'rule'].includes(item.dispatch), 'invalid dispatch');
    if (item.dispatch === 'rule') assert(typeof item.rule === 'string', 'missing rule name');
  }
  if (Object.hasOwn(item, 'declared_type')) assert.equal(typeof item.declared_type, 'string');
  if (Object.hasOwn(item, 'expected_restriction')) assert.equal(typeof item.expected_restriction, 'string');
}
const stop = bundle.indexOf(BOUNDARY);
assert(stop > 0, 'CLI entry boundary missing');
assert.equal(bundle.indexOf(BOUNDARY, stop + 1), -1, 'CLI boundary is ambiguous');
const prefix = bundle.subarray(0, stop);
const source = bundle.toString('utf8');
const headers = Array.from(source.matchAll(/^\/\/ (.+)$/gm));
const moduleHashes = {};
for (const [label, expected] of [
  ['../../node_modules/.pnpm/langium@1.2.1/node_modules/langium/lib/parser/value-converter.js', '5233f8c326d0d5722519cd2652dced4260883046411d2764af68f2f75da475be'],
  ['../syside-languageserver/lib/model/naming.js', '741e39b574fdaa774b276cc566dca1f2fca38309b3f483bb912fe175d638fa44'],
]) {
  const matches = headers.flatMap((header, i) => header[1] === label ? [i] : []);
  assert.equal(matches.length, 1, 'module marker missing/ambiguous: ' + label);
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
// Original barrel imports may import Node I/O modules. This harness invokes no
// network, subprocess, parser, package manager or original CLI entry method.
const context = vm.createContext({
  require: builtinOnlyRequire, module: { exports: {} }, exports: {},
  process, console, Buffer, URL, URLSearchParams,
  setTimeout, clearTimeout, setImmediate, clearImmediate,
  corpusJson: JSON.stringify(corpus),
}, { name: 'pinned-syside-values-current-runtime' });
new vm.Script(prefix.toString('utf8'), { filename: bundlePath + ':exact-prefix' })
  .runInContext(context, { timeout: 30000 });
const probe = String.raw`(() => {
  const converter = new (require_value_converter().DefaultValueConverter)();
  const naming = require_naming();
  function bits(number) {
    const bytes = new DataView(new ArrayBuffer(8));
    bytes.setFloat64(0, number, false);
    return bytes.getUint32(0, false).toString(16).padStart(8, '0') +
      bytes.getUint32(4, false).toString(16).padStart(8, '0');
  }
  function encoded(value) {
    if (value === undefined) return { kind: 'undefined' };
    if (typeof value === 'string') return { kind: 'string',
      units: Array.from({ length: value.length }, (_, i) => value.charCodeAt(i)) };
    if (typeof value === 'number') return { kind: 'number', bits: bits(value) };
    if (typeof value === 'boolean') return { kind: 'boolean', value };
    // Only explicitly restricted cases admit these native-only value kinds.
    if (typeof value === 'bigint') return { kind: 'bigint', decimal: value.toString() };
    if (value instanceof Date) return { kind: 'date', time_bits: bits(value.getTime()),
      valid: !Number.isNaN(value.getTime()) };
    throw Error('unhandled native return type: ' + typeof value);
  }
  function invoke(item) {
    if (item.op === 'sanitize') return naming.sanitizeName(item.text);
    if (item.op === 'unsanitize') return naming.unsanitizeName(item.text);
    let feature;
    if (item.dispatch === 'raw') feature = { $type: 'Keyword' };
    else if (item.dispatch === 'unresolved') feature = { $type: 'RuleCall', rule: {} };
    else feature = { $type: 'RuleCall', rule: { ref: {
      $type: 'TerminalRule', name: item.rule,
      type: item.declared_type === undefined ? undefined : { name: item.declared_type },
    } } };
    return converter.convert(item.text, { feature });
  }
  const observations = [], restrictions = [];
  for (const item of JSON.parse(corpusJson).cases) {
    let value;
    try { value = encoded(invoke(item)); }
    catch (error) { value = { kind: 'error', name: error.name, message: error.message }; }
    if (item.expected_restriction) restrictions.push({ id: item.id,
      expected_restriction: item.expected_restriction, native_value: value });
    else observations.push({ id: item.id, value });
  }
  return JSON.stringify({ observations, restrictions });
})()`;
const capture = JSON.parse(new vm.Script(probe, { filename: 'values-original-method-probe' })
  .runInContext(context, { timeout: 30000 }));
assert(capture.observations.length > 0);
assert.equal(capture.observations.length + capture.restrictions.length, corpus.cases.length);
const result = {
  ...capture,
  provenance: {
    qualification: 'PINNED_BUNDLE_CURRENT_RUNTIME_OBSERVATION_ONLY',
    source_baseline: '6ba2ac64e6b16ee2b05752ad55395eebb87d4910',
    source_candidate: '1326f234cae23b0e4777d5a48a069b17e9e332f1',
    candidate_naming_sha256: '6eb7e99dca2d6f00b7a7b577be6ab63d569fc18cf05c2ad171c203d56a037dd0',
    bundle_path: fs.realpathSync(bundlePath), bundle_sha256: sha(bundle), bundle_bytes: bundle.length,
    embedded_module_sha256: moduleHashes,
    evaluated_prefix_sha256: sha(prefix), evaluated_prefix_bytes: prefix.length,
    excluded_entry_boundary: BOUNDARY.toString(), boundary_byte_offset: stop,
    corpus_path: fs.realpathSync(corpusPath), corpus_sha256: sha(corpusBytes),
    corpus_cases: corpus.cases.length, oracle_sha256: sha(fs.readFileSync(__filename)),
    probe_sha256: sha(probe), node: process.version, v8: process.versions.v8,
    versions: process.versions, exec_path: process.execPath,
    real_exec_path: fs.realpathSync(process.execPath), executable_sha256: sha(fs.readFileSync(process.execPath)),
    platform: process.platform, arch: process.arch,
    requested_builtins: Array.from(requestedBuiltins).sort(),
    started_utc: started, finished_utc: new Date().toISOString(), vm_timeout_ms_per_script: 30000,
    calls: ['DefaultValueConverter.convert(input, synthetic resolved TerminalRule/Keyword/RuleCall CST feature)',
      'require_naming().sanitizeName(input)', 'require_naming().unsanitizeName(input)'],
    numeric_policy: 'Every comparable Number is encoded by DataView as 16 lowercase big-endian hex digits; signed zero, NaN signs/payloads and infinities are not normalized.',
    limits: ['Original methods execute with supplied dispatch descriptors; no real parser, CST resolution or whole importer is qualified.',
      'The unmodified prefix excludes the CLI entry; current Node/V8 is not bound to the native Docker runtime or deployed artifact.',
      'Candidate source provenance and embedded module provenance remain distinct.',
      'Rust presence, byte spans, budgets/cancellation and explicit unsupported domains require separate checks.',
      'No native application, durable lifecycle, resource capacity, performance or feature acceptance follows.'],
  },
};
const serialized = JSON.stringify(result, null, 2) + '\n';
fs.writeFileSync(outputPath, serialized, { encoding: 'utf8', flag: 'wx', mode: 0o600 });
console.log(JSON.stringify({ output_path: outputPath, output_sha256: sha(serialized),
  observations: capture.observations.length, restrictions: capture.restrictions.length,
  qualification: result.provenance.qualification }));
