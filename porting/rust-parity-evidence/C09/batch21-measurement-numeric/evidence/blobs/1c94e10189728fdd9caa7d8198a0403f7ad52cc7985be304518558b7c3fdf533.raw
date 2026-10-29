// Developer-only replay of a supplied corpus through actual Rust WASM.
'use strict';
const fs=require('node:fs'),vm=require('node:vm'),crypto=require('node:crypto'),path=require('node:path');
if(process.argv.length!==6)throw Error('WASM, shim, corpus and new output required');
const workspace=fs.realpathSync('/workspace');
const within=p=>{const relative=path.relative(workspace,p);return relative!=='..'&&!relative.startsWith('..'+path.sep)&&!path.isAbsolute(relative);};
const resolveInput=p=>{if(!path.isAbsolute(p))throw Error('absolute input required');const actual=fs.realpathSync(p);if(!within(actual)||!fs.statSync(actual).isFile())throw Error('workspace input required');return actual;};
const [wasm,shim,corpus]=process.argv.slice(2,5).map(resolveInput);
const requested=process.argv[5];if(!path.isAbsolute(requested))throw Error('absolute output required');
const parent=fs.realpathSync(path.dirname(requested));if(!within(parent))throw Error('workspace output parent required');
const out=path.join(parent,path.basename(requested));
try {fs.lstatSync(out);throw Error('new output leaf required');} catch(e){if(e.code!=='ENOENT')throw e;}
const sha=b=>crypto.createHash('sha256').update(b).digest('hex');
(async()=>{
 const binary=fs.readFileSync(wasm),source=fs.readFileSync(shim),raw=fs.readFileSync(corpus);
 vm.runInThisContext(source.toString(),{filename:shim});
 const {instance}=await WebAssembly.instantiate(binary);
 const input=JSON.parse(raw),before=JSON.stringify(input);
 const rows=input.cases.map(c=>runMeasurement(instance,c));
 if(before!==JSON.stringify(input))throw Error('caller inputs changed');
 const provenance={wasm_sha256:sha(binary),shim_sha256:sha(source),corpus_sha256:sha(raw),
  runner_sha256:sha(fs.readFileSync(__filename)),node:process.version,v8:process.versions.v8,
  scope:'Actual Rust WASM typed host boundary on supplied facts; no DOM/application qualification.'};
 fs.writeFileSync(out,JSON.stringify({rows,provenance},null,2)+'\n',{flag:'wx'});
 console.log(JSON.stringify({rows:rows.length,out}));
})().catch(e=>{console.error(e);process.exitCode=1;});
