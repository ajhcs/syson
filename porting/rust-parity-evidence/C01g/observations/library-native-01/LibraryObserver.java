// SPDX-License-Identifier: EPL-2.0
// Root-run only: real configuration/ResourceSet observation, no DB or app launch.
import java.io.File;
import java.security.MessageDigest;
import java.util.HexFormat;
import com.fasterxml.jackson.core.JsonFactory;
import com.fasterxml.jackson.core.JsonGenerator;
import org.eclipse.emf.ecore.EObject;
import org.eclipse.emf.ecore.resource.Resource;
import org.eclipse.emf.ecore.resource.ResourceSet;
import org.eclipse.sirius.components.emf.ResourceMetadataAdapter;
import org.eclipse.sirius.components.emf.services.EObjectIDManager;
import org.eclipse.syson.application.configuration.SysONDefaultLibrariesConfiguration;
import org.eclipse.syson.application.configuration.SysONLoadDefaultLibrariesOnApplicationStartConfiguration;
import org.eclipse.syson.application.libraries.SysONLibraryLoader;
import org.eclipse.syson.sysml.Element;
import org.eclipse.syson.sysml.LibraryPackage;

public final class LibraryObserver {
    public static void main(String[] args) throws Exception {
        if (args.length != 1 || new File(args[0]).exists()) throw new IllegalArgumentException("new owned output file required");
        var config = new SysONDefaultLibrariesConfiguration(new SysONLoadDefaultLibrariesOnApplicationStartConfiguration());
        ResourceSet rs = config.getLibrariesResourceSet();
        var loader = new SysONLibraryLoader();
        var ids = new EObjectIDManager();
        try (var output = new java.io.FilterOutputStream(java.nio.file.Files.newOutputStream(java.nio.file.Path.of(args[0]), java.nio.file.StandardOpenOption.CREATE_NEW)) {
            long count = 0;
            @Override public void write(int b) throws java.io.IOException { if (++count > 64L*1024*1024) throw new java.io.IOException("native output cap"); out.write(b); }
            @Override public void write(byte[] b, int off, int len) throws java.io.IOException { count += len; if (count > 64L*1024*1024) throw new java.io.IOException("native output cap"); out.write(b, off, len); }
        }; JsonGenerator g = new JsonFactory().createGenerator(output, com.fasterxml.jackson.core.JsonEncoding.UTF8)) {
            g.writeStartObject(); g.writeStringField("claim", "ACTUAL_IN_MEMORY_DEFAULT_LIBRARY_OBSERVATION");
            g.writeArrayFieldStart("classpath_resources");
            for (var def : java.util.List.of(SysONDefaultLibrariesConfiguration.KERML, SysONDefaultLibrariesConfiguration.SYSML)) {
                for (var r : loader.findAllLibraryResourcesFromApplicationClasspath(def)) {
                    byte[] raw = r.getContentAsByteArray();
                    g.writeStartObject(); g.writeStringField("classpath_path", r.getPath());
                    g.writeStringField("resource_uri", loader.createEmfUriFor(r, def).toString());
                    g.writeStringField("sha256", HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(raw)));
                    g.writeNumberField("bytes", raw.length); g.writeEndObject();
                }
            }
            g.writeEndArray(); g.writeArrayFieldStart("resources");
            for (Resource r : rs.getResources()) {
                g.writeStartObject(); g.writeStringField("resource_uri", r.getURI().toString());
                var metadata = r.eAdapters().stream().filter(ResourceMetadataAdapter.class::isInstance).map(ResourceMetadataAdapter.class::cast).findFirst();
                g.writeObjectFieldStart("metadata");
                if (metadata.isPresent()) { g.writeStringField("name", metadata.get().getName()); g.writeBooleanField("read_only", metadata.get().isReadOnly()); }
                g.writeEndObject(); g.writeBooleanField("is_loaded", r.isLoaded());
                g.writeNumberField("root_count", r.getContents().size());
                g.writeArrayFieldStart("errors"); for (var d : r.getErrors()) g.writeString(d.toString()); g.writeEndArray();
                g.writeArrayFieldStart("warnings"); for (var d : r.getWarnings()) g.writeString(d.toString()); g.writeEndArray();
                g.writeArrayFieldStart("nodes");
                long count = 0;
                for (var it = r.getAllContents(); it.hasNext();) {
                    EObject o = it.next(); count++;
                    String node = ids.findId(o).orElse(null);
                    String semantic = o instanceof Element e ? e.getElementId() : null;
                    g.writeStartObject(); g.writeStringField("node_id", node); g.writeStringField("semantic_id", semantic);
                    g.writeStringField("e_class", o.eClass().getName());
                    g.writeStringField("package_uri", o.eClass().getEPackage().getNsURI());
                    g.writeStringField("declared_name", o instanceof Element e ? e.getDeclaredName() : null);
                    g.writeStringField("fragment", r.getURIFragment(o));
                    boolean root = r.getContents().contains(o);
                    g.writeBooleanField("root", root);
                    if (root || o instanceof LibraryPackage) {
                        g.writeBooleanField("node_lookup_same", node != null && r.getEObject(node) == o);
                        g.writeBooleanField("semantic_lookup_same", semantic != null && r.getEObject(semantic) == o);
                    }
                    g.writeEndObject();
                }
                g.writeEndArray(); g.writeNumberField("object_count", count); g.writeEndObject();
            }
            g.writeEndArray(); g.writeEndObject();
        }
    }
}
