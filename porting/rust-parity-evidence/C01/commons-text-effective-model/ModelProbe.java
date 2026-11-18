import java.io.ByteArrayInputStream;
import java.io.InputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.security.MessageDigest;
import java.util.ArrayList;
import java.util.HexFormat;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Properties;

import org.apache.maven.model.Dependency;
import org.apache.maven.model.Parent;
import org.apache.maven.model.Profile;
import org.apache.maven.model.Repository;
import org.apache.maven.model.building.DefaultModelBuilderFactory;
import org.apache.maven.model.building.DefaultModelBuildingRequest;
import org.apache.maven.model.building.ModelBuildingException;
import org.apache.maven.model.building.ModelBuildingRequest;
import org.apache.maven.model.building.ModelBuildingResult;
import org.apache.maven.model.building.ModelSource;
import org.apache.maven.model.resolution.ModelResolver;
import org.apache.maven.model.resolution.UnresolvableModelException;

final class ModelProbe {
    private static final String ROOT = "org.apache.commons:commons-text:1.10.0";
    private static final String COMMONS_PARENT = "org.apache.commons:commons-parent:54";
    private static final String APACHE_PARENT = "org.apache:apache:27";
    private static final String BOM = "org.junit:junit-bom:5.9.1";

    private record FrozenSource(String coordinate, byte[] contents) implements ModelSource {
        public InputStream getInputStream() {
            return new ByteArrayInputStream(contents);
        }

        public String getLocation() {
            return "pinned-pom:" + coordinate;
        }
    }

    private static final class FrozenResolver implements ModelResolver {
        private final Map<String, FrozenSource> models;
        private final List<String> requests;

        FrozenResolver(Map<String, FrozenSource> models, List<String> requests) {
            this.models = models;
            this.requests = requests;
        }

        private ModelSource lookup(String role, String group, String artifact, String version)
                throws UnresolvableModelException {
            String coordinate = group + ":" + artifact + ":" + version;
            boolean allowed = role.equals("parent")
                ? List.of(COMMONS_PARENT, APACHE_PARENT).contains(coordinate)
                : BOM.equals(coordinate);
            FrozenSource model = allowed ? models.get(coordinate) : null;
            requests.add(String.join("\t", role, clean(coordinate), model == null ? "absent" : "pinned"));
            if (model == null) {
                throw new UnresolvableModelException("not available in the pinned offline model set",
                    group, artifact, version);
            }
            return model;
        }

        public ModelSource resolveModel(String group, String artifact, String version)
                throws UnresolvableModelException {
            return lookup("coordinate", group, artifact, version);
        }

        public ModelSource resolveModel(Parent parent) throws UnresolvableModelException {
            return lookup("parent", parent.getGroupId(), parent.getArtifactId(), parent.getVersion());
        }

        public ModelSource resolveModel(Dependency dependency) throws UnresolvableModelException {
            return lookup("import", dependency.getGroupId(), dependency.getArtifactId(), dependency.getVersion());
        }

        public void addRepository(Repository repository) {
            // No repository map or transport exists. Maven repository declarations are ignored.
        }

        public void addRepository(Repository repository, boolean replace) {
            addRepository(repository);
        }

        public ModelResolver newCopy() {
            return new FrozenResolver(models, requests);
        }
    }

    private static String clean(String value) {
        return value == null ? "" : value.replace('\t', ' ').replace('\n', ' ').replace('\r', ' ');
    }

    private static String dependency(Dependency d) {
        return String.join("\t", clean(d.getGroupId()), clean(d.getArtifactId()), clean(d.getVersion()),
            clean(d.getType()), clean(d.getClassifier()), clean(d.getScope()));
    }

    public static void main(String[] args) throws Exception {
        if (args.length == 0 || !List.of("baseline", "withheld-bom").contains(args[0])) {
            throw new IllegalArgumentException("expected a known probe case");
        }
        String probeCase = args[0];
        var coordinates = probeCase.equals("baseline") ? List.of(ROOT, COMMONS_PARENT, APACHE_PARENT, BOM)
            : List.of(ROOT, COMMONS_PARENT, APACHE_PARENT);
        if (args.length != 1 + 2 * coordinates.size()) {
            throw new IllegalArgumentException("expected only the exact model path and digest pairs for this case");
        }

        System.out.println("case\t" + probeCase);
        System.out.println("java\t" + Runtime.version());
        System.out.println("maven-model-builder\t" +
            DefaultModelBuilderFactory.class.getPackage().getImplementationVersion());
        var models = new LinkedHashMap<String, FrozenSource>();
        for (int index = 0; index < coordinates.size(); index++) {
            String coordinate = coordinates.get(index);
            Path path = Path.of(args[1 + 2 * index]);
            long length = Files.size(path);
            if (length <= 0 || length > 262144) {
                throw new IllegalArgumentException("model byte limit: " + coordinate);
            }
            byte[] source = Files.readAllBytes(path);
            String hash = HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(source));
            if (source.length != length || !hash.equals(args[2 + 2 * index])) {
                throw new IllegalArgumentException("model byte drift: " + coordinate);
            }
            System.out.println("input\t" + coordinate + "\t" + hash);
            models.put(coordinate, new FrozenSource(coordinate, source));
        }
        FrozenSource root = models.remove(ROOT);

        var system = new Properties();
        system.putAll(System.getProperties());
        System.getenv().forEach((key, value) -> system.setProperty("env." + key, value));
        var requests = new ArrayList<String>();
        var request = new DefaultModelBuildingRequest();
        request.setModelSource(root);
        request.setModelResolver(new FrozenResolver(Map.copyOf(models), requests));
        request.setSystemProperties(system);
        request.setUserProperties(new Properties());
        request.setProfiles(List.of());
        request.setActiveProfileIds(List.of());
        request.setInactiveProfileIds(List.of("sonar", "verify-iplog"));
        request.setValidationLevel(ModelBuildingRequest.VALIDATION_LEVEL_MAVEN_3_0);
        request.setProcessPlugins(false);
        request.setTwoPhaseBuilding(false);
        request.setLocationTracking(true);

        ModelBuildingResult result;
        boolean built;
        try {
            result = new DefaultModelBuilderFactory().newInstance().build(request);
            built = true;
        } catch (ModelBuildingException failure) {
            result = failure.getResult();
            built = false;
        }
        requests.forEach(attempt -> System.out.println("request\t" + attempt));
        if (result == null) {
            throw new IllegalStateException("Maven returned no model-building result");
        }
        for (String modelId : result.getModelIds()) {
            if (modelId.isEmpty()) {
                continue;
            }
            var raw = result.getRawModel(modelId);
            var activeProfiles = result.getActivePomProfiles(modelId).stream().map(Profile::getId).toList();
            System.out.println("profiles\t" + modelId + "\t" + String.join(",", activeProfiles));
            var selector = raw.getProperties().getProperty("commons.junit.version");
            if (selector != null) {
                System.out.println("raw-selector\t" + modelId + "\t" + selector);
            }
        }
        for (var problem : result.getProblems()) {
            System.out.println("problem\t" + problem.getSeverity() + "\t" + clean(problem.getMessage()));
        }
        if (built) {
            var effective = result.getEffectiveModel();
            System.out.println("effective\t" + effective.getGroupId() + ":" + effective.getArtifactId() + ":" +
                effective.getVersion() + "\t" + effective.getPackaging());
            for (Dependency d : effective.getDependencyManagement().getDependencies()) {
                if (d.getGroupId().startsWith("org.junit.")) {
                    System.out.println("junit-managed\t" + dependency(d));
                }
            }
            for (Dependency d : effective.getDependencies()) {
                if (("org.apache.commons".equals(d.getGroupId()) && "commons-lang3".equals(d.getArtifactId()))
                    || d.getGroupId().startsWith("org.junit.")) {
                    System.out.println("direct\t" + dependency(d));
                }
            }
        }
        System.out.println("built\t" + built);
    }
}
