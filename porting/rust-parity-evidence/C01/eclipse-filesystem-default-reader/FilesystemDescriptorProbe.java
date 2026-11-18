import java.nio.file.Files;
import java.nio.file.Path;
import java.security.MessageDigest;
import java.util.ArrayList;
import java.util.Collection;
import java.util.HexFormat;
import java.util.List;
import java.util.Properties;

import org.apache.maven.model.building.DefaultModelBuilderFactory;
import org.apache.maven.repository.internal.DefaultArtifactDescriptorReader;
import org.apache.maven.repository.internal.DefaultModelCacheFactory;
import org.apache.maven.repository.internal.MavenRepositorySystemUtils;
import org.eclipse.aether.RepositorySystemSession;
import org.eclipse.aether.artifact.Artifact;
import org.eclipse.aether.artifact.DefaultArtifact;
import org.eclipse.aether.impl.ArtifactResolver;
import org.eclipse.aether.impl.RemoteRepositoryManager;
import org.eclipse.aether.repository.RemoteRepository;
import org.eclipse.aether.repository.RepositoryPolicy;
import org.eclipse.aether.resolution.ArtifactDescriptorPolicy;
import org.eclipse.aether.resolution.ArtifactDescriptorPolicyRequest;
import org.eclipse.aether.resolution.ArtifactDescriptorRequest;
import org.eclipse.aether.resolution.ArtifactRequest;
import org.eclipse.aether.resolution.ArtifactResult;
import org.eclipse.aether.resolution.VersionResult;

final class FilesystemDescriptorProbe {
    private static final String GROUP = "org.eclipse.platform";
    private static final String NAME = "org.eclipse.core.filesystem";
    private static final String GA = GROUP + ":" + NAME;
    private static final RemoteRepository CENTRAL = new RemoteRepository.Builder(
        "central", "default", "https://repo.maven.apache.org/maven2")
        .setReleasePolicy(new RepositoryPolicy(true, "daily", "warn"))
        .setSnapshotPolicy(new RepositoryPolicy(false, "daily", "warn")).build();
    private static final List<RemoteRepository> REPOSITORIES = List.of(CENTRAL);

    private record Pinned(String version, Path path, String digest) {
        void verify() throws Exception {
            long length = Files.size(path);
            if (!Files.isRegularFile(path) || length <= 0 || length > 262144) {
                throw new IllegalStateException("POM outside admitted shape");
            }
            byte[] raw = Files.readAllBytes(path);
            var hash = HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(raw));
            if (raw.length != length || !hash.equals(digest)) {
                throw new IllegalStateException("POM digest differs");
            }
        }
    }

    private static final class Refused extends IllegalStateException {
        Refused(String message) { super(message); }
    }

    private static final class Guard {
        private final Pinned input;
        private String fault;
        private int versionJar, versionPom, pom, repositories, invalidEvents;

        Guard(Pinned input) { this.input = input; }

        Refused refusal(String reason) {
            if (fault == null) {
                fault = reason;
            }
            return new Refused(reason);
        }

        void require(boolean allowed, String reason) {
            if (!allowed) throw refusal(reason);
        }

        void context(RepositorySystemSession session, List<RemoteRepository> origins, String requestContext) {
            require(session.isOffline() && origins.size() == 1 && central(origins.getFirst())
                    && "project".equals(requestContext), "descriptor origin, context or offline drift");
        }

        void coordinate(Artifact a, String type) {
            require(GROUP.equals(a.getGroupId()) && NAME.equals(a.getArtifactId())
                    && input.version().equals(a.getVersion()) && a.getClassifier().isEmpty()
                    && type.equals(a.getExtension()), "coordinate outside exact Filesystem case");
        }

        void clean() {
            if (fault != null) throw new Refused(fault);
        }
    }

    private static boolean policy(RepositoryPolicy p, boolean enabled) {
        return p.isEnabled() == enabled && "daily".equals(p.getUpdatePolicy()) && "warn".equals(p.getChecksumPolicy());
    }

    private static boolean central(RemoteRepository r) {
        return "central".equals(r.getId()) && CENTRAL.getUrl().equals(r.getUrl())
            && "default".equals(r.getContentType()) && r.getProxy() == null
            && r.getAuthentication() == null && !r.isBlocked() && !r.isRepositoryManager()
            && r.getMirroredRepositories().isEmpty()
            && policy(r.getPolicy(false), true) && policy(r.getPolicy(true), false);
    }

    private static RemoteRepositoryManager repositoryGuard(Guard guard) {
        return new RemoteRepositoryManager() {
            @Override
            public List<RemoteRepository> aggregateRepositories(RepositorySystemSession session,
                    List<RemoteRepository> dominant, List<RemoteRepository> recessive, boolean raw) {
                guard.require(session.isOffline() && raw && recessive.size() == 1 && central(recessive.getFirst())
                    && (dominant.isEmpty() || (dominant.size() == 1 && central(dominant.getFirst()))),
                    "repository aggregation outside inherited super-POM Central");
                guard.require(++guard.repositories <= 4, "repository callback bound");
                return new ArrayList<>(REPOSITORIES);
            }

            @Override
            public RepositoryPolicy getPolicy(RepositorySystemSession session, RemoteRepository repository,
                    boolean releases, boolean snapshots) {
                throw guard.refusal("repository transfer policy refused");
            }
        };
    }

    private static ArtifactResolver pomOnly(Guard guard) {
        return new ArtifactResolver() {
            @Override
            public ArtifactResult resolveArtifact(RepositorySystemSession session, ArtifactRequest request) {
                guard.context(session, request.getRepositories(), request.getRequestContext());
                var a = request.getArtifact();
                guard.coordinate(a, "pom");
                guard.require(++guard.pom == 1, "POM callback count differs");
                return new ArtifactResult(request).setArtifact(a.setFile(guard.input.path().toFile()))
                    .setRepository(CENTRAL);
            }

            @Override
            public List<ArtifactResult> resolveArtifacts(RepositorySystemSession session,
                    Collection<? extends ArtifactRequest> requests) {
                throw guard.refusal("batch resolution refused");
            }
        };
    }

    private static void row(String... fields) {
        System.out.println(String.join("\t", fields));
    }

    private static void observe(Pinned input, String requested) throws Exception {
        input.verify();
        var guard = new Guard(input);
        var session = MavenRepositorySystemUtils.newSession().setOffline(true);
        var props = new Properties();
        props.putAll(System.getProperties());
        System.getenv().forEach((k, v) -> props.setProperty("env." + k, v));
        var user = new Properties();
        user.setProperty("style.color", "never");
        session.setSystemProperties(props).setUserProperties(user);

        var request = new ArtifactDescriptorRequest(new DefaultArtifact(GA + ":" + requested),
                                                     REPOSITORIES, "project");
        guard.context(session, request.getRepositories(), request.getRequestContext());
        guard.coordinate(request.getArtifact(), "jar");
        int flags = session.getArtifactDescriptorPolicy().getPolicy(session,
            new ArtifactDescriptorPolicyRequest(request.getArtifact(), request.getRequestContext()));
        guard.require(flags == (ArtifactDescriptorPolicy.IGNORE_INVALID | ArtifactDescriptorPolicy.IGNORE_MISSING),
            "Maven default invalid/missing policy drift");

        var reader = new DefaultArtifactDescriptorReader(repositoryGuard(guard),
            (offline, version) -> {
                guard.context(offline, version.getRepositories(), version.getRequestContext());
                var a = version.getArtifact();
                switch (a.getExtension()) {
                    case "jar" -> {
                        guard.coordinate(a, "jar");
                        guard.require(++guard.versionJar == 1, "JAR coordinate echo count differs");
                    }
                    case "pom" -> {
                        guard.coordinate(a, "pom");
                        guard.require(++guard.versionPom == 1, "POM coordinate echo count differs");
                    }
                    default -> throw guard.refusal("version callback type refused");
                }
                return new VersionResult(version).setVersion(input.version());
            },
            (offline, range) -> { throw guard.refusal("range or secondary model lookup refused"); },
            pomOnly(guard), new DefaultModelBuilderFactory().newInstance(),
            event -> {
                guard.invalidEvents++;
                throw guard.refusal("unexpected descriptor event");
            }, new DefaultModelCacheFactory());
        var result = reader.readArtifactDescriptor(session, request);
        guard.clean();
        guard.require(result.getArtifact().getFile() == null, "binary path outside scope");
        guard.coordinate(result.getArtifact(), "jar");
        guard.require(guard.versionJar == 1 && guard.versionPom == 1 && guard.pom == 1
            && guard.repositories > 0 && guard.invalidEvents == 0
            && result.getExceptions().isEmpty() && result.getRelocations().isEmpty()
            && result.getManagedDependencies().isEmpty() && result.getAliases().isEmpty(),
            "unexpected resolution or descriptor outcome");
        input.verify();
        row("case", input.version(), input.digest(), Integer.toString(flags));
        row("callbacks", Integer.toString(guard.versionJar), Integer.toString(guard.versionPom),
            Integer.toString(guard.pom), Integer.toString(guard.repositories), Integer.toString(guard.invalidEvents));
        for (var dependency : result.getDependencies()) {
            var child = dependency.getArtifact();
            row("edge", child.getGroupId(), child.getArtifactId(), child.getVersion(), child.getExtension(),
                child.getClassifier(), dependency.getScope(), Boolean.toString(dependency.isOptional()),
                Integer.toString(dependency.getExclusions().size()));
        }
        row("native-built", Integer.toString(result.getDependencies().size()));
    }

    public static void main(String[] args) throws Exception {
        row("java", Runtime.version().toString());
        row("maven-model-builder", DefaultModelBuilderFactory.class.getPackage().getImplementationVersion());
        if (args.length == 4 && "outside-coordinate".equals(args[0])) {
            try {
                observe(new Pinned(args[1], Path.of(args[2]), args[3]), "1.6.1");
            } catch (Refused expected) {
                if (!"coordinate outside exact Filesystem case".equals(expected.getMessage())) throw expected;
                row("terminal", "outside-coordinate-refused", "before-descriptor-reader");
                System.exit(42);
            }
            throw new IllegalStateException("unexpected outside-coordinate acceptance");
        }
        if (args.length != 76 || !"native".equals(args[0])) {
            throw new IllegalArgumentException("expected exact Filesystem descriptor cases");
        }
        for (int i = 1; i < args.length; i += 3) {
            observe(new Pinned(args[i], Path.of(args[i + 1]), args[i + 2]), args[i]);
        }
        row("terminal", "direct-descriptor-only", "no-collection-or-binary-resolution");
    }
}
