"""
A URL is an absolute location within the atlas.
"""
scalar URL

"""
Names are dotted paths of GraphQL identifiers, like `spotify.User`
and `String`.
"""
scalar Name

type Query {
  at(id: URL): Point
  edge(name: Name, from: Name, to: Name): Edge
  type(id: URL): Type
  interface(id: URL): Interface
  edge(id: URL): Edge    
  points: [Point]
}

## This would be cool:
# interface Point @key(fields: "id", via: "Query.at(id)") {
#   id: URL
# }

interface Point {
  id: URL
  name: Name
}

# -- Points --
type Type implements Point @key(fields: "id", via: "Query.at(id)") {
  id: URL
  name: Name
  fields: [Edge]
}

type Interface implements Point @key(fields: "id", via: "Query.at(id)") {
  id: URL
  name: Name
}

type Input implements Point @key(fields: "id", via: "Query.at(id)") {
  id: URL
  name: Name
}

type Scalar implements Point @key(fields: "id", via: "Query.at(id)") {
  id: URL
  name: Name
}

type Enum implements Point @key(fields: "id", via: "Query.at(id)") {
  id: URL
  name: Name
}

type List @key(fields: "of") {
  of: Point
}

type Must @key(fields: "of") {
  of: Point
}

type Edge implements Point @key(fields: "id", via: "Query.at(id)") {
  id: URL
  name: Name
  from: Point
  to: Point
}


# The schema of the GraphQL schema introspection system:

type __Schema {
  types: [__Type!]!
  queryType: __Type!
  mutationType: __Type
  subscriptionType: __Type
  directives: [__Directive!]!
}

type __Type {
  kind: __TypeKind!
  name: String
  description: String

  # OBJECT and INTERFACE only
  fields(includeDeprecated: Boolean = false): [__Field!]

  # OBJECT only
  interfaces: [__Type!]

  # INTERFACE and UNION only
  possibleTypes: [__Type!]

  # ENUM only
  enumValues(includeDeprecated: Boolean = false): [__EnumValue!]

  # INPUT_OBJECT only
  inputFields: [__InputValue!]

  # NON_NULL and LIST only
  ofType: __Type
}

type __Field {
  name: String!
  description: String
  args: [__InputValue!]!
  type: __Type!
  isDeprecated: Boolean!
  deprecationReason: String
}

type __InputValue {
  name: String!
  description: String
  type: __Type!
  defaultValue: String
}

type __EnumValue {
  name: String!
  description: String
  isDeprecated: Boolean!
  deprecationReason: String
}

enum __TypeKind {
  SCALAR
  OBJECT
  INTERFACE
  UNION
  ENUM
  INPUT_OBJECT
  LIST
  NON_NULL
}

type __Directive {
  name: String!
  description: String
  locations: [__DirectiveLocation!]!
  args: [__InputValue!]!
}

enum __DirectiveLocation {
  """
  A Query.
  """
  QUERY
  MUTATION
  SUBSCRIPTION
  FIELD
  FRAGMENT_DEFINITION
  FRAGMENT_SPREAD
  INLINE_FRAGMENT
  SCHEMA
  SCALAR
  OBJECT
  FIELD_DEFINITION
  ARGUMENT_DEFINITION
  INTERFACE
  UNION
  ENUM
  ENUM_VALUE
  INPUT_OBJECT
  INPUT_FIELD_DEFINITION @abc @1 @2(x: y) @2
}

interface Foo implements Bar  & Baz & X @0 {
  x: Int
}