use std::str::FromStr;

use thiserror::Error;

pub use crate::common::{Directive, Txt, Type, Value};
use crate::position::Pos;

pub use crate::query::{FragmentDefinition, OperationDefinition};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Document<'a> {
    pub definitions: Vec<Definition<'a>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Definition<'a> {
    Schema(SchemaDefinition<'a>),
    Type(TypeDefinition<'a>),
    TypeExtension(TypeExtension<'a>),
    Directive(DirectiveDefinition<'a>),
    Operation(OperationDefinition<'a>),
    Fragment(FragmentDefinition<'a>),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SchemaDefinition<'a> {
    pub position: Pos,
    pub directives: Vec<Directive<'a>>,
    pub query: Option<Txt<'a>>,
    pub mutation: Option<Txt<'a>>,
    pub subscription: Option<Txt<'a>>,
}

pub trait Named<'a> {
    fn name(&self) -> Option<&'a str>;
}

impl<'a> Named<'a> for Definition<'a> {
    fn name(&self) -> Option<&'a str> {
        match self {
            Definition::Schema(_) => None,
            Definition::Type(t) => t.name(),
            Definition::TypeExtension(te) => te.name(),
            Definition::Directive(d) => Some(d.name),
            Definition::Operation(o) => o.name,
            Definition::Fragment(f) => Some(f.name),
        }
    }
}

impl<'a> Named<'a> for TypeDefinition<'a> {
    fn name(&self) -> Option<&'a str> {
        match self {
            TypeDefinition::Scalar(s) => Some(s.name),
            TypeDefinition::Object(o) => Some(o.name),
            TypeDefinition::Interface(i) => Some(i.name),
            TypeDefinition::Union(u) => Some(u.name),
            TypeDefinition::Enum(e) => Some(e.name),
            TypeDefinition::InputObject(io) => Some(io.name),
        }
    }
}

impl<'a> Named<'a> for TypeExtension<'a> {
    fn name(&self) -> Option<&'a str> {
        match self {
            TypeExtension::Scalar(s) => Some(s.name),
            TypeExtension::Object(o) => Some(o.name),
            TypeExtension::Interface(i) => Some(i.name),
            TypeExtension::Union(u) => Some(u.name),
            TypeExtension::Enum(e) => Some(e.name),
            TypeExtension::InputObject(io) => Some(io.name),
        }
    }
}

impl<'a> Named<'a> for Field<'a> {
    fn name(&self) -> Option<&'a str> {
        Some(self.name)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeDefinition<'a> {
    Scalar(ScalarType<'a>),
    Object(ObjectType<'a>),
    Interface(InterfaceType<'a>),
    Union(UnionType<'a>),
    Enum(EnumType<'a>),
    InputObject(InputObjectType<'a>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeExtension<'a> {
    Scalar(ScalarTypeExtension<'a>),
    Object(ObjectTypeExtension<'a>),
    Interface(InterfaceTypeExtension<'a>),
    Union(UnionTypeExtension<'a>),
    Enum(EnumTypeExtension<'a>),
    InputObject(InputObjectTypeExtension<'a>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScalarType<'a> {
    pub position: Pos,
    pub description: Option<String>,
    pub name: Txt<'a>,
    pub directives: Vec<Directive<'a>>,
}

impl<'a> ScalarType<'a> {
    pub fn new(name: Txt<'a>) -> Self {
        Self {
            position: Pos::default(),
            description: None,
            name,
            directives: vec![],
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScalarTypeExtension<'a> {
    pub position: Pos,
    pub name: Txt<'a>,
    pub directives: Vec<Directive<'a>>,
}

impl<'a> ScalarTypeExtension<'a> {
    pub fn new(name: Txt<'a>) -> Self {
        Self {
            position: Pos::default(),
            name,
            directives: vec![],
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObjectType<'a> {
    pub position: Pos,
    pub description: Option<String>,
    pub name: Txt<'a>,
    pub implements_interfaces: Vec<Txt<'a>>,
    pub directives: Vec<Directive<'a>>,
    pub fields: Vec<Field<'a>>,
}

impl<'a> ObjectType<'a> {
    pub fn new(name: Txt<'a>) -> Self {
        Self {
            position: Pos::default(),
            description: None,
            name,
            implements_interfaces: vec![],
            directives: vec![],
            fields: vec![],
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObjectTypeExtension<'a> {
    pub position: Pos,
    pub name: Txt<'a>,
    pub implements_interfaces: Vec<Txt<'a>>,
    pub directives: Vec<Directive<'a>>,
    pub fields: Vec<Field<'a>>,
}

impl<'a> ObjectTypeExtension<'a> {
    pub fn new(name: Txt<'a>) -> Self {
        Self {
            position: Pos::default(),
            name,
            implements_interfaces: vec![],
            directives: vec![],
            fields: vec![],
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Field<'a> {
    pub position: Pos,
    pub description: Option<String>,
    pub name: Txt<'a>,
    pub arguments: Vec<InputValue<'a>>,
    pub field_type: Type<'a>,
    pub directives: Vec<Directive<'a>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InputValue<'a> {
    pub position: Pos,
    pub description: Option<String>,
    pub name: Txt<'a>,
    pub value_type: Type<'a>,
    pub default_value: Option<Value<'a>>,
    pub directives: Vec<Directive<'a>>,
}

impl<'a> Named<'a> for InputValue<'a> {
    fn name(&self) -> Option<&'a str> {
        Some(self.name)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct InterfaceType<'a> {
    pub position: Pos,
    pub description: Option<String>,
    pub name: Txt<'a>,
    pub implements_interfaces: Vec<Txt<'a>>,
    pub directives: Vec<Directive<'a>>,
    pub fields: Vec<Field<'a>>,
}

impl<'a> InterfaceType<'a> {
    pub fn new(name: Txt<'a>) -> Self {
        Self {
            position: Pos::default(),
            description: None,
            name,
            implements_interfaces: vec![],
            directives: vec![],
            fields: vec![],
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct InterfaceTypeExtension<'a> {
    pub position: Pos,
    pub name: Txt<'a>,
    pub directives: Vec<Directive<'a>>,
    pub fields: Vec<Field<'a>>,
}

impl<'a> InterfaceTypeExtension<'a> {
    pub fn new(name: Txt<'a>) -> Self {
        Self {
            position: Pos::default(),
            name,
            directives: vec![],
            fields: vec![],
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnionType<'a> {
    pub position: Pos,
    pub description: Option<String>,
    pub name: Txt<'a>,
    pub directives: Vec<Directive<'a>>,
    pub types: Vec<Txt<'a>>,
}

impl<'a> UnionType<'a> {
    pub fn new(name: Txt<'a>) -> Self {
        Self {
            position: Pos::default(),
            description: None,
            name,
            directives: vec![],
            types: vec![],
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnionTypeExtension<'a> {
    pub position: Pos,
    pub name: Txt<'a>,
    pub directives: Vec<Directive<'a>>,
    pub types: Vec<Txt<'a>>,
}

impl<'a> UnionTypeExtension<'a> {
    pub fn new(name: Txt<'a>) -> Self {
        Self {
            position: Pos::default(),
            name,
            directives: vec![],
            types: vec![],
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumType<'a> {
    pub position: Pos,
    pub description: Option<String>,
    pub name: Txt<'a>,
    pub directives: Vec<Directive<'a>>,
    pub values: Vec<EnumValue<'a>>,
}

impl<'a> EnumType<'a> {
    pub fn new(name: Txt<'a>) -> Self {
        Self {
            position: Pos::default(),
            description: None,
            name,
            directives: vec![],
            values: vec![],
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumValue<'a> {
    pub position: Pos,
    pub description: Option<String>,
    pub name: Txt<'a>,
    pub directives: Vec<Directive<'a>>,
}

impl<'a> EnumValue<'a> {
    pub fn new(name: Txt<'a>) -> Self {
        Self {
            position: Pos::default(),
            description: None,
            name,
            directives: vec![],
        }
    }
}

impl<'a> Named<'a> for EnumValue<'a> {
    fn name(&self) -> Option<&'a str> {
        Some(self.name)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumTypeExtension<'a> {
    pub position: Pos,
    pub name: Txt<'a>,
    pub directives: Vec<Directive<'a>>,
    pub values: Vec<EnumValue<'a>>,
}

impl<'a> EnumTypeExtension<'a> {
    pub fn new(name: Txt<'a>) -> Self {
        Self {
            position: Pos::default(),
            name,
            directives: vec![],
            values: vec![],
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct InputObjectType<'a> {
    pub position: Pos,
    pub description: Option<String>,
    pub name: Txt<'a>,
    pub directives: Vec<Directive<'a>>,
    pub fields: Vec<InputValue<'a>>,
}

impl<'a> InputObjectType<'a> {
    pub fn new(name: Txt<'a>) -> Self {
        Self {
            position: Pos::default(),
            description: None,
            name,
            directives: vec![],
            fields: vec![],
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct InputObjectTypeExtension<'a> {
    pub position: Pos,
    pub name: Txt<'a>,
    pub directives: Vec<Directive<'a>>,
    pub fields: Vec<InputValue<'a>>,
}

impl<'a> InputObjectTypeExtension<'a> {
    pub fn new(name: Txt<'a>) -> Self {
        Self {
            position: Pos::default(),
            name,
            directives: vec![],
            fields: vec![],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DirectiveLocation {
    // executable
    Query,
    Mutation,
    Subscription,
    Field,
    FragmentDefinition,
    FragmentSpread,
    InlineFragment,

    // type_system
    Schema,
    Scalar,
    Object,
    FieldDefinition,
    ArgumentDefinition,
    Interface,
    Union,
    Enum,
    EnumValue,
    InputObject,
    InputFieldDefinition,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DirectiveDefinition<'a> {
    pub position: Pos,
    pub description: Option<String>,
    pub name: Txt<'a>,
    pub arguments: Vec<InputValue<'a>>,
    pub locations: Vec<DirectiveLocation>,
}

impl<'a> DirectiveDefinition<'a> {
    pub fn new(name: Txt<'a>) -> Self {
        Self {
            position: Pos::default(),
            description: None,
            name,
            arguments: vec![],
            locations: vec![],
        }
    }
}

impl DirectiveLocation {
    /// Returns GraphQL syntax compatible name of the directive
    pub fn as_str(&self) -> &'static str {
        use self::DirectiveLocation::*;
        match *self {
            Query => "QUERY",
            Mutation => "MUTATION",
            Subscription => "SUBSCRIPTION",
            Field => "FIELD",
            FragmentDefinition => "FRAGMENT_DEFINITION",
            FragmentSpread => "FRAGMENT_SPREAD",
            InlineFragment => "INLINE_FRAGMENT",
            Schema => "SCHEMA",
            Scalar => "SCALAR",
            Object => "OBJECT",
            FieldDefinition => "FIELD_DEFINITION",
            ArgumentDefinition => "ARGUMENT_DEFINITION",
            Interface => "INTERFACE",
            Union => "UNION",
            Enum => "ENUM",
            EnumValue => "ENUM_VALUE",
            InputObject => "INPUT_OBJECT",
            InputFieldDefinition => "INPUT_FIELD_DEFINITION",
        }
    }

    /// Returns `true` if this location is for queries (execution)
    pub fn is_query(&self) -> bool {
        use self::DirectiveLocation::*;
        match *self {
            Query | Mutation | Subscription | Field | FragmentDefinition | FragmentSpread
            | InlineFragment => true,

            Schema | Scalar | Object | FieldDefinition | ArgumentDefinition | Interface | Union
            | Enum | EnumValue | InputObject | InputFieldDefinition => false,
        }
    }

    /// Returns `true` if this location is for schema
    pub fn is_schema(&self) -> bool {
        !self.is_query()
    }
}

#[derive(Debug, Error)]
#[error("invalid directive location")]
pub struct InvalidDirectiveLocation;

impl FromStr for DirectiveLocation {
    type Err = InvalidDirectiveLocation;
    fn from_str(s: &str) -> Result<DirectiveLocation, InvalidDirectiveLocation> {
        use self::DirectiveLocation::*;
        let val = match s {
            "QUERY" => Query,
            "MUTATION" => Mutation,
            "SUBSCRIPTION" => Subscription,
            "FIELD" => Field,
            "FRAGMENT_DEFINITION" => FragmentDefinition,
            "FRAGMENT_SPREAD" => FragmentSpread,
            "INLINE_FRAGMENT" => InlineFragment,
            "SCHEMA" => Schema,
            "SCALAR" => Scalar,
            "OBJECT" => Object,
            "FIELD_DEFINITION" => FieldDefinition,
            "ARGUMENT_DEFINITION" => ArgumentDefinition,
            "INTERFACE" => Interface,
            "UNION" => Union,
            "ENUM" => Enum,
            "ENUM_VALUE" => EnumValue,
            "INPUT_OBJECT" => InputObject,
            "INPUT_FIELD_DEFINITION" => InputFieldDefinition,
            _ => return Err(InvalidDirectiveLocation),
        };

        Ok(val)
    }
}
