use super::*;

fn binding(name: &str, definition: Type) -> TypeBinding {
    TypeBinding { name: name.into(), definition, fields: vec![] }
}

#[test]
fn clones_share_identity_until_mutation() {
    let mut table = TypeTable::new();
    let n = table.add(binding("Named", Type::Primitive(Prim::Int)));
    let mut other = table.clone();
    let snapshot = Arc::clone(table.revision());
    assert!(Arc::ptr_eq(&snapshot, other.revision()));
    other.update(n, Type::Primitive(Prim::Bool), vec![]);
    assert!(!Arc::ptr_eq(&snapshot, other.revision()));
    assert!(Arc::ptr_eq(&snapshot, table.revision()));
    assert_eq!(table.expansion(n), &Type::Primitive(Prim::Int));
    assert_eq!(other.expansion(n), &Type::Primitive(Prim::Bool));
    assert!(!Arc::ptr_eq(TypeTable::new().revision(), TypeTable::new().revision()));
}

#[test]
fn all_publication_paths_invalidate() {
    let mut table = TypeTable::new();
    let mut old = Arc::clone(table.revision());
    let n = table.add(binding("Named", Type::Primitive(Prim::Int)));
    assert!(!Arc::ptr_eq(&old, table.revision()));
    old = Arc::clone(table.revision());
    table.update(n, Type::Primitive(Prim::Bool), vec![]);
    assert!(!Arc::ptr_eq(&old, table.revision()));
    old = Arc::clone(table.revision());
    let row = table.add_constructor(binding("Row", Type::row(Type::Variable(0))), 1, false);
    assert!(!Arc::ptr_eq(&old, table.revision()));
    assert_eq!(table.constructor_arity(row), 1);
    old = Arc::clone(table.revision());
    let simple = table.add_simple_constructor(binding("Simple", Type::Primitive(Prim::Int)), 0);
    assert!(!Arc::ptr_eq(&old, table.revision()));
    let len = table.bindings.len();
    old = Arc::clone(table.revision());
    let mut repeated = binding("Simple", Type::Primitive(Prim::Int));
    repeated.fields = vec![Some("changed_field".into())];
    assert_eq!(table.add_simple_constructor(repeated, 0), simple);
    assert_eq!(table.bindings.len(), len, "reused-slot mutation must invalidate too");
    assert!(!Arc::ptr_eq(&old, table.revision()));
    assert_eq!(table.binding(simple).fields, vec![Some("changed_field".into())]);
    old = Arc::clone(table.revision());
    assert!(table.forget("Simple"));
    assert!(!Arc::ptr_eq(&old, table.revision()));
    old = Arc::clone(table.revision());
    assert!(!table.forget("absent"));
    assert!(Arc::ptr_eq(&old, table.revision()), "unsuccessful forget changes nothing");
}

#[test]
fn recursive_completion_invalidates() {
    let mut table = TypeTable::new();
    let n = table.add_constructor(binding("Pair", Type::Undetermined), 0, true);
    let old = Arc::clone(table.revision());
    let rhs = Type::tuple(vec![Type::Primitive(Prim::Int), Type::Primitive(Prim::Bool)]);
    table.complete_recursive_group(&[n], &[(rhs.clone(), vec![])], 0).unwrap();
    assert!(!Arc::ptr_eq(&old, table.revision()));
    assert_eq!(table.expansion(n), &rhs);
    assert!(!table.is_recursive(n));
}

#[test]
fn semantic_equality_debug_and_auto_traits_ignore_revision() {
    fn send_sync<T: Send + Sync>() {}
    send_sync::<TypeTable>();
    assert_eq!(format!("{:?}", TypeTable::new()), "TypeTable { bindings: [], active: {}, constructors: {} }");
    let mut table = TypeTable::new();
    let n = table.add(binding("Named", Type::Primitive(Prim::Int)));
    let snapshot = table.clone();
    table.update(n, Type::Primitive(Prim::Int), vec![]);
    assert!(!Arc::ptr_eq(snapshot.revision(), table.revision()));
    assert_eq!(snapshot, table);
    assert_eq!(format!("{snapshot:?}"), format!("{table:?}"));
}
