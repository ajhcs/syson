import java.lang.reflect.Proxy;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.Collection;
import java.util.List;

import org.apache.maven.repository.internal.DefaultVersionRangeResolver;
import org.apache.maven.repository.internal.MavenRepositorySystemUtils;
import org.eclipse.aether.RepositorySystemSession;
import org.eclipse.aether.SyncContext;
import org.eclipse.aether.artifact.Artifact;
import org.eclipse.aether.artifact.DefaultArtifact;
import org.eclipse.aether.metadata.Metadata;
import org.eclipse.aether.repository.LocalRepository;
import org.eclipse.aether.repository.LocalRepositoryManager;
import org.eclipse.aether.repository.RemoteRepository;
import org.eclipse.aether.repository.RepositoryPolicy;
import org.eclipse.aether.resolution.MetadataRequest;
import org.eclipse.aether.resolution.MetadataResult;
import org.eclipse.aether.resolution.VersionRangeRequest;

final class FilesystemOffers {
    private static final String GA = "org.eclipse.platform:org.eclipse.core.filesystem";
    private static final String RANGE = "[1.7.0,2.0.0)";
    private static final RemoteRepository CENTRAL = new RemoteRepository.Builder(
        "central", "default", "https://repo.maven.apache.org/maven2")
        .setReleasePolicy(new RepositoryPolicy(true, "daily", "warn"))
        .setSnapshotPolicy(new RepositoryPolicy(false, "daily", "warn")).build();
    private static final List<RemoteRepository> REPOSITORIES = List.of(CENTRAL);

    private final Path pinned;
    private int callbacks;

    private FilesystemOffers(Path pinned) {
        this.pinned = pinned;
    }

    private List<MetadataResult> metadata(RepositorySystemSession session,
            Collection<? extends MetadataRequest> requests) {
        require(++callbacks == 1 && session.isOffline() && requests.size() == 2, "metadata scope drift");
        var results = new ArrayList<MetadataResult>();
        Metadata.Nature nature = null;
        for (var request : requests) {
            var item = request.getMetadata();
            require(GA.equals(item.getGroupId() + ":" + item.getArtifactId())
                && item.getVersion().isEmpty() && "maven-metadata.xml".equals(item.getType())
                && "project".equals(request.getRequestContext()) && !request.isFavorLocalRepository()
                && (item.getNature() == Metadata.Nature.RELEASE || item.getNature() == Metadata.Nature.RELEASE_OR_SNAPSHOT),
                "metadata request identity drift");
            var repo = request.getRepository();
            var response = new MetadataResult(request);
            if (results.isEmpty()) {
                require(repo == null && !request.isDeleteLocalCopyIfMissing(), "local metadata flags drifted");
                nature = item.getNature();
                row("metadata", "local", nature.name(), "absent");
            } else {
                require(CENTRAL.equals(repo) && exactPolicy(repo.getPolicy(false), true)
                    && exactPolicy(repo.getPolicy(true), false) && request.isDeleteLocalCopyIfMissing()
                    && nature == item.getNature(), "Central metadata flags, policy, or order drifted");
                row("metadata", "central", nature.name(), pinned == null ? "absent" : "pinned");
                if (pinned != null) {
                    response.setMetadata(item.setFile(pinned.toFile()));
                }
            }
            results.add(response);
        }
        return results;
    }

    private void resolve(String arm) throws Exception {
        var session = MavenRepositorySystemUtils.newSession().setOffline(true);
        var local = new LocalRepository(Path.of(System.getProperty("user.home"), "uncreated").toFile());
        session.setLocalRepositoryManager((LocalRepositoryManager) Proxy.newProxyInstance(
            LocalRepositoryManager.class.getClassLoader(), new Class<?>[]{LocalRepositoryManager.class},
            (receiver, method, args) -> {
                require("getRepository".equals(method.getName()), "ambient repository access refused");
                return local;
            }));
        var resolver = new DefaultVersionRangeResolver(this::metadata,
            (offline, shared) -> new SyncContext() {
                public void acquire(Collection<? extends Artifact> artifacts, Collection<? extends Metadata> records) {}
                public void close() {}
            },
            event -> { throw new IllegalStateException("repository event outside scope: " + event.getType()); });
        row("case", arm, GA, RANGE);
        row("tools", Runtime.version().toString(),
            DefaultVersionRangeResolver.class.getPackage().getImplementationVersion());
        var request = new VersionRangeRequest(new DefaultArtifact(GA + ":" + RANGE), REPOSITORIES, "project");
        var result = resolver.resolveVersionRange(session, request);
        require(callbacks == 1 && result.getExceptions().isEmpty() && result.getVersions().size() <= 70
            && result.getVersions().isEmpty() == (pinned == null), "unexpected metadata offer result");
        for (var offered : result.getVersions()) {
            require(CENTRAL.equals(result.getRepository(offered)), "offered version origin drift");
            row("offer", offered.toString(), "central");
        }
        row("offers", Integer.toString(result.getVersions().size()));
        row("terminal", "offers-only", "no-descriptor-or-binary-resolution");
    }

    private static boolean exactPolicy(RepositoryPolicy policy, boolean enabled) {
        return policy.isEnabled() == enabled && "daily".equals(policy.getUpdatePolicy())
            && "warn".equals(policy.getChecksumPolicy());
    }

    private static void require(boolean ok, String reason) {
        if (!ok) {
            throw new IllegalStateException(reason);
        }
    }

    private static void row(String... fields) {
        System.out.println(String.join("\t", fields));
    }

    public static void main(String[] args) throws Exception {
        require((args.length == 2 && "pinned".equals(args[0]))
            || (args.length == 1 && "withheld".equals(args[0])), "unexpected replay arm");
        Path pinned = null;
        if ("pinned".equals(args[0])) {
            pinned = Path.of(args[1]);
            require(pinned.isAbsolute() && pinned.toRealPath().equals(pinned)
                && Files.isRegularFile(pinned) && Files.size(pinned) == 1188, "captured primary differs");
        }
        new FilesystemOffers(pinned).resolve(args[0]);
    }
}
