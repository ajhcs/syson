"""Generate typed source-boundary observations; browser measurements are separate."""
import copy,json,struct,sys
from pathlib import Path
bits=lambda n:struct.pack('>d',n).hex()
def case(id,kind='Package',height=18.):
 return {'id':id,'category':'source_numeric','kind':kind,'node':{'id':'node','current':{},'defaults':{},'forced':{},'is_new':False,'resized_by_user':False,'keep_aspect_ratio':False,'direct_children':0,'border':{}},'previous':[],'visible_node_ids':['node'],'label_text':'Vehicle 雪 🧭','font':{'family':'Arial, sans-serif','size_px':bits(14.),'weight':400,'letter_spacing_px':bits(0.),'line_height_px':bits(20.)},'stamp':{'layout_revision':1,'font_revision':2,'request_id':3},'observation':{'bounds':{'width':bits(90.),'height':bits(height)}}}
a=[case('package-natural'),case('frame-natural','ViewFrame'),case('note-natural','Note')]
c=case('missing-label');c['observation']['bounds']=None;a.append(c)
c=case('absent-visible-id');c['visible_node_ids']=[];a.append(c)
c=case('first-visible-duplicate');c['visible_node_ids']=['other','node','node'];a.append(c)
for id,new,forced in [('previous-first',False,None),('new-ignores-previous',True,None),('forced-zero',False,0.),('forced-nonzero',False,9.)]:
 c=case(id);c['node'].update(is_new=new,resized_by_user=True);c['node']['defaults']={'width':bits(30.),'height':bits(20.)};c['previous']=[{'id':'node','dimensions':{'width':bits(300.)}},{'id':'node','dimensions':{'height':bits(800.)}}]
 if forced is not None:c['node']['forced']={'width':bits(forced),'height':bits(forced)}
 a.append(c)
for kind in ['Package','ViewFrame']:
 for power in [53,56]:a.append(case(f'rounding-{kind}-{power}',kind,float(2**power)))
for current in [0.,-0.]:
 for previous in [0.,-0.]:
  for axis in ['width','height']:
   c=case(f'note-tie-{axis}-{bits(current)}-{bits(previous)}','Note',current);c['node'].update(resized_by_user=True);c['node']['current']={axis:bits(current)};c['node']['defaults']={'height':bits(0.)};c['node']['border']={'left':bits(current)};c['previous']=[{'id':'node','dimensions':{'width':bits(0.),'height':bits(0.),axis:bits(previous)}}];a.append(c)
for label in [0.,-0.]:
 for default in [0.,-0.]:
  c=case(f'note-default-{bits(label)}-{bits(default)}','Note',label);c['node']['defaults']={'height':bits(default)};c['node']['border']={'all':bits(label)};a.append(c)
c=case('note-aspect','Note',60.);c['node'].update(keep_aspect_ratio=True);c['node']['defaults']={'width':bits(100.),'height':bits(50.)};c['node']['forced']={'width':bits(300.)};a.append(c)
for kind in ['Package','ViewFrame','Note']:
 c=case('maximum-finite-'+kind,kind,float.fromhex('0x1.fffffffffffffp+1023'));a.append(c)
for id,change in [('stale-font',{'stamp':{'layout_revision':1,'font_revision':99,'request_id':3}}),('wrong-target',{'target':{'kind':'dom','element_id':'foreign-label'}}),('host-failure',{'fail':True})]:
 c=case(id);c['category']='rust_boundary';c['observation'].update(change);a.append(c)
c=case('unsupported-parent');c['category']='rust_boundary';c['node']['direct_children']=1;a.append(c)
c=case('computed-height-overflow','Note');c['category']='rust_boundary';c['node']['border']={'left':bits(float.fromhex('0x1.fffffffffffffp+1023'))};a.append(c)
p=Path(sys.argv[1]);p.open('x').write(json.dumps({'schema':1,'cases':a},ensure_ascii=False,indent=2)+'\n');print(len(a),'cases')
