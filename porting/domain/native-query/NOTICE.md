# Source attribution for the native query adapter

The Rust query engine and selected JSON adapter derive the named methods from eclipse-syson/syson at `6ba2ac64e6b16ee2b05752ad55395eebb87d4910`. Original source files, LICENSE, NOTICE and their per-file notices remain in the repository. No original copyright or license is replaced by the Rust adapter's compact header. Exact source paths and hashes are in PROVENANCE.json.

- ElementUtil.java: Copyright (c) 2024, 2026 Obeo.
- ElementImpl.java, FeatureImpl.java and NamespaceImpl.java: Copyright (c) 2023, 2026 Obeo.
- RelationshipImpl.java and ConjugatedPortDefinitionImpl.java: Copyright (c) 2023, 2025 Obeo.

The original source notices state:

> This program and the accompanying materials are made available under the terms of the Eclipse Public License v2.0 which accompanies this distribution, and is available at https://www.eclipse.org/legal/epl-2.0/
>
> SPDX-License-Identifier: EPL-2.0
>
> Contributors: Obeo - initial API and implementation

The Rust port contributors supply the lazy query-fact interface, immutable resource snapshots, explicit observation/reference boundaries and synthetic tests. This attribution records these selected sources; it does not close the wider port's artifact/dependency license inventory.
