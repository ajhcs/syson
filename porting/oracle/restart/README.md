# SysON reference project after a Java restart

On 17 November 2026, the official SysON 2026.9.3 release reopened an edited project after its Java process shut down and relaunched against the same PostgreSQL instance. A fresh Chromium process reopened the same General View, rendered its package, and loaded the exact saved **Documentation** text. This is an upstream Linux observation for the durable application journey. Rust persistence and correspondence between the official release artifact and the port's pinned source remain separate qualification work.

## What the browser exercised

Chromium created a blank project named `RestartDemo`, then uploaded a SysML document through Explorer. The document defines `Wheel` and a `wheel : Wheel` usage. The browser created a General View, dragged the imported package onto it, and edited the view's **Documentation** field in Details. The GraphQL mutations returned success and both downloads returned HTTP 200 with a ZIP attachment. Both browser sessions ran at 1440 × 960 and recorded zero console errors and uncaught page exceptions.

The harness uses a verified Linux PID descriptor to stop Java and confirms the application port closes while PostgreSQL stays reachable. It then launches the same executable with the original arguments, environment and configuration. PostgreSQL's PID, process start time, data directory identity and port match before shutdown, during the outage, and after reopening. The original and replacement Java PIDs differ. The views showed the same package and Details value. Explorer expansion and canvas framing could differ after reopening.

## What the archives retained

Both ZIPs passed CRC checks. Each contained one manifest, one semantic document and one diagram representation. The semantic document and diagram entries were byte-identical. The manifest parsed to identical JSON despite different key order, and the complete ZIPs had different bytes. The typed relationship retained the same `FeatureTyping` identity, `wheel` usage and `Wheel` definition. The view and rendered package node retained their IDs and object bindings. [Run receipt](release-2026.9.3.json).

The browser recovered the exact **Documentation** value after restart, while that value was absent from both ZIPs. Sirius Web stores the field in representation metadata. The release's project export record includes the representation ID, project ID, description ID, target, label, kind and content. This observation covers the General View's Documentation field on this release. [Details widget](https://github.com/eclipse-sirius/sirius-web/blob/3191e2b8990373afee3415f6b26f8c5039a28321/packages/sirius-web/backend/sirius-web-application/src/main/java/org/eclipse/sirius/web/application/representation/services/RepresentationMetadataDetailsViewPageDescriptionProvider.java#L177-L203), [export record](https://github.com/eclipse-sirius/sirius-web/blob/3191e2b8990373afee3415f6b26f8c5039a28321/packages/sirius-web/backend/sirius-web-application/src/main/java/org/eclipse/sirius/web/application/project/services/RepresentationExportData.java#L22-L30).

## Reproduce on the reference host

Run from the repository root on BuilderBob. This uses the [pinned release launcher](../oracle.py), a Linux user systemd session with cgroup v2 and PID descriptors, and the staged Node and Playwright Chromium. The experiment creates a fresh PostgreSQL data directory and binds both services to loopback. Choose a new, short output directory on the workspace. The launcher creates a fresh user scope with a 210-second deadline, 2 CPU cores, a 3 GiB memory high threshold, a 4 GiB memory limit, and zero swap. PostgreSQL limits the Unix socket path to 107 bytes.

```bash
output=/workspace/build/builderbob/syson-rst-replay
node=/workspace/build/builderbob/runtimes/node-v24.21.0-linux-x64/bin/node
playwright_root=/workspace/cache/builderbob/global-tools/maintenance/staged/playwright/1.64.1
env SYSON_RESTART_PLAYWRIGHT="$playwright_root/node_modules/playwright" \
  PLAYWRIGHT_BROWSERS_PATH="$playwright_root/browsers" \
  SYSON_RESTART_BROWSER_LD_LIBRARY_PATH=/workspace/cache/builderbob/global-tools/browser-deps/sysroot/usr/lib/x86_64-linux-gnu \
  bash porting/oracle/restart/run.sh "$output" "$node"
```

A zero exit requires the result, removal of the runtime state file, and absence of the owned scope. The launcher records the scope invocation ID outside the output and uses it to stop an active scope after failure or interruption. The run retains screenshots, DOM observations, both original ZIPs, process and resource evidence, and failure logs in the output directory. The measured memory peak belongs to this synthetic workload and host.
