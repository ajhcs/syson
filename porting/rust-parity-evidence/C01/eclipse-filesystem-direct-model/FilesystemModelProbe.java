import java.io.ByteArrayInputStream;
import java.io.InputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.security.MessageDigest;
import java.util.ArrayList;
import java.util.HexFormat;
import java.util.List;
import java.util.Properties;

import org.apache.maven.model.Dependency;
import org.apache.maven.model.Parent;
import org.apache.maven.model.Repository;
import org.apache.maven.model.building.DefaultModelBuilderFactory;
import org.apache.maven.model.building.DefaultModelBuildingRequest;
import org.apache.maven.model.building.ModelBuildingException;
import org.apache.maven.model.building.ModelBuildingRequest;
import org.apache.maven.model.building.ModelBuildingResult;
import org.apache.maven.model.building.ModelSource;
import org.apache.maven.model.resolution.ModelResolver;
import org.apache.maven.model.resolution.UnresolvableModelException;

final class FilesystemModelProbe {
    private static final String GROUP = "org.eclipse.platform";
    private static final String ARTIFACT = "org.eclipse.core.filesystem";

    private record FrozenSource(String version, byte[] contents) implements ModelSource {
        public InputStream getInputStream() {
            return new ByteArrayInputStream(contents);
        }

        public String getLocation() {
            return "pinned-pom:" + GROUP + ":" + ARTIFACT + ":" + version;
        }
    }

    private static final class NoResolution implements ModelResolver {
        private final List<String> callbacks;

        NoResolution(List<String> callbacks) {
            this.callbacks = callbacks;
        }

        private ModelSource refuse(String role, String group, String artifact, String version)
                throws UnresolvableModelException {
            callbacks.add(String.join("\t", "lookup-refused", role, clean(group), clean(artifact), clean(version)));
            throw new UnresolvableModelException("Model lookup is outside the replay", group, artifact, version);
        }

        public ModelSource resolveModel(String group, String artifact, String version)
                throws UnresolvableModelException {
            return refuse("coordinate", group, artifact, version);
        }

        public ModelSource resolveModel(Parent parent) throws UnresolvableModelException {
            return refuse("parent", parent.getGroupId(), parent.getArtifactId(), parent.getVersion());
        }

        public ModelSource resolveModel(Dependency dependency) throws UnresolvableModelException {
            return refuse("import", dependency.getGroupId(), dependency.getArtifactId(), dependency.getVersion());
        }

        public void addRepository(Repository repository) {
            addRepository(repository, false);
        }

        public void addRepository(Repository repository, boolean replace) {
            // Record declarations (including Maven's super POM) without registering or using any repository.
            callbacks.add(String.join("\t", "repository-ignored", clean(repository.getId()),
                clean(repository.getUrl()), Boolean.toString(replace)));
        }

        public ModelResolver newCopy() {
            return new NoResolution(callbacks);
        }
    }

    private static String clean(String value) {
        return value == null ? "" : value.replace('\t', ' ').replace('\n', ' ').replace('\r', ' ');
    }

    private static void printDependency(Dependency dependency) {
        System.out.println(String.join("\t", "direct", clean(dependency.getGroupId()),
            clean(dependency.getArtifactId()), clean(dependency.getVersion()), clean(dependency.getType()),
            clean(dependency.getClassifier()), clean(dependency.getScope()), clean(dependency.getOptional()),
            Boolean.toString(dependency.isOptional()), Integer.toString(dependency.getExclusions().size())));
    }

    private static final class WrongHash extends IllegalArgumentException { }

    private static void observe(String version, Path path, String expectedHash) throws Exception {
        long length = Files.size(path);
        if (length <= 0 || length > 262144) {
            throw new IllegalArgumentException("POM input length outside scope");
        }
        byte[] source = Files.readAllBytes(path);
        String hash = HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(source));
        if (!hash.equals(expectedHash) || source.length != length) {
            throw new WrongHash();
        }

        var system = new Properties();
        system.putAll(System.getProperties());
        System.getenv().forEach((key, value) -> system.setProperty("env." + key, value));
        var callbacks = new ArrayList<String>();
        var request = new DefaultModelBuildingRequest();
        request.setModelSource(new FrozenSource(version, source));
        request.setModelResolver(new NoResolution(callbacks));
        request.setSystemProperties(system);
        request.setUserProperties(new Properties());
        request.setProfiles(List.of());
        request.setActiveProfileIds(List.of());
        request.setInactiveProfileIds(List.of("sonar", "verify-iplog"));
        request.setValidationLevel(ModelBuildingRequest.VALIDATION_LEVEL_MAVEN_3_0);
        request.setProcessPlugins(false);
        request.setTwoPhaseBuilding(false);
        request.setLocationTracking(true);

        System.out.println("case\t" + version + "\t" + hash);
        ModelBuildingResult result;
        boolean built;
        try {
            result = new DefaultModelBuilderFactory().newInstance().build(request);
            built = true;
        } catch (ModelBuildingException failure) {
            result = failure.getResult();
            built = false;
        }
        callbacks.forEach(System.out::println);
        if (result == null) {
            throw new IllegalStateException("Maven returned no model-building result");
        }
        for (String modelId : result.getModelIds()) {
            if (modelId.isEmpty()) {
                continue;
            }
            var raw = result.getRawModel(modelId);
            System.out.println(String.join("\t", "maven-raw-model", clean(modelId), clean(raw.getModelVersion())));
        }
        for (var problem : result.getProblems()) {
            System.out.println("problem\t" + problem.getSeverity() + "\t" + clean(problem.getMessage()));
        }
        if (built) {
            var model = result.getEffectiveModel();
            System.out.println(String.join("\t", "effective-model", clean(model.getGroupId()),
                clean(model.getArtifactId()), clean(model.getVersion()), clean(model.getModelVersion()),
                clean(model.getPackaging())));
            for (Dependency dependency : model.getDependencies()) {
                printDependency(dependency);
            }
        }
        System.out.println("built\t" + built);
    }

    private static final List<String> VERSIONS = List.of(
        "1.7.0", "1.7.100", "1.7.200", "1.7.300", "1.7.400", "1.7.500", "1.7.600", "1.7.700", "1.9.0", "1.9.100", "1.9.200", "1.9.300", "1.9.400", "1.9.500", "1.10.0", "1.10.100", "1.10.200", "1.10.300", "1.10.400", "1.11.0", "1.11.100", "1.11.200", "1.11.300", "1.11.400", "1.11.500"
    );

    public static void main(String[] args) throws Exception {
        System.out.println("java\t" + Runtime.version());
        System.out.println("maven-model-builder\t" +
            DefaultModelBuilderFactory.class.getPackage().getImplementationVersion());
        if (args.length == 4 && "wrong-hash".equals(args[0]) && VERSIONS.getFirst().equals(args[1])) {
            try {
                observe(args[1], Path.of(args[2]), args[3]);
            } catch (WrongHash expected) {
                System.out.println("terminal\twrong-hash-refused\tbefore-model-builder");
                System.exit(41);
            }
            throw new IllegalStateException("wrong hash control reached model builder");
        }
        if (args.length != 1 + VERSIONS.size() * 3 || !"native".equals(args[0])) {
            throw new IllegalArgumentException("Expected the exact native direct census");
        }
        for (int i = 1; i < args.length; i += 3) {
            if (!VERSIONS.get((i - 1) / 3).equals(args[i])) {
                throw new IllegalArgumentException("Model order changed");
            }
            observe(args[i], Path.of(args[i + 1]), args[i + 2]);
        }
        System.out.println("terminal\tdirect-model-only\tno-dependency-graph-or-binary");
    }
}
