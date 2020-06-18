//! Query Language Abstract Syntax Tree (AST)
//!
//! The types and fields here resemble official [graphql grammar] whenever it
//! makes sense for rust.
//!
//! [graphql grammar]: http://facebook.github.io/graphql/October2016/#sec-Appendix-Grammar-Summary
//!
pub use crate::common::{Directive, Txt, Type, Value};
use crate::position::Pos;

/// Root of query data
#[derive(Debug, Clone, PartialEq)]
pub struct Document<'a> {
    pub definitions: Vec<Definition<'a>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Definition<'a> {
    SelectionSet(SelectionSet<'a>),
    Operation(OperationDefinition<'a>),
    Fragment(FragmentDefinition<'a>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct FragmentDefinition<'a> {
    pub position: Pos,
    pub description: Option<String>,
    pub name: Txt<'a>,
    pub type_condition: Txt<'a>,
    pub directives: Vec<Directive<'a>>,
    pub selection_set: SelectionSet<'a>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OperationDefinition<'a> {
    pub position: Pos,
    pub kind: Operation,
    pub description: Option<String>,
    pub name: Option<Txt<'a>>,
    pub variable_definitions: Vec<VariableDefinition<'a>>,
    pub directives: Vec<Directive<'a>>,
    pub selection_set: SelectionSet<'a>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Operation {
    Query,
    Mutation,
    Subscription,
}

impl Operation {
    /// Returns GraphQL syntax compatible name of the operation
    pub fn as_str(&self) -> &'static str {
        match *self {
            Self::Query => "query",
            Self::Mutation => "mutation",
            Self::Subscription => "subscription",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SelectionSet<'a> {
    pub span: (Pos, Pos),
    pub items: Vec<Selection<'a>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VariableDefinition<'a> {
    pub position: Pos,
    pub name: Txt<'a>,
    pub var_type: Type<'a>,
    pub default_value: Option<Value<'a>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Selection<'a> {
    Field(Field<'a>),
    FragmentSpread(FragmentSpread<'a>),
    InlineFragment(InlineFragment<'a>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Field<'a> {
    pub position: Pos,
    pub alias: Option<Txt<'a>>,
    pub name: Txt<'a>,
    pub arguments: Vec<(Txt<'a>, Value<'a>)>,
    pub directives: Vec<Directive<'a>>,
    pub selection_set: SelectionSet<'a>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FragmentSpread<'a> {
    pub position: Pos,
    pub fragment_name: Txt<'a>,
    pub directives: Vec<Directive<'a>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InlineFragment<'a> {
    pub position: Pos,
    pub type_condition: Option<Txt<'a>>,
    pub directives: Vec<Directive<'a>>,
    pub selection_set: SelectionSet<'a>,
}
