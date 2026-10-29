// Development-only in-memory native reference observation; no app or database.
import java.nio.file.*;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.node.ObjectNode;
import org.eclipse.emf.common.util.URI;
import org.eclipse.emf.ecore.EObject;
import org.eclipse.emf.ecore.resource.Resource;
import org.eclipse.emf.ecore.resource.ResourceSet;
import org.eclipse.sirius.components.emf.services.EObjectIDManager;
import org.eclipse.syson.application.configuration.SysONDefaultLibrariesConfiguration;
import org.eclipse.syson.application.configuration.SysONLoadDefaultLibrariesOnApplicationStartConfiguration;
import org.eclipse.syson.sysml.Element;

public final class ReferenceObserver {
  static final ObjectMapper JSON = new ObjectMapper();
  static final EObjectIDManager IDS = new EObjectIDManager();
  static ObjectNode tuple(EObject value) {
    var row=JSON.createObjectNode();
    if(value==null) { row.put("status","absent"); return row; }
    row.put("status",value.eIsProxy()?"proxy":"resolved");
    row.put("resource",value.eResource()==null?null:value.eResource().getURI().toString());
    row.put("node_id",IDS.findId(value).orElse(null));
    row.put("element_id",value instanceof Element e?e.getElementId():null);
    row.put("class",value.eClass().getName());
    row.put("name",value instanceof Element e?e.getDeclaredName():null);
    return row;
  }
  static EObject semantic(Resource r,String id) {
    for(var it=r.getAllContents();it.hasNext();) {
      var value=it.next();
      if(value instanceof Element e && id.equals(e.getElementId())) return value;
    }
    throw new IllegalStateException("fixture source object absent: "+id);
  }
  public static void main(String[] args) throws Exception {
    if(args.length!=2 || Files.exists(Path.of(args[1]))) throw new IllegalArgumentException("fixture,new owned output");
    var config=new SysONDefaultLibrariesConfiguration(new SysONLoadDefaultLibrariesOnApplicationStartConfiguration());
    ResourceSet rs=config.getLibrariesResourceSet();
    var vehicle=new org.eclipse.sirius.components.emf.services.JSONResourceFactory().createResource(URI.createURI("memory:/Batmobile.json"));
    rs.getResources().add(vehicle);
    if(vehicle==null) throw new IllegalStateException("native JSON resource factory absent");
    try(var input=Files.newInputStream(Path.of(args[0]))) { vehicle.load(input,java.util.Map.of()); }
    var result=JSON.createObjectNode();result.put("scope","ACTUAL_NATIVE_IN_MEMORY_RESOURCESET_SELECTED_REFERENCES");
    var cases=result.putObject("cases");
    for(String[] item:new String[][]{{"local","775ea645-9c42-4e52-8cd8-de30ed8b865b"},{"external","5dba8007-30b4-4988-9d3f-d5687ba94faa"}}) {
      var source=semantic(vehicle,item[1]);var feature=source.eClass().getEStructuralFeature("type");
      if(feature==null) throw new IllegalStateException("native feature absent");
      var target=(EObject)source.eGet(feature,true);var row=cases.putObject(item[0]);
      row.set("source",tuple(source));row.set("target",tuple(target));
      row.put("type_is_Type",target!=null && org.eclipse.syson.sysml.SysmlPackage.eINSTANCE.getType().isInstance(target));
    }
    var views=rs.getResources().stream().filter(r->r.getURI().toString().contains("faf517ae-dbcd-30a4-b3b9-3d9cb3bbf5c1")).findFirst().orElseThrow();
    var general=semantic(views,"03904fdf-d6f2-57b1-92d5-95d36b8208dc");
    result.put("view_node_alias",views.getEObject(IDS.findId(general).orElseThrow())==general);
    result.put("view_semantic_alias",views.getEObject(((Element)general).getElementId())==general);
    int before=rs.getResources().size();
    result.put("missing_without_load",rs.getEObject(URI.createURI("sysmllibrary:///00000000-0000-4000-8000-000000000099#00000000-0000-4000-8000-000000000099"),false)==null);
    result.put("missing_lookup_keeps_resource_count",rs.getResources().size()==before);
    result.put("resource_count",rs.getResources().size());
    result.put("diagnostics",rs.getResources().stream().mapToInt(r->r.getErrors().size()+r.getWarnings().size()).sum());
    try(var output=Files.newOutputStream(Path.of(args[1]),StandardOpenOption.CREATE_NEW)) { JSON.writerWithDefaultPrettyPrinter().writeValue(output,result); }
  }
}
