// SPDX-License-Identifier: EPL-2.0
import java.nio.file.*;
import java.util.List;
import org.eclipse.syson.application.migration.StartDoneAndControlNodeMigrationParticipant;
import org.eclipse.sirius.components.collaborative.representations.migration.RepresentationMigrationService;
import org.eclipse.sirius.components.core.api.IEditingContext;
import tools.jackson.databind.*;
import tools.jackson.databind.json.JsonMapper;
import tools.jackson.databind.node.*;
public class ActionOracle {
 public static void main(String[] args) throws Exception {
  if(args.length!=1)throw new IllegalArgumentException("one packet path required");
  ObjectMapper mapper=JsonMapper.builder().build();JsonNode packet=mapper.readTree(Files.readString(Path.of(args[0])));ArrayNode results=mapper.createArrayNode();
  for(JsonNode fixture:packet.get("cases")){
   StartDoneAndControlNodeMigrationParticipant participant=new StartDoneAndControlNodeMigrationParticipant();
   ObjectNode row=mapper.createObjectNode();row.put("id",fixture.get("id").asString());row.put("version",participant.getVersion());row.put("kind",participant.getKind());ObjectNode object=null;
   try {
    object=(ObjectNode)mapper.readTree(fixture.get("input").asString());JsonNode before=object.deepCopy();
    if(fixture.get("mode").asString().equals("callback"))participant.replaceJsonNode(null,object,fixture.get("current_attribute").asString(),fixture.get("current_value"));
    else {
     IEditingContext context=new IEditingContext(){public String getId(){return "owned-unused-oracle-context";}};
     new RepresentationMigrationService(context,List.of(participant),object).parseProperties(object,mapper);
    }
    row.put("status","PREPARED");row.set("json",object);row.put("changed",!before.equals(object));
   }catch(RuntimeException error){row.put("status","REFUSED");row.put("error_class",error.getClass().getName());row.put("error_message",error.getMessage());if(object!=null)row.set("native_partial_tree",object);}
   results.add(row);
  }
  ObjectNode output=mapper.createObjectNode();output.set("results",results);System.out.println(mapper.writeValueAsString(output));
 }
}
