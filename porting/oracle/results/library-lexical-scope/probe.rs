// SPDX-License-Identifier: EPL-2.0
use serde_json::{Value, json};
use std::fs;
use syson_domain::model::{Element, ImportedMembership, Model, Visibility};
use syson_domain::query::{self, QueryGraph};
use syson_metamodel::ClassifierId;
use syson_model::Document;
use syson_references::{FragmentIdentity, Registry, Resource};
use syson_syntax::{CstKind, Grammar, Language, Options, Recognition};

type Range = (usize, usize);
fn contains((a, b): Range, (c, d): Range) -> bool {
    a <= c && d <= b
}
fn spans(tree: &Recognition, name: &str) -> Vec<Range> {
    let mut result: Vec<_> = tree
        .cst
        .iter()
        .filter(|n| {
            matches!(&n.kind,
        CstKind::Rule(r)|CstKind::RuleCall(r) if r==name)
        })
        .map(|n| (n.span.start.byte, n.span.end.byte))
        .collect();
    result.sort_unstable();
    result.dedup();
    result
}
fn one(tree: &Recognition, name: &str, outer: Range) -> Range {
    let found: Vec<_> = spans(tree, name)
        .into_iter()
        .filter(|s| contains(outer, *s))
        .collect();
    let [range] = found.as_slice() else {
        panic!("{name}: {found:?}")
    };
    *range
}
fn words<'a>(source: &'a str, tree: &Recognition, range: Range) -> Vec<&'a str> {
    tree.tokens
        .iter()
        .filter(|t| !t.hidden && contains(range, (t.span.start.byte, t.span.end.byte)))
        .map(|t| t.text(source).unwrap())
        .collect()
}
fn qualified(source: &str, tree: &Recognition, range: Range) -> String {
    let words = words(source, tree, range);
    let [a, "::", b] = words.as_slice() else {
        panic!("name: {words:?}")
    };
    format!("{a}::{b}")
}
fn member_ids(node: &Value, target: &str, out: &mut Vec<String>) {
    match node {
        Value::Object(map) => {
            if map.get("eClass").and_then(Value::as_str) == Some("sysml:OwningMembership")
                && node["data"]["ownedRelatedElement"]
                    .as_array()
                    .is_some_and(|children| {
                        children
                            .iter()
                            .any(|child| child["data"]["elementId"] == target)
                    })
            {
                out.push(node["data"]["elementId"].as_str().unwrap().into());
            }
            for child in map.values() {
                member_ids(child, target, out);
            }
        }
        Value::Array(list) => {
            for child in list {
                member_ids(child, target, out);
            }
        }
        _ => (),
    }
}

fn run(source: &str, grammar: &Grammar, document: &Document, key: &str) -> Value {
    let tree = grammar.recognize(source, Options::default()).unwrap();
    let typing = one(&tree, "TypeReference", (0, source.len()));
    let tokens = words(source, &tree, typing);
    let [name] = tokens.as_slice() else {
        panic!("plain unqualified type expected")
    };
    let name = *name;
    let mut projection = document.projection().unwrap();
    let lookup = |m: &Model, n: &str| {
        m.find_by_name_and_type(n, |e| e.class == "DataType")
            .unwrap()
    };
    let mut imports = Vec::new();
    for range in spans(&tree, "Import") {
        let prefix = words(source, &tree, one(&tree, "ImportPrefix", range));
        let visibility = match prefix.as_slice() {
            ["public", "import"] => Visibility::Public,
            ["private", "import"] => Visibility::Private,
            _ => panic!("visibility: {prefix:?}"),
        };
        let reference = qualified(source, &tree, one(&tree, "QualifiedReferenceChain", range));
        let target = lookup(&projection.model, &reference).expect("import target");
        imports.push((range, visibility, reference, target));
    }
    let ranges = spans(&tree, "Package");
    let mut packages = Vec::new();
    for range in ranges.iter().copied() {
        let declaration = spans(&tree, "PackageDeclaration")
            .into_iter()
            .find(|s| contains(range, *s))
            .expect("package declaration");
        let tokens = words(source, &tree, declaration);
        let ["package", name] = tokens.as_slice() else {
            panic!("package: {tokens:?}")
        };
        let mut package = Element::new("Package", Some(name));
        package.is_namespace = true;
        for &(span, visibility, _, target) in &imports {
            if contains(range, span)
                && !ranges
                    .iter()
                    .copied()
                    .any(|other| other != range && contains(range, other) && contains(other, span))
            {
                package.imports.push(ImportedMembership {
                    visibility,
                    related: vec![target],
                });
            }
        }
        let owner = packages
            .iter()
            .rev()
            .find(|(outer, _)| contains(*outer, range))
            .map(|(_, id)| *id);
        packages.push((range, projection.model.insert(package, owner).unwrap()));
    }
    let context = packages
        .iter()
        .rev()
        .find(|(range, _)| contains(*range, typing))
        .expect("type has an owning package")
        .1;
    let model = &projection.model;
    let query_local = query::find_by_name_and_type(model, &[context], name, |id| {
        Ok(model.get(id)?.class == "DataType")
    })
    .unwrap()
    .is_some();
    let query_all = lookup(model, name).is_some();

    let mut scopes = Vec::new();
    let mut scope = Some(context);
    let mut chosen = None;
    while let Some(id) = scope {
        let element = model.get(id).unwrap();
        scopes.push(element.effective_name().unwrap().to_owned());
        chosen = model
            .owned_elements(id)
            .unwrap()
            .into_iter()
            .chain(
                element
                    .imports
                    .iter()
                    .flat_map(|entry| entry.related.iter().copied()),
            )
            .find(|&member| model.name_matches(member, name).unwrap());
        if chosen.is_some() {
            break;
        }
        scope = element.owner();
    }

    let registry = Registry::new(vec![Resource {
        key: key.into(),
        document: document.clone(),
        fragments: FragmentIdentity::SemanticElementId,
    }])
    .unwrap();
    let emit = |class: ClassifierId, fragment: &str| {
        let wire = format!("sysml:{} {key}#{fragment}", class.metadata().name());
        let target = registry
            .resolve("project", Some(&json!(wire)), class)
            .unwrap()
            .unwrap();
        assert_eq!(target.element_id(), fragment);
        assert_ne!(target.node_id(), fragment);
        wire
    };
    let [(import_span, visibility, import_name, imported)] = imports.as_slice() else {
        panic!("one scoped membership import expected");
    };
    let imported_id = projection.stable_id(*imported).unwrap();
    let mut memberships = Vec::new();
    member_ids(document.value(), imported_id, &mut memberships);
    let [member] = memberships.as_slice() else {
        panic!("owning membership identity")
    };
    let membership_wire = emit(ClassifierId::OwningMembership, member);
    let chosen_class = chosen.map(|id| model.get(id).unwrap().class.as_str());
    let datatype_wire = chosen
        .filter(|_| chosen_class == Some("DataType"))
        .map(|id| emit(ClassifierId::DataType, projection.stable_id(id).unwrap()));
    json!({
        "parsed_name":name,"name_span":typing, "scope_chain":scopes, "selected_class":chosen_class,
        "query_graph_scoped_unqualified":query_local, "query_graph_global_unqualified":query_all,
        "import":{"span":import_span,"visibility":format!("{visibility:?}"),"qualified_name":import_name},
        "importedMembership":membership_wire, "type":datatype_wire
    })
}

fn main() {
    let args: Vec<_> = std::env::args_os().collect();
    assert_eq!(args.len(), 3);
    let fixtures: Value = serde_json::from_slice(&fs::read(&args[1]).unwrap()).unwrap();
    let doc = Document::from_json(&fs::read_to_string(&args[2]).unwrap()).unwrap();
    let grammar = Grammar::load(Language::SysML).unwrap();
    let key = "kermllibrary:///b2c6dd37-2084-3ce4-9ce2-580fdf30629c";
    let mut results = serde_json::Map::new();
    for (name, source) in fixtures.as_object().unwrap() {
        results.insert(
            name.clone(),
            run(source.as_str().unwrap(), &grammar, &doc, key),
        );
    }
    println!("{}", serde_json::to_string_pretty(&results).unwrap());
}
