// Development-only namespace admission controls using the original native factory.
import java.nio.file.*;
import com.fasterxml.jackson.databind.ObjectMapper;
import org.eclipse.emf.common.util.URI;
import org.eclipse.emf.ecore.resource.impl.ResourceSetImpl;
import org.eclipse.sirius.components.emf.services.JSONResourceFactory;
import org.eclipse.syson.sysml.SysmlPackage;
public final class NamespaceObserver {
 public static void main(String[] args) throws Exception {
  var json=new ObjectMapper();var output=json.createArrayNode();
  for(String ns:new String[]{SysmlPackage.eNS_URI,"urn:syson-parity:foreign","ABSENT"}) {
   var rs=new ResourceSetImpl();rs.getPackageRegistry().put(SysmlPackage.eNS_URI,SysmlPackage.eINSTANCE);
   var root=json.createObjectNode();root.putObject("json").put("version","1.0");if(!ns.equals("ABSENT"))root.putObject("ns").put("sysml",ns);
   var node=root.putArray("content").addObject();node.put("id","10000000-0000-4000-8000-000000000001");node.put("eClass","sysml:PartDefinition");node.putObject("data").put("elementId","20000000-0000-4000-8000-000000000001");
   var resource=new JSONResourceFactory().createResource(URI.createURI("memory:/namespace.json"));rs.getResources().add(resource);var row=output.addObject();row.put("namespace",ns);row.set("input",root);
   try {resource.load(new java.io.ByteArrayInputStream(json.writeValueAsBytes(root)),java.util.Map.of());row.put("status","loaded");}
   catch(Exception e){row.put("status","refused");row.put("error",e.toString());}
   row.put("roots",resource.getContents().size());row.put("errors",resource.getErrors().size());row.put("warnings",resource.getWarnings().size());if(!resource.getContents().isEmpty())row.put("class",resource.getContents().getFirst().eClass().getName());
  }
  try(var stream=Files.newOutputStream(Path.of(args[1]),StandardOpenOption.CREATE_NEW)){json.writerWithDefaultPrettyPrinter().writeValue(stream,output);}
 }
}
