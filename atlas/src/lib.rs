use std::{sync::Arc, collections::HashMap};
use graphql_parser::{query::Operation, schema::{Document, Definition, parse_schema, ParseError, TypeDefinition, TypeExtension, Named, Field, EnumValue, InputValue}};

#[derive(Debug, PartialEq)]
pub struct Point {
    kind: Kind,
    name: Option<String>,
    edges: Edges,
}

type Sel = Vec<Arc<Point>>;

type Edges = HashMap<Edge, Sel>;

trait Select {
    fn sel(&self, edge: Edge) -> Sel;
    fn child(&self, name: &str) -> Sel {
        self.sel(Edge::name(name))
    }
    fn kind(&self) -> Vec<Kind>;    
}

impl Select for Sel {
    fn sel(&self, edge: Edge) -> Sel {
        self.iter().filter_map(|pt| pt.edges.get(&edge))
            .flat_map(|p| { p.iter().map(Arc::clone) })
            .collect()
    }

    fn kind(&self) -> Vec<Kind> {
        self.iter().map(|p| p.kind).collect()
    }
}

impl Select for Point {
    fn sel(&self, edge: Edge) -> Sel {
        match self.edges.get(&edge) {
            Some(s) => s.clone(),
            None => vec![],
        }
    }
    
    fn kind(&self) -> Vec<Kind> {
        vec![self.kind]
    }
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum Edge {
    Name(String),
}

impl Edge {
    pub fn name(name: &str) -> Edge {
        Edge::Name(String::from(name))
    }
}


#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Kind {
    Schema,
    Scalar(Ext),
    Object(Ext),
    Interface(Ext),
    Union(Ext),
    Enum(Ext),
    EnumValue,
    InputObject(Ext),
    InputValue,
    Directive,
    Operation(Operation),
    Fragment,
    Field,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Ext {
    Definition,
    Extension,
}

impl Point {
    pub fn from_text(source: &str, name: Option<String>) -> Result<Sel, ParseError> {
        let mut point = parse_schema(source)?.point().unwrap();
        point.name = name;
        Ok(vec![Arc::new(point)])
    }
}

fn index_names<'a, T>(col: &Vec<T>) -> Edges
where T: AsPoint + Named<'a>
{
    let mut edges = Edges::new();
    for child in col {
        if let (Some(name), Some(point)) = (child.name(), child.point()) {
            let family = edges.entry(Edge::name(name))
                .or_insert(vec![]);
            family.push(Arc::new(point));
        }
    }
    edges    
}

trait AsPoint {
    fn point(&self) -> Option<Point> { None }
    fn edges(&self) -> Edges { Edges::new() }
}

impl<'a> AsPoint for Document<'a> {
    fn point(&self) -> Option<Point> {
        Some(Point {
            kind: Kind::Schema,
            name: None,
            edges: self.edges(),
        })
    }

    fn edges(&self) -> Edges {
        index_names(&self.definitions)
    }
}

impl<'a> AsPoint for Field<'a> {
    fn point(&self) -> Option<Point> {
        Some(Point {            
            kind: Kind::Field,
            name: Some(String::from(self.name)),
            edges: self.edges()
        })
    }
}

impl<'a> AsPoint for EnumValue<'a> {
    fn point(&self) -> Option<Point> {
        Some(Point {
            kind: Kind::EnumValue,
            name: Some(String::from(self.name)),
            edges: HashMap::new(),
        })
    }
}


impl<'a> AsPoint for InputValue<'a> {
    fn point(&self) -> Option<Point> {
        Some(Point {
            kind: Kind::InputValue,
            name: Some(String::from(self.name)),
            edges: HashMap::new(),
        })
    }
}

macro_rules! empty {
    () => { Edges::new() }
}

impl<'a> AsPoint for Definition<'a> {
    fn point(&self) -> Option<Point> {
        Some(Point {
            kind: match self {
                Definition::Schema(_) => { return None }
                Definition::Type(def) => match def {
                    TypeDefinition::Scalar(_) => Kind::Scalar(Ext::Definition),
                    TypeDefinition::Object(_) => Kind::Object(Ext::Definition),
                    TypeDefinition::Interface(_) => Kind::Interface(Ext::Definition),
                    TypeDefinition::Union(_) => Kind::Union(Ext::Definition),
                    TypeDefinition::Enum(_) => Kind::Enum(Ext::Definition),
                    TypeDefinition::InputObject(_) => Kind::InputObject(Ext::Definition)
                }
                Definition::TypeExtension(ext) => match ext {
                    TypeExtension::Scalar(_) => Kind::Scalar(Ext::Extension),
                    TypeExtension::Object(_) => Kind::Object(Ext::Extension),
                    TypeExtension::Interface(_) => Kind::Interface(Ext::Extension),
                    TypeExtension::Union(_) => Kind::Union(Ext::Extension),
                    TypeExtension::Enum(_) => Kind::Enum(Ext::Extension),
                    TypeExtension::InputObject(_) => Kind::InputObject(Ext::Extension),
                }
                Definition::Directive(_) => Kind::Directive,
                Definition::Operation(op) => Kind::Operation(op.kind),
                Definition::Fragment(_) => Kind::Fragment,
            },
            name: self.name().map(|n| n.to_owned()),
            edges: self.edges(),
        })
    }

    fn edges(&self) -> Edges {
        match self {
            Definition::Schema(_) => empty!(),
            Definition::Type(t) => match t {
                TypeDefinition::Scalar(_) => empty!(),
                TypeDefinition::Object(o) => index_names(&o.fields),
                TypeDefinition::Interface(i) => index_names(&i.fields),
                TypeDefinition::Union(_) => HashMap::new(),
                TypeDefinition::Enum(en) => index_names(&en.values),
                TypeDefinition::InputObject(io) => index_names(&io.fields),
            },
            Definition::TypeExtension(_) => empty!(),
            Definition::Directive(_) => empty!(),
            Definition::Operation(_) => empty!(),
            Definition::Fragment(_) => empty!(),
        }
    }    
}


#[test]
fn parses_graphql() -> Result<(), ParseError> {
    use crate::{Point, Edge};
    let text = "
        type User {
            id: String
            name: Int
        }
    ";

    let schema = Point::from_text(text, None)?;
    let user = schema.child("User");    
    assert_eq!(user[0].kind, Kind::Object(Ext::Definition), "it's an object type");
    assert_eq!(user.sel(Edge::name("id"))[0].kind, Kind::Field, "has an id field");
    assert_eq!(user.child("name")[0].kind, Kind::Field, "has a name field");

    assert_eq!(schema.child("User").child("name").kind(), vec![Kind::Field], "has one name field");
    Ok(())
}
