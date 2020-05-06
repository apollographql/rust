# The Atlas

The Atlas records everything we know about the structure of some part of the universal graph. This includes the graph *structure*—the kind of description we'd find in a schema file. It also includes metadata which might be encoded in a schema file with directives or provided externally. An important piece of metadata is what endpoints can serve particular fields, but this graph metadata also encompasses everything else you might want to say about a graph: information about who can change parts of the graph, which fields contain PII, who the graph owner for some subgraph is, etc.

This document is an attempt to answer a few questions: *where* does the atlas store this data, *how*, and most importantly *why*?

## Problems

Last bit first: *why*? What problems does the atlas solve?

The atlas describes points in the graph.

&#8220;A point in the graph&#8221; might be represented in many ways, for example:
- a token within a schema file
- a token within a query against a particular schema
- the type or other schema element associated with some universal resource identifier (an atlas URL, for example).

For each point in the graph, there are various things we'll want to know:
- for types:
    - the names of accessible fields and their types
    - the (much larger) set of accessible extension fields which may not have bare names but are valid at this point
    - (note that the above effectively answers the question of where I can go when standing on a type)
- the scalar information (format, raw byte size) at this point (for scalar fields)
- metadata annotations applied through directives or elsewhere, such as `@private`, `@userdata`, `@external`, etc

This is the most general problem statement for the atlas. Here are some more specific problems which fall out of this big one:

### Resolve references in schema files

Consider [this bit of gql]('./fixtures/basic/admin.gql):

```graphql
extend type User @from(type: "users.User") {
  role: Role
}
```

`@from(type: "users.User")` references a type from somewhere in the graph. Maybe it's defined in this file. Maybe it's defined in a nearby file. Or maybe it's defined extremely elsewhere, as part of another graph with wholly different owners. Regardless, this is the question for us: what does `users.User` mean? Does it refer to anything, and what is that thing, what fields does it have, what endpoints can serve it, and so on?

In particular, the compiler needs to be able to retrieve at least:
- **a uri.** some universal identifier for the resolved type distinguishing it from all other types named `User`, in this or any other atlas.
- **a set of fields for this type.** including all applied extensions, and so on. At a minimum, the compiler needs this information in order to emit the type into the generated `gql`
- **metadata about this type (i.e. directives).** which may alter the way it's compiled
- **transitive dependencies.** all of the above for types referenced from every field, and their fields, and so on.

### Support language servers

A (relatively straightfoward) generalization of [resolving references](#resolve-references-in-schema-files) is to be able to identify and describe any token in any schema file on disk. This is effectively the core work of a language server. It makes sense to wrap this into the atlas' responsibilities. The atlas *isn't* a language server (in the sense of implementing the language server protocol), but it should be able to form the core of one.

### Provide forge with canonical representations of graph structures

Two pieces to this:

**Input.** In order to compose a schema, the compiler needs to reference other existing compiled graphs. This graph structure data needs to be represented in some way for transmission and caching.

**Output.** The compiler needs to emit compiled graphs in some form.

In other words, we an interchange format for graph structure data. We're being vague as to how right now—it could be a `.gql` file with directive, it could be some other format. Regardless, the atlas is responsible for defining what this is.

### Provide flexible mechanisms to externally tag graph structures

That is, we will want to mark pieces of the graph in arbitrary ways. For example, an organization may want to mark certain fields as containing personally identifying information. Naturally, you can do this with a custom directive on the field:

```graphql
type User {
  address: String @userdata(privacy: PII)
}
```

But constellation also allows us to do this in another schema entirely:

```graphql
extend type users.User {
  users.address: String @userdata(privacy: PII)
}
```

Here we're using hypothetical starql syntax, but it should be possible for such post-schema directives to be applied by other tooling as well.

## Structure

### Graphs are made of Points

We use the term *point* to refer to all the stuff in a graph. These are all points:
- types
- inputs
- interfaces
- fields
- unions
- enums
- scalars
- directives
- schemas

### `atlas:` URLs uniquely address points

Technically atlas URLs are URIs or maybe URNs but [WHATWG basically thinks the distinction is unnecessary and confusing](https://url.spec.whatwg.org/#goals) so we'll just follow their lead and call them URLs.

They take this form:

```
atlas:specifier:path?layer
```

Every part except the initial `atlas:` is optional.

Some examples:

```
# these are all in the main atlas

## a schema
atlas::spotify

## a point within the schema
atlas::spotify.User

## a field
### note: still thinking about the syntax here
atlas::spotify.id/spotify.User=spotify.ID

## a scalar
atlas::genius.ID

## an extension field
atlas::lyrical.geniusId/spotify.Song=genius.ID

# these are in different atlases
## identified by aliases
atlas:corp-internal:svc.secrets/User=String

## identified by atlas id
atlas:[3deff29adabade98f3234acb9320]:svc.secrets/User=String
```

#### Atlas URLs start off by specifying the atlas

Atlases can be specified either by *id* or *alias*. IDs are specified in `[brackets]`, aliases are bare words:

Specified by id:
```
atlas:[3deff29adabade98f3234acb9320]:
atlas:[9eff9asdd9329d89d9cbd9cf3dde]:
```

Specified by alias:
```
atlas:main:
atlas:acme-corp-internal:
```

Aliases are useful only within a context in which they're defined—for example, in a workspace with a [`.atlas` directory](#atlas-root).

You can also omit the specifier:

```
atlas::
```

If omitted, the id of the public atlas will be used. These are equivalent:

```
atlas::
atlas:[3deff29adabade98f3234acb9320]:
# note, this is a lie 🖕🏽
# we have not created a public atlas yet
# therefore there is no public atlas id
# but once there is, it will go here.
```

**Discussion Q:** Another option here is that `atlas::` is equivalent to `atlas:main:` or some other alias. 

#### Dotted paths identify points

```
atlas::genius.User
atlas::nyt.Article
```

#### Edges are located within their declaring schema

All `id` fields declared on any type *by* the `genius` schema:

```
atlas::genius.id
```

A field declared in `genius` named `id` on the type `genius.User`:

```
atlas::genius.id/genius.User
```

A field `User.id` declared in `genius` on `genius.User` returning a `genius.ID`:

```
atlas::genius.id/genius.User=genius.ID
```

Why is the namespaced field name first? Atlas URLs organize themselves so that the *originator of a piece of graph structure* is first. So if `lyrical` extends `spotify.Song` with a `geniusSong` field of type `genius.Song`, we'd represent that like so:

```
atlas::lyrical.geniusSong/spotify.Song=genius.Song
```

Why? Because atlases control access by granting users the ability to write different parts of the namespace. I, as the author of lyrical, have the ability to write to the `lyrical` namespace, but not to the `spotify` namespace. Therefore, the edges I declare must be within my namespace, even if they operate on another type.

#### URLs end with the layer

```
atlas::schema.Type?path=file://some/wkspce/schema.stars
atlas::schema.Type?path=file://some/wkspce/schema.stars&ver=2
```

Layers are described in more detail in the [next section](#atlas-layers).

### Layers

When accessed from some particular client, atlas data is likely strewn across various locales. 

For example, when compiling types against the public atlas, the layers might include:
  - schema files in the workspace
  - the user's global cache in ~/.atlas
  - the public atlas in heaven

In another instance, we might be using the atlas within a language server. The LSP provides access to unsaved content in the user's editor, which we can represent as another layer within the atlas.

For many purposes, it's not important to distinguish between different layers of an atlas. But in other cases, it might—for instance, if the user's editor shows a diff between two versions of a schema file, we'll need different atlas URLs to identify each of them in order to parse and link them separately and show the appropriate language annotations for each.

### Schema

Details TK, the atlas provides [a schema](./primer/atlas.stars) very much like the introspection schema, but extended with:
  - the ability to track directives and where they're applied
  - a `URL` scalar which holds an atlas url
  - `id: URL` fields on all schema elements
  - more `Query` methods

Additional schemas can compose with the atlas core schema, providing queries by e.g. fileystem path and token position, or the ability to provide announcements

## Filesystem binding

### Resolving references

Consider [this gql]('./fixtures/basic/admin.gql):

```graphql
extend type User @from(type: "users.User") {
  role: Role
}
```

Or the equivalent starql:

```graphql
extend type users.User {
  role: Role
}
```

In either case, to complete the type `User`, the compiler needs to resolve `users.User`. This happens in two steps:

1. first, we look up the first part of the dotted path—`user`, by scanning up the [scope chain](#scope-chain) until a scope is found in which the name `user` has been defined.
2. then, we descend into that definition, looking for a `User` element.

### Scope chain

The scope chain consists of:

1. For field references within a type or extension, the **type namespace**, containing:
    - all fields explicitly declared by the type or type extension
    - all names introduced with `let`
2. The **schema namespace**, containing:
    - all types explicitly declared in the schema file
    - all names introduced with `let`
3. For each ancestor directory up to the [atlas root](#atlas-root), the namespace of that directory, containing:
    - the base names (extension stripped) of all files with recognized schema extensions: `.stars`, `.gql`, and `.graphql`. It is an error for two sibling schemas to share a base name (i.e. you cannot have `users.stars` and `users.gql` as siblings in the same directory); the compiler will warn about this situation and refuse to resolve the name entirely.
    - the names of any directories containing 
    
    
### atlas root

A `.atlas` directory marks the root* of a constellation project. (* — Unless configured to be a child, by setting `child` in `atlas.toml` to `true`; sub-constellations configured as children allow name lookups continue up the directory tree).

Specifically, an `.atlas` directory contains an `atlas.toml` file, which looks like [this](./.atlas/atlas.toml):

```toml
# should atlas lookup proceed up the directory structure?
#   default: false
child = false

# default atlas to use when none is specified.
#   default: “main”
default = "main"

[atlases.main]
  id = "31498eaba8eb44d5a4ecc4acf7471469"
  url = "https://atlas.apollo.dev"

[atlases.internal]
  id = "a223bbc2347dfaefsdf3fefdssdac382"
  url = "https://some-internal-atlas.somewhere"
```

This `atlas.toml` defines two atlases: `main` and `internal`.

`main` is the default atlas (specified explicitly here, though that’s unnecessary, as it’s the default). There’s nothing special about main atlas, really, except that it’s the atlas that’s used if no atlas is specified.

`.atlas` directories may have a cache. If present, the cache is treated as read only unless explicitly updated.

### global atlas

The global atlas lives in `~/.atlas`. It has the same structure as any other `.atlas` folder, with the exception that the cache is always present.

### atlas cache

TK, but generally we use cacache holding the current state for each atlas under its proper uri as a key. The state is a lemma which references others, also held in the cache under their own hash.

## Distributed data format

We can represent an atlas as a collection of signed structs. These are sketched in [data.stars](./data.stars).

The first such struct an *axiom*. It's nothing more than a message describing the atlas and signed by some identity. The hash of the entire axiom becomes the atlas' id.

From there, we can change the atlas by creating *lemmas*. A lemma is a signed message describing some change to the atlas. For example, lemmas  can `Grant` other identities write access to namespaces within the atlas, or `Revoke` the same. We can also `Announce` endpoints for schemas, create lemmas for directive metadata, and so on.

This structure gives us something important: a clearly defined, tamperproof chain of custody for atlas data.