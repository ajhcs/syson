// SPDX-License-Identifier: EPL-2.0
import com.google.gson.*;
import java.nio.file.*;
import org.eclipse.emf.ecore.*;
import org.eclipse.syson.sysml.SysmlPackage;
import org.eclipse.syson.application.migration.*;
import org.eclipse.sirius.components.emf.migration.api.IMigrationParticipant;

public class MigrationOracle {
    static IMigrationParticipant participant(String unit) {
        return switch (unit) {
            case "B25a" -> new AnnotationAnnotatingElementMigrationParticipant();
            case "B25b" -> new ConnectorAsUsageSourceAndTargetMigrationParticipant();
            case "B25c" -> new ViewUsageExposeElementMigrationParticipant();
            default -> throw new IllegalArgumentException("Unexpected unit");
        };
    }
    public static void main(String[] args) throws Exception {
        if (args.length == 1 && args[0].equals("schema")) {
            JsonObject output = new JsonObject();
            output.addProperty("namespace",SysmlPackage.eNS_URI);
            output.addProperty("classifiers",SysmlPackage.eINSTANCE.getEClassifiers().size());
            for (String root : new String[]{"Annotation","AnnotatingElement","ConnectorAsUsage","ViewUsage"}) {
                EClass parent = (EClass)SysmlPackage.eINSTANCE.getEClassifier(root);
                JsonArray rows = new JsonArray();
                for (EClassifier type : SysmlPackage.eINSTANCE.getEClassifiers()) {
                    if (type instanceof EClass child && parent.isSuperTypeOf(child)) rows.add(child.getName());
                }
                output.add(root,rows);
            }
            System.out.println(output);return;
        }
        if (args.length != 2) throw new IllegalArgumentException("unit and fixture packet required");
        if (args[0].equals("sequence")) {
            JsonArray results=new JsonArray();
            JsonObject packet=JsonParser.parseString(Files.readString(Path.of(args[1]))).getAsJsonObject();
            for (JsonElement raw : packet.getAsJsonArray("cases")) {
                JsonObject fixture=raw.getAsJsonObject();JsonObject object=JsonParser.parseString(fixture.get("input").getAsString()).getAsJsonObject();
                JsonArray predicates=new JsonArray();JsonArray versions=new JsonArray();
                for (String unit : new String[]{"B25b","B25a","B25c"}) {
                    IMigrationParticipant migration=participant(unit);versions.add(migration.getVersion());
                    predicates.add(new MigrationParticipantService().isSysMLResource(object));migration.preDeserialization(null,object);
                }
                JsonObject row=new JsonObject();row.add("id",fixture.get("id"));row.add("json",object);row.add("matched_resources",predicates);row.add("versions",versions);row.addProperty("serialized",object.toString());results.add(row);
            }
            JsonObject output=new JsonObject();output.add("results",results);System.out.println(output);return;
        }
        JsonObject packet=JsonParser.parseString(Files.readString(Path.of(args[1]))).getAsJsonObject();
        JsonArray results=new JsonArray();
        for (JsonElement raw : packet.getAsJsonArray("cases")) {
            JsonObject fixture=raw.getAsJsonObject();JsonObject row=new JsonObject();
            row.add("id",fixture.get("id"));
            IMigrationParticipant migration=participant(args[0]);
            row.addProperty("version",migration.getVersion());
            try {
                JsonObject object=JsonParser.parseString(fixture.get("input").getAsString()).getAsJsonObject();
                row.addProperty("matched_resource",new MigrationParticipantService().isSysMLResource(object));
                migration.preDeserialization(null,object);
                row.addProperty("status","PREPARED");row.add("json",object);row.addProperty("serialized",object.toString());
            } catch (RuntimeException error) {
                row.addProperty("status","REFUSED");row.addProperty("error_class",error.getClass().getName());
                row.addProperty("error_message",error.getMessage());
            }
            results.add(row);
        }
        JsonObject output=new JsonObject();output.addProperty("unit",args[0]);output.add("results",results);System.out.println(output);
    }
}
