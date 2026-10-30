// SPDX-License-Identifier: EPL-2.0
use serde_json::{Value, json};
use std::collections::HashMap;
use syson_migration_pipeline::{
    Error, Project, Representation, Resolver, SemanticDocument, prepare,
};
use syson_model::Document;
use syson_persistence::{DIAGRAM_KIND, DIAGRAM_VERSION, REFERENCES_VERSION, ResolvedElement};

#[derive(Default)]
struct Snapshot {
    hits: HashMap<String, ResolvedElement>,
    fail: Option<String>,
    calls: Vec<String>,
}
impl Resolver for Snapshot {
    fn resolve(&mut self, id: &str) -> Result<Option<ResolvedElement>, String> {
        self.calls.push(id.into());
        if self.fail.as_deref() == Some(id) {
            Err("snapshot unavailable".into())
        } else {
            Ok(self.hits.get(id).cloned())
        }
    }
}
fn node(id: &str, class: &str, data: Value) -> Value {
    let mut data = data.as_object().unwrap().clone();
    data.insert("elementId".into(), json!(format!("semantic-{id}")));
    json!({"id":id,"eClass":format!("sysml:{class}"),"data":data})
}
fn admitted(content: Value) -> SemanticDocument {
    let value = json!({"json":{"version":"1.0","encoding":"utf-8"},
        "ns":{"sysml":"http://www.eclipse.org/syson/sysml"},"content":content});
    SemanticDocument {
        document: Document::from_json(&value.to_string()).unwrap(),
        references_version: None,
    }
}
fn rep(content: Value) -> Representation {
    Representation {
        kind: DIAGRAM_KIND.into(),
        content,
        diagram_version: None,
    }
}
fn map() -> HashMap<String, String> {
    [
        ("old".into(), "new".into()),
        ("new".into(), "wrong-on-replay".into()),
    ]
    .into()
}
fn typed(uri: &str) -> SemanticDocument {
    admitted(json!([node(
        "typing",
        "FeatureTyping",
        json!({"type":uri})
    )]))
}
fn element(id: &str, uri: &str) -> ResolvedElement {
    ResolvedElement {
        element_id: id.into(),
        resource_uri: Some(uri.into()),
    }
}
#[test]
fn nested_semantic_and_diagram_migrations_preserve_nonreferences_and_replay() {
    let typing = node(
        "typing",
        "FeatureTyping",
        json!({"type":"sysml:DataType sysmllibrary:/Scalar#old"}),
    );
    let member = node(
        "member",
        "OwningMembership",
        json!({"ownedRelatedElement":[typing]}),
    );
    let input = Project {
        documents: vec![admitted(json!([node(
            "package",
            "Package",
            json!({
            "declaredName":"sysmllibrary:/Scalar#old","ownedRelationship":[member]})
        )]))],
        representations: vec![rep(
            json!({"nodes":[{"children":[{"targetObjectId":"old"}],"label":"old"}],
            "numeric":{"targetObjectId":7},"targets":[{"targetObjectId":"user"},{"targetObjectId":"live"}]}),
        )],
    };
    let before = input.clone();
    let mut resolver = Snapshot::default();
    resolver
        .hits
        .insert("user".into(), element("must-stay-user", "memory:/project"));
    resolver.hits.insert(
        "live".into(),
        element("stable-live", "kermllibrary:/Actions"),
    );
    let mut ids = map();
    ids.insert("user".into(), "must-not-fallback".into());
    let output = prepare(&input, &ids, &mut resolver).unwrap();
    assert_eq!(input, before);
    let data = &output.documents[0].document.value()["content"][0]["data"];
    assert_eq!(data["declaredName"], "sysmllibrary:/Scalar#old");
    assert_eq!(
        data["ownedRelationship"][0]["data"]["ownedRelatedElement"][0]["data"]["type"],
        "sysml:DataType sysmllibrary:/Scalar#new"
    );
    assert_eq!(
        output.representations[0].content["nodes"][0]["children"][0]["targetObjectId"],
        "new"
    );
    assert_eq!(
        output.representations[0].content["nodes"][0]["label"],
        "old"
    );
    assert_eq!(
        output.representations[0].content["numeric"]["targetObjectId"],
        7
    );
    assert_eq!(
        output.representations[0].content["targets"][0]["targetObjectId"],
        "user"
    );
    assert_eq!(
        output.representations[0].content["targets"][1]["targetObjectId"],
        "stable-live"
    );
    assert_eq!(
        output.documents[0].references_version.as_deref(),
        Some(REFERENCES_VERSION)
    );
    assert_eq!(
        output.representations[0].diagram_version.as_deref(),
        Some(DIAGRAM_VERSION)
    );
    resolver.calls.clear();
    assert_eq!(prepare(&output, &ids, &mut resolver).unwrap(), output);
    assert!(resolver.calls.is_empty());
}
#[test]
fn all_semantic_documents_precede_diagrams_and_resolved_ids_win() {
    let input = Project {
        documents: vec![
            typed("sysml:Type sysmllibrary:/First#old"),
            typed("kermllibrary:/Second#old"),
        ],
        representations: vec![rep(json!({"targetObjectId":"old"}))],
    };
    let mut resolver = Snapshot::default();
    resolver.hits.insert(
        "sysmllibrary:/First#old".into(),
        element("resolved", "memory:/nonlibrary"),
    );
    let output = prepare(&input, &map(), &mut resolver).unwrap();
    assert_eq!(
        resolver.calls,
        vec!["sysmllibrary:/First#old", "kermllibrary:/Second#old", "old"]
    );
    assert_eq!(
        output.documents[0].document.value()["content"][0]["data"]["type"],
        "sysml:Type sysmllibrary:/First#resolved"
    );
}
#[test]
fn resolver_failure_after_semantic_work_returns_no_partial_project() {
    let input = Project {
        documents: vec![typed("sysmllibrary:/A#old")],
        representations: vec![rep(json!({"targetObjectId":"old"}))],
    };
    let before = input.clone();
    let mut resolver = Snapshot {
        fail: Some("old".into()),
        ..Default::default()
    };
    assert_eq!(
        prepare(&input, &map(), &mut resolver),
        Err(Error::Resolver("snapshot unavailable".into()))
    );
    assert_eq!(input, before);
    resolver.fail = None;
    assert_eq!(
        prepare(&input, &map(), &mut resolver)
            .unwrap()
            .representations[0]
            .content["targetObjectId"],
        "new"
    );
}
#[test]
fn wrong_kind_and_nonlibrary_reference_do_not_call_resolver() {
    let input = Project {
        documents: vec![typed("memory:/project#old")],
        representations: vec![Representation {
            kind: "siriusComponents://representation?type=Table".into(),
            content: json!({"targetObjectId":"old"}),
            diagram_version: Some("unrelated-version".into()),
        }],
    };
    let mut resolver = Snapshot::default();
    let output = prepare(&input, &map(), &mut resolver).unwrap();
    assert!(resolver.calls.is_empty());
    assert_eq!(output.representations, input.representations);
    assert_eq!(output.documents[0].document, input.documents[0].document);
}
#[test]
fn unsupported_watermarks_refuse_before_resolution() {
    for semantic in [true, false] {
        let mut input = Project {
            documents: vec![typed("sysmllibrary:/A#old")],
            representations: vec![rep(json!({"targetObjectId":"old"}))],
        };
        if semantic {
            input.documents[0].references_version = Some("2099-future".into());
        } else {
            input.representations[0].diagram_version = Some("2099-future".into());
        }
        let before = input.clone();
        let mut resolver = Snapshot::default();
        assert_eq!(
            prepare(&input, &map(), &mut resolver),
            Err(Error::UnsupportedVersion("2099-future".into()))
        );
        assert!(resolver.calls.is_empty());
        assert_eq!(input, before);
    }
}
#[test]
fn unknown_library_features_and_invalid_reference_shape_refuse_atomically() {
    let cases = [
        (
            json!({"futureReference":"sysmllibrary:/A#old"}),
            Error::Unclassified("futureReference".into()),
        ),
        (
            json!({"type":{"reference":"sysmllibrary:/A#old"}}),
            Error::ReferenceShape("type".into()),
        ),
    ];
    for (data, expected) in cases {
        let input = Project {
            documents: vec![admitted(json!([node("x", "FeatureTyping", data)]))],
            representations: vec![],
        };
        let before = input.clone();
        assert_eq!(
            prepare(&input, &map(), &mut Snapshot::default()),
            Err(expected)
        );
        assert_eq!(input, before);
    }
}
#[test]
fn real_document_admission_rejects_duplicate_ids_before_pipeline() {
    let mut value = Document::empty().value().clone();
    value["content"] = json!([
        node("same", "Package", json!({})),
        node("same", "Package", json!({}))
    ]);
    assert!(Document::from_json(&value.to_string()).is_err());
}
#[test]
fn item_document_aggregate_depth_and_expanding_output_budgets() {
    let mut resolver = Snapshot::default();
    let too_many = Project {
        documents: vec![
            SemanticDocument {
                document: Document::empty(),
                references_version: None
            };
            65
        ],
        representations: vec![],
    };
    assert_eq!(prepare(&too_many, &map(), &mut resolver), Err(Error::Limit));
    let large = rep(json!({"padding":"x".repeat(1024*1024)}));
    assert_eq!(
        prepare(
            &Project {
                documents: vec![],
                representations: vec![large]
            },
            &map(),
            &mut resolver
        ),
        Err(Error::Limit)
    );
    let bounded = rep(json!({"padding":"x".repeat(1024*1024-1024)}));
    assert!(
        prepare(
            &Project {
                documents: vec![],
                representations: vec![bounded.clone(); 8]
            },
            &map(),
            &mut resolver
        )
        .is_ok()
    );
    assert_eq!(
        prepare(
            &Project {
                documents: vec![],
                representations: vec![bounded; 9]
            },
            &map(),
            &mut resolver
        ),
        Err(Error::Limit)
    );
    let mut deep = json!(0);
    for _ in 0..65 {
        deep = json!([deep]);
    }
    assert_eq!(
        prepare(
            &Project {
                documents: vec![],
                representations: vec![rep(deep)]
            },
            &map(),
            &mut resolver
        ),
        Err(Error::Limit)
    );
    let input = Project {
        documents: vec![],
        representations: vec![rep(json!({"targetObjectId":"old"}))],
    };
    let before = input.clone();
    let ids = [("old".into(), "x".repeat(1024 * 1024))].into();
    assert_eq!(prepare(&input, &ids, &mut resolver), Err(Error::Limit));
    assert_eq!(input, before);
}

#[test]
fn native_sql_fixture_prepares_the_two_ids_asserted_by_native_integration_test() {
    let document = Document::from_json(include_str!("fixtures/native-document.json")).unwrap();
    let content: Value =
        serde_json::from_str(include_str!("fixtures/native-diagram.json")).unwrap();
    let input = Project {
        documents: vec![SemanticDocument {
            document,
            references_version: None,
        }],
        representations: vec![rep(content)],
    };
    let ids = syson_persistence::parse_id_map(include_str!("fixtures/native-id-map.txt"));
    let output = prepare(&input, &ids, &mut Snapshot::default()).unwrap();
    let typing = output.documents[0]
        .document
        .element("b2850b04-3f1f-4799-8b44-f0706f22628b")
        .unwrap();
    assert!(
        typing["data"]["type"]
            .as_str()
            .unwrap()
            .ends_with("#76028d3d-69a4-5e12-9002-ce403e0244bd")
    );
    fn targets(value: &Value, id: &str) -> usize {
        match value {
            Value::Object(o) => {
                usize::from(o.get("targetObjectId").and_then(Value::as_str) == Some(id))
                    + o.values().map(|v| targets(v, id)).sum::<usize>()
            }
            Value::Array(a) => a.iter().map(|v| targets(v, id)).sum(),
            _ => 0,
        }
    }
    assert_eq!(
        targets(
            &input.representations[0].content,
            "03262155-734a-473f-9535-262925cd5b8b"
        ),
        1
    );
    assert_eq!(
        targets(
            &output.representations[0].content,
            "03262155-734a-473f-9535-262925cd5b8b"
        ),
        0
    );
    assert_eq!(
        targets(
            &output.representations[0].content,
            "9a0d2905-0f9c-5bb4-af74-9780d6db1817"
        ),
        1
    );
}
#[test]
fn many_expansions_stop_before_per_document_and_project_growth_exhaustion() {
    let input = Project {
        documents: vec![],
        representations: vec![rep(json!({"nodes":
        (0..64).map(|_|json!({"targetObjectId":"old"})).collect::<Vec<_>>()}))],
    };
    let before = input.clone();
    let ids = [("old".into(), "x".repeat(32 * 1024))].into();
    let mut resolver = Snapshot::default();
    assert_eq!(prepare(&input, &ids, &mut resolver), Err(Error::Limit));
    assert!(resolver.calls.len() < 64);
    assert_eq!(input, before);
    let input = Project {
        documents: vec![],
        representations: vec![rep(json!({"targetObjectId":"old"})); 12],
    };
    let mut resolver = Snapshot::default();
    let ids = [("old".into(), "x".repeat(900 * 1024))].into();
    assert_eq!(prepare(&input, &ids, &mut resolver), Err(Error::Limit));
    assert!(resolver.calls.len() < 12);
}
#[test]
fn oversized_resolver_id_refuses_without_formatting_or_mutating_input() {
    let input = Project {
        documents: vec![typed("sysmllibrary:/A#old")],
        representations: vec![],
    };
    let before = input.clone();
    let mut resolver = Snapshot::default();
    resolver.hits.insert(
        "sysmllibrary:/A#old".into(),
        element(&"x".repeat(2 * 1024 * 1024), "memory:/x"),
    );
    assert_eq!(prepare(&input, &map(), &mut resolver), Err(Error::Limit));
    assert_eq!(input, before);
}

#[test]
fn literal_metadata_and_numbers_survive_migration_reparse_and_replay() {
    let document = Document::from_json(
        r#"{"json":{"version":"1.0","encoding":"utf-8"},
        "ns":{"sysml":"http://www.eclipse.org/syson/sysml"},
        "opaque":{"$serde_json::private::Number":"not a number",
        "$serde_json::private::RawValue":"{not JSON}",
        "numbers":[1E+400,-0,123456789012345678901234567890,1.234567890123456789]},
        "content":[{"id":"typing","eClass":"sysml:FeatureTyping",
        "data":{"elementId":"semantic-typing","type":"sysmllibrary:/Literal#old"}}]}"#,
    )
    .unwrap();
    let opaque = document.value()["opaque"].clone();
    assert_eq!(opaque["$serde_json::private::Number"], "not a number");
    assert_eq!(opaque["$serde_json::private::RawValue"], "{not JSON}");
    let mut representation = json!({"targetObjectId":"old"});
    let fields = representation.as_object_mut().unwrap();
    fields.insert("opaque".into(), opaque.clone());
    assert_eq!(fields["opaque"], opaque);
    let input = Project {
        documents: vec![SemanticDocument {
            document,
            references_version: None,
        }],
        representations: vec![rep(representation)],
    };
    let before = input.clone();
    let mut resolver = Snapshot::default();
    let output = prepare(&input, &map(), &mut resolver).unwrap();
    assert_eq!(input, before);
    let migrated = &output.documents[0].document;
    assert_eq!(migrated.value()["opaque"], opaque);
    assert_eq!(output.representations[0].content["opaque"], opaque);
    assert_eq!(
        migrated.element("semantic-typing").unwrap()["data"]["type"],
        "sysmllibrary:/Literal#new"
    );
    assert_eq!(output.representations[0].content["targetObjectId"], "new");
    assert_eq!(
        opaque["numbers"].to_string(),
        "[1E+400,-0,123456789012345678901234567890,1.234567890123456789]"
    );
    let reopened = Document::from_json(&migrated.to_json().unwrap()).unwrap();
    assert_eq!(reopened, *migrated);
    assert_eq!(reopened.value()["opaque"], opaque);
    resolver.calls.clear();
    assert_eq!(prepare(&output, &map(), &mut resolver).unwrap(), output);
    assert!(resolver.calls.is_empty());
}
