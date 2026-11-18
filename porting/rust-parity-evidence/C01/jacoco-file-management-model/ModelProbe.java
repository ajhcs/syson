import java.io.ByteArrayInputStream;
import java.io.InputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
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
    private static final String BOM = "org.junit:junit-bom:5.12.1";
    private static final List<String> COORDINATES = List.of(
        "org.apache.maven.shared:file-management:3.2.0",
        "org.apache.maven.shared:maven-shared-components:44",
        "org.apache.maven:maven-parent:44",
        "org.apache:apache:34", BOM
    );

    private record Bytes(String coordinate, byte[] contents) implements ModelSource {
        public InputStream getInputStream() {
            return new ByteArrayInputStream(contents);
        }

        public String getLocation() {
            return "pinned-pom:" + coordinate;
        }
    }

    private static final class FrozenResolver implements ModelResolver {
        private final Map<String, Bytes> parents;
        private final List<String> requests;
        private final boolean admitBom;

        FrozenResolver(Map<String, Bytes> parents, List<String> requests, boolean admitBom) {
            this.parents = parents;
            this.requests = requests;
            this.admitBom = admitBom;
        }

        private Bytes lookup(String role, String group, String artifact, String version)
                throws UnresolvableModelException {
            var coordinate = group + ":" + artifact + ":" + version;
            boolean approved = role.equals("parent") ? !BOM.equals(coordinate)
                : admitBom && BOM.equals(coordinate) && List.of("coordinate", "import").contains(role);
            var model = approved ? parents.get(coordinate) : null;
            requests.add(role + "\t" + coordinate + "\t" + (model == null ? "absent" : "pinned"));
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
            // Repository declarations are model data. This resolver never uses them.
        }

        public void addRepository(Repository repository, boolean replace) {
            addRepository(repository);
        }

        public ModelResolver newCopy() {
            return new FrozenResolver(parents, requests, admitBom);
        }
    }

    public static void main(String[] args) throws Exception {
        if (args.length != COORDINATES.size() + 1) {
            throw new IllegalArgumentException("expected case plus five pinned model paths");
        }
        String probeCase = args[0];
        if (!List.of("baseline", "ci-property", "selector-control", "withheld-bom").contains(probeCase)) {
            throw new IllegalArgumentException("unknown probe case");
        }

        var models = new LinkedHashMap<String, Bytes>();
        for (int index = 0; index < COORDINATES.size(); index++) {
            var coordinate = COORDINATES.get(index);
            var source = Files.readAllBytes(Path.of(args[index + 1]));
            if (source.length == 0 || source.length > 262144) {
                throw new IllegalArgumentException("pinned model byte limit: " + coordinate);
            }
            models.put(coordinate, new Bytes(coordinate, source));
        }
        var root = models.remove(COORDINATES.get(0));

        var system = new Properties();
        system.putAll(System.getProperties());
        System.getenv().forEach((key, value) -> system.setProperty("env." + key, value));
        var user = new Properties();
        user.setProperty("style.color", "never");
        if (probeCase.equals("ci-property")) {
            system.setProperty("env.CI", "true");
        }
        if (probeCase.equals("selector-control")) {
            user.setProperty("versions.junit5", "99.99.99-model-probe");
        }

        var requests = new ArrayList<String>();
        var request = new DefaultModelBuildingRequest();
        request.setModelSource(root);
        request.setModelResolver(new FrozenResolver(Map.copyOf(models), requests, !probeCase.equals("withheld-bom")));
        request.setSystemProperties(system);
        request.setUserProperties(user);
        request.setProfiles(List.of());
        request.setActiveProfileIds(List.of());
        request.setInactiveProfileIds(List.of("sonar", "verify-iplog"));
        request.setValidationLevel(ModelBuildingRequest.VALIDATION_LEVEL_MAVEN_3_0);
        request.setProcessPlugins(false);
        request.setTwoPhaseBuilding(false);
        request.setLocationTracking(true);

        System.out.println("case\t" + probeCase);
        System.out.println("java\t" + Runtime.version());
        System.out.println("maven-model-builder\t" +
            DefaultModelBuilderFactory.class.getPackage().getImplementationVersion());

        ModelBuildingResult result;
        boolean built = false;
        try {
            result = new DefaultModelBuilderFactory().newInstance().build(request);
            built = true;
        } catch (ModelBuildingException failure) {
            result = failure.getResult();
        }

        for (String attempt : requests) {
            System.out.println("request\t" + attempt);
        }
        if (result == null) {
            throw new IllegalStateException("Maven returned no model-building result");
        }
        for (String modelId : result.getModelIds()) {
            if (modelId.isEmpty()) {
                continue;
            }
            var model = result.getRawModel(modelId);
            var activeProfiles = result.getActivePomProfiles(modelId).stream()
                .map(Profile::getId).toList();
            System.out.println("profiles\t" + modelId + "\t" + String.join(",", activeProfiles));
            var selector = model.getProperties().getProperty("versions.junit5");
            if (selector != null) {
                System.out.println("raw-selector\t" + modelId + "\t" + selector);
            }
        }
        for (var problem : result.getProblems()) {
            System.out.println("problem\t" + problem.getSeverity() + "\t" +
                problem.getMessage().replace('\t', ' ').replace('\n', ' ').replace('\r', ' '));
        }
        if (built) {
            var effective = result.getEffectiveModel();
            System.out.println("effective\t" + effective.getGroupId() + ":" + effective.getArtifactId() + ":" +
                effective.getVersion());
            var managed = effective.getDependencyManagement().getDependencies().stream()
                .filter(d -> "org.junit.jupiter".equals(d.getGroupId()) && "junit-jupiter-api".equals(d.getArtifactId()))
                .toList();
            var direct = effective.getDependencies().stream()
                .filter(d -> "org.junit.jupiter".equals(d.getGroupId()) && "junit-jupiter-api".equals(d.getArtifactId()))
                .toList();
            for (var dependency : managed) {
                System.out.println("jupiter-managed\t" + dependency.getVersion());
            }
            for (var dependency : direct) {
                System.out.println("jupiter-direct\t" + dependency.getVersion() + "\t" + dependency.getScope());
            }
        }
        System.out.println("built\t" + built);
    }
}
