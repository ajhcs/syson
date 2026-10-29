// Development-only original registered callback on named synthetic AST nodes.
// Real builtin/process objects are available: this VM is not a security sandbox.
'use strict';
const fs=require('node:fs'),path=require('node:path'),vm=require('node:vm'),crypto=require('node:crypto');
const {builtinModules}=require('node:module'),assert=require('node:assert/strict');
const sha=bytes=>crypto.createHash('sha256').update(bytes).digest('hex');
const BUNDLE='47d8fe11fda1b58b6257463087f663be953c2ab6c2b5e1e2ccc44161ed5cd0bd';
const CORPUS='24cda4f62b2368cdc48cd9c2c976ba5c67ba295160e4bdc9483f9dc1d886632a';
const BOUNDARY=Buffer.from('// src/index.ts\nvar src_exports = {};');
assert.equal(process.argv.length,5,'bundle, corpus and new output required');
const workspace=fs.realpathSync('/workspace');
const within=p=>{const r=path.relative(workspace,p);return r!=='..'&&!r.startsWith('..'+path.sep)&&!path.isAbsolute(r);};
function input(p){assert(path.isAbsolute(p));const actual=fs.realpathSync(p);assert(within(actual)&&fs.statSync(actual).isFile());return actual;}
const bundlePath=input(process.argv[2]),corpusPath=input(process.argv[3]),requested=process.argv[4];
assert(path.isAbsolute(requested));const parent=fs.realpathSync(path.dirname(requested));assert(within(parent));
const outputPath=path.join(parent,path.basename(requested));
try{fs.lstatSync(outputPath);throw Error('output must be new');}catch(e){if(e.code!=='ENOENT')throw e;}
const started=new Date().toISOString(),bundle=fs.readFileSync(bundlePath),corpusBytes=fs.readFileSync(corpusPath);
assert.equal(sha(bundle),BUNDLE);assert.equal(sha(corpusBytes),CORPUS);
const corpus=JSON.parse(corpusBytes);assert.equal(corpus.schema,1);assert.equal(corpus.cases.length,42);
const ids=new Set();for(const c of corpus.cases){assert(typeof c.id==='string'&&c.id&&!ids.has(c.id));ids.add(c.id);assert(['compatible','rust-restricted','rust-unrepresentable'].includes(c.domain));assert(c.targets.length);}
const stop=bundle.indexOf(BOUNDARY);assert(stop>0&&bundle.indexOf(BOUNDARY,stop+1)===-1);
const prefix=bundle.subarray(0,stop),source=bundle.toString('utf8'),headers=[...source.matchAll(/^\/\/ (.+)$/gm)],modules={};
for(const [label,hash] of [
 ['../syside-languageserver/lib/services/parser/parser.js','ced8f9cab928606d13a323bcaca8693025c2ae381fd9ed2b0bcf27b45707991e'],
 ['../syside-languageserver/lib/services/sysml-ast-reflection.js','feba10097b686ba078add908e48f4543bb6a5353f4bb7ae69b9f03867f2fd382'],
 ['../syside-languageserver/lib/sysml-module.js','4dbbd7a940fd07186c3a8b9312b4fd3d616f03a5cebb204ce7be579398fe4186'],
 ['../syside-languageserver/lib/services/shared/workspace/file-system-provider.js','9cb3100ef0a2db99d65741ce66f90f96295a29affa268a1fdc0dea362883e0e4']]){
 const found=headers.flatMap((h,i)=>h[1]===label?[i]:[]);assert.equal(found.length,1);const i=found[0];assert.equal(sha(source.slice(headers[i].index,headers[i+1].index)),hash);modules[label]=hash;
}
const requestedBuiltins=new Set(),builtins=new Set(builtinModules.map(n=>n.replace(/^node:/,'')));
const builtinRequire=name=>{assert(builtins.has(name.replace(/^node:/,'')));requestedBuiltins.add(name);return require(name);};
const context=vm.createContext({require:builtinRequire,module:{exports:{}},exports:{},process,console,Buffer,URL,URLSearchParams,setTimeout,clearTimeout,setImmediate,clearImmediate,corpusJson:JSON.stringify(corpus)},{name:'pinned-original-synthetic-operator-callback'});
new vm.Script(prefix.toString('utf8'),{filename:bundlePath+':exact-prefix'}).runInContext(context,{timeout:30000});
const probe=String.raw`(()=>{
 const filesystem=require_file_system_provider2(),services=require_sysml_module().createSysMLServices(filesystem.SysMLEmptyFileSystem);
 const provider=services.shared.workspace.FileSystemProvider,reflection=services.shared.AstReflection;
 if(!(provider instanceof filesystem.SysMLEmptyFileSystemProvider)||provider.standardLibrary!==undefined||provider.extensionDir!==undefined)throw Error('original empty filesystem required');
 if(services.SysML.shared.AstReflection!==reflection)throw Error('callback and setup must share the actual original reflection');
 const builder=new(require_parser3().SysMLCstNodeBuilder)(services.SysML),original=builder.postprocessingMap.get('OperatorExpression');
 if(typeof original!=='function'||original.name!=='fixOperatorExpression'||original.length!==2)throw Error('original registered handler drift');
 const own=(o,k)=>Object.getOwnPropertyDescriptor(o,k),data=(o,k)=>{const d=own(o,k);return d&&'value'in d?d.value:undefined;};
 const ast=o=>o&&typeof o==='object'&&typeof data(o,'$type')==='string';
 function bits(n){const v=new DataView(new ArrayBuffer(8));v.setFloat64(0,n,false);return v.getUint32(0,false).toString(16).padStart(8,'0')+v.getUint32(4,false).toString(16).padStart(8,'0');}
 function number(hex){if(!/^[0-9a-f]{16}$/.test(hex))throw Error('invalid number bits');const v=new DataView(new ArrayBuffer(8));v.setUint32(0,parseInt(hex.slice(0,8),16),false);v.setUint32(4,parseInt(hex.slice(8),16),false);return v.getFloat64(0,false);}
 function scalar(v){
  if(v===undefined)return{kind:'undefined'};if(v===null)return{kind:'null'};
  if(typeof v==='number')return{kind:'number',bits:bits(v)};
  if(typeof v==='boolean'||typeof v==='string')return{kind:typeof v,value:v};
  if(typeof v==='function')return{kind:'function',name:v.name,arity:v.length};
  if(typeof v==='symbol')return{kind:'symbol',description:v.description};
  if(typeof v==='bigint')return{kind:'bigint',decimal:String(v)};
  throw Error('unsupported scalar');
 }
 function decode(d){
  switch(d.kind){case'undefined':return undefined;case'null':return null;case'boolean':if(typeof d.value!=='boolean')throw Error('boolean');return d.value;
   case'string':if(typeof d.value!=='string')throw Error('string');return d.value;case'number':return number(d.bits);
   case'node':return make(d);case'array':return d.items.map(decode);default:throw Error('unknown descriptor');}
 }
 let setupCalls=[];
 function make(spec){
  const bare=spec.fields.operands?.kind==='absent',setup=bare?'bare_synthetic_fields':'defaulted_creation';
  const node=bare?{$type:spec.node_type}:reflection.createNode(spec.node_type,{});setupCalls.push({node_type:spec.node_type,setup});
  for(const [key,d]of Object.entries(spec.fields)){
   if(d.kind==='absent'){delete node[key];continue;}
   if(d.kind==='array'){
    node[key]=[];
    for(const child of d.items){const value=decode(child);if(ast(value))reflection.assignNode(value,{$container:node,$containerProperty:key,$containerIndex:node[key].length});else node[key].push(value);}
   }else{const value=decode(d);if(ast(value))reflection.assignNode(value,{$container:node,$containerProperty:key});else node[key]=value;}
  }
  return node;
 }
 function locate(root,path){if(path==='root')return root;if(!path.startsWith('root/'))throw Error('target path');let value=root;for(const key of path.slice(5).split('/'))value=data(value,key);if(!ast(value))throw Error('target not AST');return value;}
 const cases=[];
 for(const spec of JSON.parse(corpusJson).cases){
  setupCalls=[];const row={id:spec.id,domain:spec.domain,steps:[],setup_calls:setupCalls},identities=new WeakMap();let next=0;
  function graph(root){
   const objects=[],seen=new WeakSet();
   function encode(value){
    if(value===null||typeof value!=='object')return scalar(value);
    if(!identities.has(value))identities.set(value,'o'+next++);const id=identities.get(value);
    if(seen.has(value))return{$object:id};seen.add(value);
    const keys=Reflect.ownKeys(value),record={id,kind:Array.isArray(value)?'array':ast(value)?'ast':'object',property_keys:keys.map(k=>typeof k==='symbol'?{symbol:k.description}:k),properties:[]};
    if(ast(value))record.node_type=data(value,'$type');objects.push(record);
    for(const key of keys){const d=own(value,key),field={key:typeof key==='symbol'?{symbol:key.description}:key,descriptor:{enumerable:d.enumerable,configurable:d.configurable}};
     if('value'in d){field.descriptor.writable=d.writable;field.value=encode(d.value);}else{field.descriptor.accessor=true;field.descriptor.getter=Boolean(d.get);field.descriptor.setter=Boolean(d.set);}
     record.properties.push(field);
    }
    return{$object:id};
   }
   return{root:encode(root),objects};
  }
  try{
   const root=make(spec.root);row.before=graph(root);
   for(const path of spec.targets){const target=locate(root,path),selected=builder.postprocessingMap.get(data(target,'$type'));
    const step={target_path:path,selection:scalar(selected),invoked:selected===original,before:graph(root)};
    if(step.invoked){try{step.native_return=scalar(selected.call(undefined,target,services.SysML));}catch(e){step.native_error={name:e.name,message:e.message,stack:e.stack};}}
    else step.native_return={kind:'not_invoked'};
    step.after=graph(root);row.steps.push(step);
   }
   row.after_handler=graph(root);
   if(spec.domain==='compatible'){
    const seen=new Set(),calls=[];
    function finish(value){
     if(Array.isArray(value)){value.forEach(finish);return;}if(!ast(value)||seen.has(value))return;seen.add(value);
     reflection.assignMandatoryProperties(value);calls.push({node_type:data(value,'$type')});
     for(const key of Object.getOwnPropertyNames(value))if(!key.startsWith('$'))finish(data(value,key));
    }
    finish(root);row.finalization={kind:'original_assignMandatoryProperties',calls};
   }else row.finalization={kind:'not_run_after_Rust_domain_refusal'};
   row.final=graph(root);
  }catch(e){row.capture_error={name:e.name,message:e.message,stack:e.stack};}
  cases.push(row);
 }
 return JSON.stringify({cases,provider:{class:provider.constructor.name,standard_library:{kind:'undefined'},extension_directory:{kind:'undefined'},connection_supplied:false},handler:{name:original.name,arity:original.length}});
})()`;
const capture=JSON.parse(new vm.Script(probe,{filename:'original-registered-operator-callback-probe'}).runInContext(context,{timeout:60000}));
const result={schema:1,...capture,provenance:{bundle_sha256:sha(bundle),bundle_path:bundlePath,embedded_module_sha256:modules,evaluated_prefix_sha256:sha(prefix),evaluated_prefix_bytes:prefix.length,excluded_entry_boundary:BOUNDARY.toString(),corpus_sha256:sha(corpusBytes),corpus_path:corpusPath,oracle_sha256:sha(fs.readFileSync(__filename)),probe_sha256:sha(probe),node:process.version,v8:process.versions.v8,exec_path:fs.realpathSync(process.execPath),executable_sha256:sha(fs.readFileSync(process.execPath)),requested_builtins:[...requestedBuiltins].sort(),started_utc:started,finished_utc:new Date().toISOString(),calls:['Actual registered fixOperatorExpression function; real SysML services/AstReflection.createNode/assignNode','Synthetic already-constructed nodes only; no parser, build, linking, server or persistence'],limits:['Trusted builtin/process VM, not a security sandbox','Every own property/descriptor and object alias retained; accessors never evaluated','Current Node/V8 only; deployed Node20/artifact correspondence remains open','Native void return is separate from Rust Report; callbacks other than fixOperatorExpression are never invoked']}};
const output=JSON.stringify(result,null,2)+'\n';fs.writeFileSync(outputPath,output,{flag:'wx',mode:0o600});
console.log(JSON.stringify({cases:capture.cases.length,capture_errors:capture.cases.filter(c=>c.capture_error).length,output_path:outputPath,output_sha256:sha(output)}));
