// SPDX-License-Identifier: EPL-2.0
//! Combined generation/role parsing/restore; fixture setup and checks are untimed.
use std::time::Instant;
use syson_domain::{identity_context::*, uuid::Uuid};
struct Fresh(u32);
impl FreshIdProvider for Fresh {
    type Error = ();
    fn next_v4(&mut self) -> Result<Uuid, ()> {
        self.0 += 1;
        let mut bytes = [0; 16];
        bytes[6] = 0x40;
        bytes[8] = 0x80;
        bytes[12..].copy_from_slice(&self.0.to_be_bytes());
        Ok(Uuid(bytes))
    }
}
fn main() {
    let ordinary = ElementContext {
        class: ElementClass::Other,
        resource: ResourceObservation::Unresolved,
        library_namespace: LibraryNamespaceObservation::Absent,
        declared_name: Observed::Unresolved,
        qualified_name: Observed::Unresolved,
    };
    for count in [256u32, 1024, 4096] {
        let expected: Vec<_> = (1..=count)
            .map(|n| {
                (
                    format!("10000000-0000-4000-8000-{n:012x}"),
                    format!("00000000-0000-4000-8000-{n:012x}"),
                )
            })
            .collect();
        let mut raw = Vec::new();
        for repetition in 0..9 {
            let mut provider = Fresh(0);
            let started = Instant::now();
            let pairs: Vec<_> = (1..=count)
                .map(|n| {
                    let id = generate_semantic_id(Some(ordinary), &mut provider)
                        .unwrap()
                        .id
                        .0;
                    (format!("10000000-0000-4000-8000-{n:012x}"), id.to_string())
                })
                .collect();
            let registry = SelectedIdentityRegistry::restore(
                pairs
                    .iter()
                    .map(|(r, s)| SelectedIdentity::from_fields(Some(r), Some(s)).unwrap()),
            )
            .unwrap();
            let elapsed = started.elapsed().as_nanos();
            assert_eq!(pairs, expected);
            assert_eq!((provider.0, registry.len()), (count, count as usize));
            for (resource, semantic) in &expected {
                let pair = SelectedIdentity::from_fields(Some(resource), Some(semantic)).unwrap();
                assert!(registry.contains_resource(pair.resource()));
                assert!(registry.contains_semantic(pair.semantic()));
            }
            if repetition >= 2 {
                raw.push(elapsed);
            }
        }
        let mut sorted = raw.clone();
        sorted.sort_unstable();
        println!(
            "{{\"pairs\":{count},\"warmups\":2,\"samples\":7,\"raw_ns\":{raw:?},\"min_ns\":{},\"median_ns\":{},\"max_ns\":{}}}",
            sorted[0], sorted[3], sorted[6]
        );
    }
}
