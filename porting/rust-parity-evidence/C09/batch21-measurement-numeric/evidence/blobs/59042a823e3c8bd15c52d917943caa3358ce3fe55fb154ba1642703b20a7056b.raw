// Developer supplied-observation oracle; real DOM measurements are separate.
'use strict';
const fs=require('node:fs'),vm=require('node:vm'),crypto=require('node:crypto');
const [methods,corpus,out]=process.argv.slice(2);
const sha=b=>crypto.createHash('sha256').update(b).digest('hex');
const decode=hex=>Buffer.from(hex,'hex').readDoubleBE();
const encode=n=>{const b=Buffer.alloc(8);b.writeDoubleBE(n);return b.toString('hex');};
const dimensions=o=>Object.fromEntries(Object.entries(o||{}).map(([k,v])=>[k,decode(v)]));
const source=fs.readFileSync(methods,'utf8'),input=JSON.parse(fs.readFileSync(corpus));
let bounds,queries=[];
const context=vm.createContext({document:{getElementById(id){queries.push(id);return bounds?{getBoundingClientRect:()=>bounds}:null;}}});
new vm.Script(source,{filename:methods}).runInContext(context,{timeout:30000});
const rows=[];
for(const c of input.cases){
 queries=[];bounds=c.observation.bounds?dimensions(c.observation.bounds):null;
 const n={id:c.node.id,...dimensions(c.node.current),data:{...Object.fromEntries(Object.entries(dimensions(c.node.defaults)).map(([k,v])=>[k==='width'?'defaultWidth':'defaultHeight',v])),isNew:c.node.is_new,resizedByUser:c.node.resized_by_user,style:{...('left' in c.node.border?{borderLeftWidth:decode(c.node.border.left)}:{}),...('all' in c.node.border?{borderWidth:decode(c.node.border.all)}:{})},insideLabel:bounds?{height:bounds.height}:undefined,nodeDescription:{keepAspectRatio:c.node.keep_aspect_ratio}}};
 const previous={nodes:c.previous.map(p=>({id:p.id,...dimensions(p.dimensions)}))};
 const visible=c.visible_node_ids.map(id=>({id})),forced=dimensions(c.node.forced);
 context.args={n,previous,visible,forced,kind:c.kind};
 new vm.Script(`(()=>{const {n,previous,visible,forced,kind}=args;if(kind==='Note')OriginalHandlers.note.handle(null,previous,n,visible,[],[],forced);else OriginalHandlers[kind==='Package'?'package':'view'].handleLeafNode(previous,n,visible,0,forced);})();`).runInContext(context,{timeout:30000});
 rows.push({id:c.id,category:c.category,size:{width:encode(n.width),height:encode(n.height)},queried_ids:queries,observation_source:'SUPPLIED_FACTS;NOTE_MODEL_HEIGHT_EXPLICIT_ASSIGNMENT'});
}
fs.writeFileSync(out,JSON.stringify({rows,provenance:{methods_sha256:sha(source),corpus_sha256:sha(fs.readFileSync(corpus)),oracle_sha256:sha(fs.readFileSync(__filename)),node:process.version,v8:process.versions.v8,scope:'Type-erased original selected source methods on supplied facts; no actual DOM or deployed-module qualification.'}},null,2)+'\n',{flag:'wx'});
console.log(JSON.stringify({cases:rows.length,out}));
