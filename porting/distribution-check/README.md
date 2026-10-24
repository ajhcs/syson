# Rust/WASM distribution content checks

`check.py` verifies the selected default or `raw-factory-candidate` source-bundle layout against an explicit reviewed policy and captured Cargo metadata. It never invokes Cargo, resolves a registry, downloads or extracts an archive, builds WASM, or claims legal clearance. It checks actual files in the supplied package; source-tree license files alone cannot satisfy package inclusion.

The policy pins integration source `ad814c232ee24109639c1d52239cbeec6f4a3e92`, all existing Rust crate source/manifest/lock bytes, root SysON notices, five admitted registry crate manifests/license texts and locked archive checksums. Metadata captures under `../follow-ons/F43/metadata/` were actually executed by the coordinator with offline/locked Cargo metadata and a `wasm32-unknown-unknown` platform filter, default and candidate features separately. The verifier checks those capture bytes/arguments and source input hashes, then walks only normal resolved edges from `syson-web`. It does not infer active edges from Cargo.lock. Proc-macro/runtime classification and unknown active registry/local dependencies require a new reviewed policy rather than being guessed.

## Required package contents

- `distribution.json`: schema 1, source revision, profile and one declared artifact hash/synthetic flag.
- `web/syson_web.wasm` plus exact pinned `web/{index.html,loader.js,style.css}`.
- Actual root `LICENSE`, `LICENSE-LGPL`, `NOTICE`, Rust dependency notices and manifest.
- `source/SOURCE-REVISION` plus all pinned files for every active local crate, including its manifests and lockfile.
- Actual license files from `rust/third-party` for all active registry crates.
- `dependency-manifests/<crate-version>/Cargo.toml` matching supplied source-manifest hashes.
- `dependency-archives/<crate-version>.crate` matching Cargo.lock SHA256 checksums.

Unlisted package payloads, symlinks, missing files and mismatched source/lock/metadata/license/archive bytes are rejected. The source tree may contain normal generated target directories, but every other file in an active local crate must appear in the pinned policy. Policies and capture inputs are trust inputs requiring review; a hash agreement is not authentication of the person who supplied them.

The fixed-source `prepare_policy.py` reproduces policy and manifest fixture bytes from the pinned Git objects, immutable capture and an explicit existing registry-source directory. It never resolves dependencies or executes Cargo; changing the source/feature/target policy still requires review.

## Use

```sh
python3 -B porting/distribution-check/check.py \
  --source-root "$SYSON_SOURCE" --package-root "$SYSON_PACKAGE" \
  --capture porting/follow-ons/F43/metadata/capture.json \
  --policy porting/distribution-check/policy.json --profile default
```

`make_fixture.py` creates a fresh explicitly synthetic package using the pinned source and the already-existing admitted registry archives supplied through `--registry-cache`. It refuses an existing output. It copies, hashes and retains archives without extracting/executing them. Its eight-byte WASM module is a synthetic test input, not a compiled application. No binary archive fixtures are committed; the five exact Cargo manifest fixtures are retained with their supplied bytes and license/source provenance. The caller supplies archive inputs; no cache/download fallback occurs.

The stdlib tests need `SYSON_TEST_DIR` pointing to an existing admitted workspace directory and `SYSON_REGISTRY_CACHE` pointing to the existing Cargo registry/cache directory. Test-only temporary packages are owned and cleaned by their TemporaryDirectory; original inputs are never modified.

```sh
PYTHONDONTWRITEBYTECODE=1 python3 -B -m unittest discover \
  -s porting/distribution-check -p test_check.py -v
```

## Limits

A successful result means only `SUPPLIED_PACKAGE_AND_CAPTURE_MATCH_POLICY`. The declared artifact hash agrees with supplied artifact bytes; it does not prove that WASM was built from those sources. Native loader/rendering behavior, all build-time/procedural generation effects, actual registry-to-artifact provenance, EPL/license compliance, standard-library distribution obligations, inherited Sirius/Syside provenance and full upstream legal clearance remain separate unresolved questions. `full_legal_clearance` is explicitly `NOT_ESTABLISHED`; no product parity or installed-client acceptance is implied. No new dependency, package baseline, credential, service or runtime is authorized by this tooling.
