//! Graph-wide recursive typedef installation (axis-types.w:1640-2010).
//! Named RHS slots come first; cyclic anonymous descendants keep identities
//! too. The caller stages this table together with all generated members.

use super::{Type, TypeBinding, TypeNumber, TypeTable};

#[derive(Clone)]
struct Node {
    shape: Type,
    edges: Vec<usize>,
}

fn reference(number: TypeNumber, arity: usize) -> Type {
    if arity == 0 { Type::Tabled(number) }
    else { Type::Applied(number, (0..arity).map(Type::Variable).collect()) }
}

impl TypeTable {
    /// `numbers` are the contiguous placeholders just reserved by the caller.
    /// All references to them already forward the full group argument list.
    pub(crate) fn complete_recursive_group(
        &mut self,
        numbers: &[TypeNumber],
        definitions: &[(Type, Vec<Option<String>>)],
        arity: usize,
    ) -> Result<(), String> {
        let Some(first) = numbers.first() else { return Ok(()); };
        let first = first.0;
        let count = numbers.len();
        assert_eq!(definitions.len(), count);
        assert_eq!(self.bindings.len(), first + count);
        assert!(numbers.iter().enumerate().all(|(i, n)| n.0 == first + i));
        let mut nodes = vec![Node { shape: Type::Undetermined, edges: vec![] }; count];
        for (i, (definition, _)) in definitions.iter().enumerate() {
            dissect(definition, self, first, count, &mut nodes)?;
            nodes[i] = nodes.pop().expect("dissection appends its root");
        }
        let recursive = cyclic_nodes(&nodes);
        for (i, node) in nodes.iter().enumerate() {
            if recursive[i] {
                if let Type::Tabled(number) | Type::Applied(number, _) = &node.shape {
                    return Err(format!("Type definition recursion uses type constructor '{}', itself recursive, which is not allowed",
                        self.binding(*number).name));
                }
            }
        }
        // Retain in dissection order, never in SCC traversal order.
        let mut retained = vec![None; nodes.len()];
        for (i, slot) in retained.iter_mut().enumerate() {
            if i < count {
                *slot = Some(numbers[i]);
            } else if recursive[i] {
                *slot = Some(self.add_constructor(TypeBinding {
                    name: String::new(), definition: Type::Undetermined, fields: vec![],
                }, arity, true));
            }
        }
        for (i, slot) in retained.iter().enumerate() {
            if let Some(number) = slot {
                let mut body = rewrite(i, &nodes, &retained, arity);
                // An existing recursive constructor is legal outside a new
                // cycle, but a stored body must have a structural top level.
                if matches!(body, Type::Tabled(_) | Type::Applied(..)) {
                    body = self.expand_application(&body).map_err(|e| format!("Invalid constructor expansion: {e:?}"))?;
                }
                self.update(*number, body,
                    if i < count { definitions[i].1.clone() } else { vec![] });
                self.constructors.insert(number.0, (arity, recursive[i]));
            }
        }
        Ok(())
    }
}

fn record(ty: &Type, table: &TypeTable, first: usize, count: usize,
          nodes: &mut Vec<Node>) -> Result<usize, String> {
    if let Type::Tabled(number) | Type::Applied(number, _) = ty {
        if (first..first + count).contains(&number.0) { return Ok(number.0 - first); }
    }
    dissect(ty, table, first, count, nodes)
}

fn dissect(ty: &Type, table: &TypeTable, first: usize, count: usize,
           nodes: &mut Vec<Node>) -> Result<usize, String> {
    let children: Vec<&Type> = match ty {
        Type::Tabled(number) | Type::Applied(number, _) => {
            if (first..first + count).contains(&number.0) {
                // Upstream dissect_to asserts for a bare local RHS. Do not
                // recurse forever or fabricate a structural definition here.
                return Err("A grouped type definition needs a structural right-hand side, not a direct reference to its own group".into());
            }
            if !table.is_recursive(*number) {
                let expanded = table.expand_application(ty).map_err(|e| format!("Invalid constructor expansion: {e:?}"))?;
                return dissect(&expanded, table, first, count, nodes);
            }
            match ty { Type::Applied(_, args) => args.iter().collect(), _ => vec![] }
        }
        Type::Row(component) => vec![component],
        Type::Function(parts) => vec![&parts.0, &parts.1],
        Type::Tuple(parts) | Type::Union(parts) => parts.iter().collect(),
        _ => vec![],
    };
    let edges = children.into_iter().map(|child| record(child, table, first, count, nodes))
        .collect::<Result<Vec<_>, _>>()?;
    let index = nodes.len();
    nodes.push(Node { shape: ty.clone(), edges });
    Ok(index)
}

fn rewrite(index: usize, nodes: &[Node], retained: &[Option<TypeNumber>], arity: usize) -> Type {
    let node = &nodes[index];
    let mut children = node.edges.iter().map(|&child| match retained[child] {
        Some(number) => reference(number, arity),
        None => rewrite(child, nodes, retained, arity),
    });
    match &node.shape {
        Type::Row(_) => Type::row(children.next().expect("row child")),
        Type::Function(_) => Type::function(children.next().expect("argument"),
            children.next().expect("result")),
        Type::Tuple(_) => Type::tuple(children.collect()),
        Type::Union(_) => Type::union_of(children.collect()),
        Type::Applied(number, _) => Type::Applied(*number, children.collect()),
        other => other.clone(),
    }
}

/// Iterative Kosaraju: flag nontrivial SCCs and singleton self loops.
/// No graph recursion on the host stack, and no quadratic closure matrix.
fn cyclic_nodes(nodes: &[Node]) -> Vec<bool> {
    let mut seen = vec![false; nodes.len()];
    let mut finish = Vec::with_capacity(nodes.len());
    for start in 0..nodes.len() {
        if seen[start] { continue; }
        seen[start] = true;
        let mut stack = vec![(start, 0)];
        while let Some((node, next)) = stack.last_mut() {
            if let Some(&child) = nodes[*node].edges.get(*next) {
                *next += 1;
                if !seen[child] { seen[child] = true; stack.push((child, 0)); }
            } else {
                finish.push(*node);
                stack.pop();
            }
        }
    }
    let mut incoming = vec![vec![]; nodes.len()];
    for (i, node) in nodes.iter().enumerate() {
        for &child in &node.edges { incoming[child].push(i); }
    }
    seen.fill(false);
    let mut recursive = vec![false; nodes.len()];
    for start in finish.into_iter().rev() {
        if seen[start] { continue; }
        seen[start] = true;
        let mut stack = vec![start];
        let mut component = vec![];
        while let Some(node) = stack.pop() {
            component.push(node);
            for &parent in &incoming[node] {
                if !seen[parent] { seen[parent] = true; stack.push(parent); }
            }
        }
        if component.len() > 1 || nodes[start].edges.contains(&start) {
            for node in component { recursive[node] = true; }
        }
    }
    recursive
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Prim;

    fn reserve(table: &mut TypeTable, name: &str, arity: usize) -> TypeNumber {
        table.add_constructor(TypeBinding { name: name.into(),
            definition: Type::Undetermined, fields: vec![] }, arity, true)
    }

    #[test]
    fn scc_flags_match_independent_closure_for_every_three_vertex_graph() {
        for mask in 0..512 {
            let nodes: Vec<_> = (0..3).map(|i| Node { shape: Type::Undetermined,
                edges: (0..3).filter(|j| mask & (1 << (i * 3 + j)) != 0).collect() }).collect();
            let mut reach = [[false; 3]; 3];
            for i in 0..3 { for &j in &nodes[i].edges { reach[i][j] = true; } }
            for k in 0..3 { for i in 0..3 { for j in 0..3 {
                reach[i][j] |= reach[i][k] && reach[k][j];
            } } }
            assert_eq!(cyclic_nodes(&nodes), (0..3).map(|i| reach[i][i]).collect::<Vec<_>>());
        }
    }

    #[test]
    fn anonymous_cycles_keep_postorder_identities_and_unused_formals() {
        let mut table = TypeTable::new();
        let tree = reserve(&mut table, "Tree", 2);
        let t = reference(tree, 2);
        let rhs = Type::union_of(vec![Type::void(), Type::tuple(vec![
            Type::tuple(vec![Type::Variable(0), t.clone()]), Type::row(t)])]);
        table.complete_recursive_group(&[tree], &[(rhs, vec![])], 2).unwrap();
        assert_eq!(table.bindings.len(), 4);
        assert!(table.lookup("").is_none());
        for i in 0..4 {
            assert!(table.is_recursive(TypeNumber(i)));
            assert_eq!(table.constructor_arity(TypeNumber(i)), 2);
        }
        assert_eq!(table.expansion(tree).display(&table).to_string(),
            "(void|((A,Tree<A,B>),[Tree<A,B>]))");
        assert!(matches!(table.expansion(TypeNumber(1)), Type::Tuple(parts) if parts.len() == 2));
        assert!(matches!(table.expansion(TypeNumber(2)), Type::Row(_)));
        assert!(matches!(table.expansion(TypeNumber(3)), Type::Tuple(parts) if parts.len() == 2));
        assert_eq!(Type::function(reference(TypeNumber(3), 2), Type::void()).display(&table).to_string(),
            "((A,Tree<A,B>),[Tree<A,B>]->)");
    }

    #[test]
    fn acyclic_groups_are_structural_and_forward_formals() {
        let mut table = TypeTable::new();
        let first = reserve(&mut table, "First", 2);
        let second = reserve(&mut table, "Second", 2);
        table.complete_recursive_group(&[first, second], &[
            (Type::row(reference(second, 2)), vec![]),
            (Type::tuple(vec![Type::Variable(0), Type::Variable(1)]), vec![]),
        ], 2).unwrap();
        assert_eq!(table.bindings.len(), 2);
        assert!(!table.is_recursive(first) && !table.is_recursive(second));
        assert_eq!(table.expansion(first).display(&table).to_string(), "[Second<A,B>]");
    }

    #[test]
    fn old_recursive_constructor_on_new_cycle_is_rejected_before_installation() {
        let mut table = TypeTable::new();
        let old = reserve(&mut table, "Old", 1);
        table.complete_recursive_group(&[old], &[(Type::row(reference(old, 1)), vec![])], 1).unwrap();
        let new = reserve(&mut table, "New", 1);
        let snapshot = table.clone();
        let result = table.complete_recursive_group(&[new], &[(Type::tuple(vec![
            Type::Primitive(Prim::Int), Type::Applied(old, vec![reference(new, 1)])]), vec![])], 1);
        assert_eq!(result.unwrap_err(), "Type definition recursion uses type constructor 'Old', itself recursive, which is not allowed");
        assert_eq!(table, snapshot);
    }

    #[test]
    fn old_recursive_instance_outside_cycle_and_nonrecursive_wrapper_are_legal() {
        let mut table = TypeTable::new();
        let old = reserve(&mut table, "Old", 1);
        table.complete_recursive_group(&[old], &[(Type::union_of(vec![Type::void(),
            Type::tuple(vec![Type::Variable(0), reference(old, 1)])]), vec![])], 1).unwrap();
        let alias = reserve(&mut table, "Alias", 1);
        table.complete_recursive_group(&[alias], &[(reference(old, 1), vec![])], 1).unwrap();
        assert!(!table.is_recursive(alias));
        assert!(matches!(table.expansion(alias), Type::Union(_)));
        let pair = table.add_simple_constructor(TypeBinding { name: "Pair".into(),
            definition: Type::tuple(vec![Type::Variable(0), Type::Variable(0)]), fields: vec![] }, 1);
        let wrapped = reserve(&mut table, "Wrapped", 1);
        table.complete_recursive_group(&[wrapped], &[(Type::union_of(vec![Type::void(),
            Type::Applied(pair, vec![reference(wrapped, 1)])]), vec![])], 1).unwrap();
        assert_eq!(table.expansion(wrapped).display(&table).to_string(),
            "(void|(Wrapped<A>,Wrapped<A>))");
    }
}
