// Development-only remaining diagnostic details; does not replay node comparison.
import java.nio.file.*;
import com.fasterxml.jackson.core.*;
import org.eclipse.syson.application.configuration.*;
public class LibraryDiagnostics {
    public static void main(String[] args) throws Exception {
        var config = new SysONDefaultLibrariesConfiguration(new SysONLoadDefaultLibrariesOnApplicationStartConfiguration());
        try (var output = new java.io.FilterOutputStream(Files.newOutputStream(Path.of(args[0]), StandardOpenOption.CREATE_NEW)) {
            long count;
            @Override public void write(int b) throws java.io.IOException { if (++count > 16L*1024*1024) throw new java.io.IOException("diagnostic output cap"); out.write(b); }
            @Override public void write(byte[] b, int off, int len) throws java.io.IOException { count += len; if (count > 16L*1024*1024) throw new java.io.IOException("diagnostic output cap"); out.write(b,off,len); }
        }; var g = new JsonFactory().createGenerator(output,JsonEncoding.UTF8)) {
            g.writeStartObject();g.writeArrayFieldStart("resources");
            for (var r : config.getLibrariesResourceSet().getResources()) {
                g.writeStartObject();g.writeStringField("resource_uri",r.getURI().toString());
                for (var kind : java.util.List.of("errors","warnings")) {
                    g.writeArrayFieldStart(kind);
                    for (var d : kind.equals("errors") ? r.getErrors() : r.getWarnings()) {
                        g.writeStartObject();g.writeStringField("type",d.getClass().getName());g.writeStringField("message",d.getMessage());
                        g.writeStringField("location",d.getLocation());g.writeNumberField("line",d.getLine());g.writeNumberField("column",d.getColumn());g.writeEndObject();
                    }
                    g.writeEndArray();
                }
                g.writeEndObject();
            }
            g.writeEndArray();g.writeEndObject();
        }
    }
}
