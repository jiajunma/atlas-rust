//! Parser-owned lexical type scopes (lexer.w:464-614).
//!
//! Tokens must be classified when the parser asks for them, not when the
//! command is collected. A declaration action installs its formal names only
//! after the opening lookahead has pushed a lexical group. This object is
//! shared by the token iterator and, during the generic grammar migration,
//! the declaration actions. It never mutates the session's semantic table.

use std::cell::RefCell;

use crate::types::TypeTable;

use super::{ParserToken, SpannedValue};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GroupKind {
    Delimiter,
    Let,
    Block,
    /// Simple definitions and constructor formal lists push a parser-owned
    /// group even though no corresponding delimiter occurs in the source.
    Virtual,
}

#[derive(Debug)]
struct Clutch {
    kind: GroupKind,
    names: Vec<String>,
}

#[derive(Default, Debug)]
struct ScopeState {
    // Oldest first: a variable index includes every outer clutch's size.
    nest: Vec<Clutch>,
    defining_types: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NameClass {
    Identifier,
    TypeName,
    Constructor,
    Variable(usize),
}

pub struct ParserTypes<'a> {
    types: &'a TypeTable,
    state: RefCell<ScopeState>,
}

impl<'a> ParserTypes<'a> {
    pub fn new(types: &'a TypeTable) -> Self {
        Self { types, state: RefCell::new(ScopeState::default()) }
    }

    pub fn push_group(&self, kind: GroupKind) {
        self.state.borrow_mut().nest.push(Clutch { kind, names: Vec::new() });
    }

    /// Erroneous closing tokens must not panic before the grammar can report
    /// the syntax error (upstream pop_nest also tolerates an empty nest).
    pub fn pop_group(&self) {
        self.state.borrow_mut().nest.pop();
    }

    /// Called after the declaration's opening lookahead, not on each IDENT.
    /// Duplicate slots remain counted; lookup finds their first occurrence.
    /// An empty nest is tolerated only for a reduction on invalid lookahead;
    /// the parser will reject that input and discard this command's state.
    pub fn introduce(&self, names: &[String]) {
        if let Some(clutch) = self.state.borrow_mut().nest.last_mut() {
            clutch.names.extend_from_slice(names);
        }
    }

    pub fn level(&self) -> usize {
        self.state.borrow().nest.iter().map(|clutch| clutch.names.len()).sum()
    }

    pub fn start_defining_types(&self) {
        self.state.borrow_mut().defining_types = true;
    }

    pub fn reset(&self) {
        *self.state.borrow_mut() = ScopeState::default();
    }

    pub fn classify(&self, name: &str) -> NameClass {
        let state = self.state.borrow();
        let mut floor = state.nest.iter().map(|clutch| clutch.names.len()).sum::<usize>();
        for clutch in state.nest.iter().rev() {
            floor -= clutch.names.len();
            if let Some(index) = clutch.names.iter().position(|value| value == name) {
                return NameClass::Variable(floor + index);
            }
        }
        if let Some(number) = self.types.lookup(name) {
            return if self.types.constructor_arity(number) == 0 {
                NameClass::TypeName
            } else {
                NameClass::Constructor
            };
        }
        if state.defining_types { NameClass::TypeName } else { NameClass::Identifier }
    }

    /// Scanner-time delimiter effects precede any reduction on this token.
    /// In particular IN closes only a LET group, not CASE's block group.
    pub(super) fn token(&self, token: ParserToken) -> ParserToken {
        use ParserToken::*;
        match &token {
            LParen(_) | LBracket(_) | ReverseLBracket(_) => self.push_group(GroupKind::Delimiter),
            Let(_) => self.push_group(GroupKind::Let),
            Begin(_) | If(_) | While(_) | For(_) | Case(_) => self.push_group(GroupKind::Block),
            RParen(_) | RBracket(_) | End(_) | Fi(_) | Od(_) | Esac(_) => self.pop_group(),
            In(_) => {
                let closes_let = self.state.borrow().nest.last()
                    .is_some_and(|clutch| clutch.kind == GroupKind::Let);
                if closes_let { self.pop_group(); }
            }
            Newline(_) => self.reset(),
            _ => {}
        }
        match token {
            Identifier(name) => match self.classify(&name.value) {
                NameClass::Identifier => Identifier(name),
                NameClass::TypeName => TypeName(name),
                NameClass::Constructor => TypeConstructor(name),
                NameClass::Variable(value) => TypeVariable(SpannedValue { value, span: name.span }),
            },
            other => other,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{source::SourceText, types::{Prim, Type, TypeBinding}};

    fn names(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_owned()).collect()
    }

    #[test]
    fn duplicate_formals_keep_slots_and_outer_offsets() {
        let table = TypeTable::new();
        let scope = ParserTypes::new(&table);
        scope.push_group(GroupKind::Virtual);
        scope.introduce(&names(&["T", "T"]));
        assert_eq!(scope.level(), 2);
        assert_eq!(scope.classify("T"), NameClass::Variable(0));
        scope.push_group(GroupKind::Delimiter);
        scope.introduce(&names(&["S", "U"]));
        assert_eq!(scope.classify("S"), NameClass::Variable(2));
        assert_eq!(scope.classify("U"), NameClass::Variable(3));
        assert_eq!(scope.classify("T"), NameClass::Variable(0));
        scope.pop_group();
        assert_eq!(scope.level(), 2);
        assert_eq!(scope.classify("S"), NameClass::Identifier);
        scope.pop_group();
        assert_eq!(scope.classify("T"), NameClass::Identifier);
    }

    #[test]
    fn persistent_names_and_constructors_do_not_become_plain_bindings() {
        let mut table = TypeTable::new();
        table.add_alias("Saved", Type::Primitive(Prim::Int));
        table.add_constructor(TypeBinding {
            name: "Pair".into(), definition: Type::tuple(vec![Type::Variable(0), Type::Variable(1)]),
            fields: vec![],
        }, 2, false);
        let scope = ParserTypes::new(&table);
        assert_eq!(scope.classify("Saved"), NameClass::TypeName);
        assert_eq!(scope.classify("Pair"), NameClass::Constructor);
        scope.start_defining_types();
        assert_eq!(scope.classify("Future"), NameClass::TypeName);
        assert_eq!(scope.classify("Pair"), NameClass::Constructor);
        scope.push_group(GroupKind::Delimiter);
        scope.introduce(&names(&["T"]));
        assert_eq!(scope.classify("T"), NameClass::Variable(0));
        scope.reset();
        assert_eq!(scope.classify("Future"), NameClass::Identifier);
        assert_eq!(scope.classify("T"), NameClass::Identifier);
        assert_eq!(scope.classify("Saved"), NameClass::TypeName);
    }

    #[test]
    fn let_in_pops_only_let_and_recovery_clears_virtual_groups() {
        let table = TypeTable::new();
        let scope = ParserTypes::new(&table);
        let span = SourceText::new("x").span(0, 1);
        scope.token(ParserToken::Case(span));
        scope.introduce(&names(&["T"]));
        scope.token(ParserToken::In(span));
        assert_eq!(scope.level(), 1);
        scope.token(ParserToken::Let(span));
        scope.introduce(&names(&["S"]));
        assert_eq!(scope.level(), 2);
        scope.token(ParserToken::In(span));
        assert_eq!(scope.level(), 1);
        scope.token(ParserToken::Esac(span));
        assert_eq!(scope.level(), 0);
        scope.push_group(GroupKind::Virtual);
        scope.introduce(&names(&["U"]));
        scope.token(ParserToken::Newline(span));
        assert_eq!(scope.level(), 0);
        scope.pop_group();
        scope.introduce(&names(&["InvalidLookahead"]));
        assert_eq!(scope.level(), 0);
    }
}
