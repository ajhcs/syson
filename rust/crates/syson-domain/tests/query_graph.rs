// SPDX-License-Identifier: EPL-2.0
// Synthetic cases for the pinned ElementUtil query order and lazy fact boundary.
use syson_domain::{
    model::ElementId,
    names::name_matches,
    query::{QueryGraph, find_by_name_and_type},
};

#[derive(Default)]
struct Node {
    name: Option<&'static str>,
    short: Option<&'static str>,
    namespace: bool,
    container: bool,
    raw: Vec<ElementId>,
    owned: Vec<ElementId>,
    imports: Vec<ElementId>,
    unavailable_name: bool,
    unavailable_imports: bool,
}
impl Node {
    fn named(name: &'static str) -> Self {
        Self {
            name: Some(name),
            ..Self::default()
        }
    }
}
struct Graph(Vec<Node>);
impl Graph {
    fn node(&self, id: ElementId) -> Result<&Node, &'static str> {
        self.0.get(id.0).ok_or("unknown object")
    }
}
impl QueryGraph for Graph {
    type Error = &'static str;

    fn raw_children(&self, id: ElementId) -> Result<Vec<ElementId>, Self::Error> {
        Ok(self.node(id)?.raw.clone())
    }
    fn owned_elements(&self, id: ElementId) -> Result<Vec<ElementId>, Self::Error> {
        Ok(self.node(id)?.owned.clone())
    }
    fn is_namespace(&self, id: ElementId) -> Result<bool, Self::Error> {
        Ok(self.node(id)?.namespace)
    }
    fn has_raw_container(&self, id: ElementId) -> Result<bool, Self::Error> {
        Ok(self.node(id)?.container)
    }
    fn is_unnamed(&self, id: ElementId) -> Result<bool, Self::Error> {
        let node = self.node(id)?;
        if node.unavailable_name {
            return Err("effective name unresolved");
        }
        Ok(node.name.is_none())
    }
    fn name_matches(&self, id: ElementId, query: &str) -> Result<bool, Self::Error> {
        let node = self.node(id)?;
        if node.unavailable_name {
            return Err("effective name unresolved");
        }
        Ok(name_matches(node.name, Some(query)) || name_matches(node.short, Some(query)))
    }
    fn public_imported_elements(&self, id: ElementId) -> Result<Vec<ElementId>, Self::Error> {
        let node = self.node(id)?;
        if node.unavailable_imports {
            return Err("native imports unbound");
        }
        Ok(node.imports.clone())
    }
}
fn id(value: usize) -> ElementId {
    ElementId(value)
}

#[test]
fn unqualified_raw_preorder_and_qualified_semantic_children_are_independent() {
    let graph = Graph(vec![
        Node {
            raw: vec![id(1)],
            owned: vec![id(2)],
            ..Node::named("Root")
        },
        Node {
            raw: vec![id(2)],
            container: true,
            ..Node::named("target")
        },
        Node {
            container: true,
            ..Node::named("target")
        },
    ]);
    assert_eq!(
        find_by_name_and_type(&graph, &[id(0)], "target", |_| Ok(true)),
        Ok(Some(id(1)))
    );
    assert_eq!(
        find_by_name_and_type(&graph, &[id(0)], "Root::target", |_| Ok(true)),
        Ok(Some(id(2)))
    );
}

#[test]
fn unqualified_type_filter_does_not_force_unused_effective_name_getters() {
    let graph = Graph(vec![
        Node {
            raw: vec![id(1), id(2), id(3)],
            ..Node::named("Root")
        },
        Node {
            unavailable_name: true,
            ..Node::default()
        },
        Node::named("target"),
        Node {
            unavailable_name: true,
            ..Node::default()
        },
    ]);
    assert_eq!(
        find_by_name_and_type(&graph, &[id(0)], "target", |value| Ok(value == id(2))),
        Ok(Some(id(2)))
    );
}

#[test]
fn first_owned_name_blocks_imports_and_later_same_name_even_if_type_is_wrong() {
    let graph = Graph(vec![
        Node {
            namespace: true,
            owned: vec![id(1), id(2)],
            unavailable_imports: true,
            ..Node::named("Root")
        },
        Node::named("target"),
        Node::named("target"),
    ]);
    assert_eq!(
        find_by_name_and_type(&graph, &[id(0)], "Root::target", |value| Ok(value == id(2))),
        Ok(None)
    );
}

#[test]
fn first_owned_path_blocks_import_fallback_when_deeper_segment_is_missing() {
    let graph = Graph(vec![
        Node {
            namespace: true,
            owned: vec![id(1)],
            unavailable_imports: true,
            ..Node::named("Root")
        },
        Node::named("branch"),
    ]);
    assert_eq!(
        find_by_name_and_type(&graph, &[id(0)], "Root::branch::target", |_| Ok(true)),
        Ok(None)
    );
}

#[test]
fn unnamed_namespace_uses_actual_container_and_effective_name_null() {
    let make = |container| {
        Graph(vec![
            Node {
                namespace: true,
                container,
                short: Some("short"),
                owned: vec![id(1)],
                ..Node::default()
            },
            Node {
                owned: vec![id(2)],
                ..Node::named("Branch")
            },
            Node::named("target"),
        ])
    };
    assert_eq!(
        find_by_name_and_type(&make(false), &[id(0)], "Branch::target", |_| Ok(true)),
        Ok(Some(id(2)))
    );
    assert_eq!(
        find_by_name_and_type(&make(true), &[id(0)], "Branch::target", |_| Ok(true)),
        Ok(None)
    );
}

#[test]
fn imports_are_evaluated_only_after_owned_miss_and_names_stop_at_first_match() {
    let mut graph = Graph(vec![
        Node {
            namespace: true,
            imports: vec![id(1), id(2)],
            ..Node::named("Root")
        },
        Node::named("target"),
        Node {
            unavailable_name: true,
            ..Node::default()
        },
    ]);
    assert_eq!(
        find_by_name_and_type(&graph, &[id(0)], "Root::target", |_| Ok(true)),
        Ok(Some(id(1)))
    );
    graph.0[0].unavailable_imports = true;
    assert_eq!(
        find_by_name_and_type(&graph, &[id(0)], "Root::target", |_| Ok(true)),
        Err("native imports unbound")
    );
}

#[test]
fn supplied_root_order_and_empty_resource_set_remain_observable() {
    let graph = Graph(vec![Node::named("target"), Node::named("target")]);
    assert_eq!(
        find_by_name_and_type(&graph, &[id(1), id(0)], "target", |_| Ok(true)),
        Ok(Some(id(1)))
    );
    assert_eq!(
        find_by_name_and_type(&graph, &[], "target", |_| Err("must stay lazy")),
        Ok(None)
    );
}

#[test]
fn qualified_final_type_check_preserves_the_adapters_error_type() {
    let graph = Graph(vec![
        Node {
            owned: vec![id(1)],
            ..Node::named("Root")
        },
        Node::named("target"),
    ]);
    assert_eq!(
        find_by_name_and_type(&graph, &[id(0)], "Root::target", |_| Err(
            "type fact unresolved"
        )),
        Err("type fact unresolved")
    );
}
