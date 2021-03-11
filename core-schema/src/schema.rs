//! A GraphQL schema referencing one or more specifications
//! via [the `@using` directive](https://apollo-specs.github.io/using/draft/pre-0).
//!
//! You can parse GraphQL IDL into a `Schema` with [`Schema::parse`](Schema.html#parse):
//!
//! ```
//! use std::borrow::Cow;
//! use core_schema::*;
//!
//! let schema = Schema::parse(r#"
//!     schema
//!       @core(feature: "https://lib.apollo.dev/core/v0.1")
//!       @core(feature: "https://spec.example.com/A/v1.0")
//!     {
//!       query: Query
//!     }
//! "#)?;
//! # Ok::<(), graphql_parser::ParseError>(())
//! ```
//!
//! The `using` field on the returned `Schema` is a list of `Request`s,
//! each one a spec requested by the document:
//!
//! ```
//! # use std::borrow::Cow;
//! # use core_schema::*;
//!
//! # let schema = Schema::parse(r#"
//! #     schema
//! #       @core(feature: "https://lib.apollo.dev/core/v0.1")
//! #       @core(feature: "https://spec.example.com/A/v1.0")
//! #     {
//! #       query: Query
//! #     }
//! # "#)?;
//! assert_eq!(schema.features, vec![
//!     Feature {
//!         spec: Spec::new("https://lib.apollo.dev/core", "core", (0, 1)),
//!         name: Cow::Borrowed("core"),
//!         position: Pos { line: 3, column: 7 },
//!     },
//!     Feature {
//!         spec: Spec::new("https://spec.example.com/A", "A", (1, 0)),
//!         name: Cow::Borrowed("A"),
//!         position: Pos { line: 4, column: 7 },
//!     },
//! ]);
//! # Ok::<(), graphql_parser::ParseError>(())
//! ```

use std::{borrow::Cow, collections::HashMap};

use graphql_parser::{
    parse_schema,
    schema::{Definition, Document, SchemaDefinition},
    ParseError, Pos,
};

use thiserror::Error;

use crate::{constants::CORE, spec, Feature, Find, Found, Implementations, Version};

/// A Schema holds a parsed GraphQL schema document and the features in that document,
/// along with any errors which occurred during validation.
pub struct Schema<'a> {
    /// the document AST, as parsed by graphql_parser
    pub document: Document<'a>,

    /// features in the document
    pub features: Vec<Feature>,

    /// validation errors
    pub errors: Vec<SchemaError>,
}

impl<'a> Schema<'a> {
    /// Parse a schema source string and return a validated `Schema` with all
    /// `Request`s identified, or fail with a `graphql_parser::ParseError`.
    ///
    /// As indicated by the type, this fn will always return `Ok` if parsing
    /// succeeds. Subsequent errors are reported in the `errors` field of the
    /// returned schema.
    ///
    /// # Example:
    ///
    /// ```
    /// use core_schema::{Schema, Feature, Spec, Version};
    /// use graphql_parser::{ParseError, Pos};
    /// use std::borrow::Cow;
    ///
    /// let schema = Schema::parse(r#"
    ///     schema
    ///       @core(feature: "https://lib.apollo.dev/core/v0.1")
    ///       @core(feature: "https://spec.example.com/A/v1.2", as: "exampleA")
    ///       @core(feature: "https://example.net/specB/v0.1")
    ///     {
    ///       query: Query
    ///     }
    /// "#)?;
    ///
    /// assert_eq!(schema.errors, vec![]);
    /// assert_eq!(schema.features, vec![
    ///    Feature {
    ///        spec: Spec::new("https://lib.apollo.dev/core", "core", (0, 1)),
    ///        name: Cow::Borrowed("core"),
    ///        position: Pos { line: 3, column: 7 },
    ///    },        
    ///    Feature {
    ///        spec: Spec::new("https://spec.example.com/A", "A", (1, 2)),
    ///        name: Cow::Borrowed("exampleA"),
    ///        position: Pos { line: 4, column: 7 },
    ///    },
    ///    Feature {
    ///        spec: Spec::new("https://example.net/specB", "specB", (0, 1)),
    ///        name: Cow::Borrowed("specB"),
    ///        position: Pos { line: 5, column: 7 },
    ///    },
    /// ]);
    /// # Ok::<(), ParseError>(())
    /// ```
    pub fn parse(source: &'a str) -> Result<Schema<'a>, ParseError> {
        let document = parse_schema(source)?;
        let mut errors = Vec::new();

        let mut schemas: Vec<_> = document
            .definitions
            .iter()
            .filter_map(|def| match def {
                Definition::Schema(sd) => Some(sd),
                _ => None,
            })
            .collect();

        if schemas.is_empty() {
            errors.push(SchemaError::NoSchemas);

            // We only return `Err` for actual parse errors. Validation failures
            // such as NoSchemas are available on the `errors` field.
            return Ok(Schema {
                document,
                features: vec![],
                errors,
            });
        }
        let schema = schemas.remove(0);
        for extraneous in schemas {
            errors.push(SchemaError::ExtraSchema(extraneous.position));
        }

        // Collect everything which *could be* a core feature declaration.
        // "Could be a core feature" means it's a directive on
        // the schema which has a `spec: String` argument. We consider
        // as candidates requests whose `spec` does not parse and retain
        // error information so we can provide it later.
        //
        // Notably not checked here is the name of the directive.
        // That's because @core might itself be prefixed (and thus, named
        // something else!) We'll filter these candidates down when we
        // discover using's true prefix in subsequent boostrapping phase.
        let mut features: Vec<_> = schema
            .directives
            .iter()
            .filter_map(|dir|
                // Parse as a using request and retain the directive.
                // This filters out any directives which do not have a spec: argument at all.
                Feature::from_directive(dir)
                    .map(|res| (dir, res)))
            .collect();

        let using_features = match bootstrap(&schema) {
            Some(req) => req,
            None => {
                errors.push(SchemaError::NoCore);
                return Ok(Schema {
                    document,
                    features: vec![],
                    errors,
                });
            }
        };

        let mut using = vec![];

        for (dir, req) in features.drain(0..) {
            if dir.name != using_features.name {
                continue;
            }
            match req {
                Ok(req) => using.push(req),
                Err(err) => errors.push(SchemaError::BadUsingRequest(dir.position, err)),
            }
        }

        let mut machined = Schema {
            document,
            features: using,
            errors,
        };
        machined.validate_no_overlapping_prefixes();
        Ok(machined)
    }

    /// Validate that no two `Feature`s are using the same `prefix`, removing
    /// *all* overlapping `Feature`s from `self.features` and reporting
    /// `SchemaError::OverlappingPrefix` in `self.errors`.
    fn validate_no_overlapping_prefixes(&mut self) {
        let mut by_prefix = HashMap::<_, u32>::new();
        for req in &self.features {
            let count = by_prefix.entry(req.name.clone()).or_default();
            *count += 1;
        }
        for (prefix, count) in by_prefix.drain() {
            if count > 1 {
                self.errors.push(SchemaError::OverlappingPrefix(
                    prefix.clone(),
                    drain_filter_collect(&mut self.features, |req| req.name == *prefix),
                ));
            }
        }
    }

    /// Given a set of implementations, take all features in the document and
    /// match them to the implementations which satisfy them ("satisfying
    /// implementations").
    ///
    /// This returns an `Iterator` over `(&Feature, Find<T>)`.
    /// `Find` is an `Iterator` over `(&Version, &T)`, which will iterate satisfying
    /// implementations ordered by lowest to highest version. The
    /// [`bound`](Bounds.html#bound) method can be used to get the lowest and highest
    /// satisfying versions.
    ///
    /// # Example:
    ///
    /// ```
    /// use core_schema::*;
    ///
    /// let impls = Implementations::new()
    ///     .provide("https://spec.example.com/A", Version(1, 2), "impl for A 1.2")
    ///     .provide("https://spec.example.com/A", Version(1, 3), "impl for A 1.3")
    ///     .provide("https://spec.example.com/A", Version(1, 7), "impl for A 1.7");
    /// let schema = Schema::parse(r#"schema @core(feature: "https://lib.apollo.dev/core/v0.1") @core(feature: "https://spec.example.com/A/v1.0") { query: Query }"#)?;
    ///
    /// let max = schema
    ///   .activations(&impls)
    ///   .filter_map(|(_request, mut impls)| impls.bounds())
    ///   .collect::<Vec<_>>();
    ///
    /// assert_eq!(max, vec![(
    ///     (&Version(1, 2), &"impl for A 1.2"), (&Version(1, 7), &"impl for A 1.7")
    /// )]);
    /// # Ok::<(), GraphQLParseError>(())
    /// ```
    pub fn activations<'impls, T>(
        &'impls self,
        implementations: &'impls Implementations<T>,
    ) -> impl Iterator<
        Item = (
            &'a Feature,
            Find<'impls, T, impl Iterator<Item = Found<'impls, T>>>,
        ),
    > + 'impls
    where
        'impls: 'a,
        T: 'impls,
    {
        self.features
            .iter()
            .map(move |request| (request, implementations.find_feature(request)))
    }
}

/// Drain all items from a `Vec<T>` which match `pred` and collect the results
///
/// Equivalent to (and probably slower than) `vec.drain_filter(pred).collect()`,
/// which we should switch to once it stabilizes.
fn drain_filter_collect<T, F: Fn(&T) -> bool>(vec: &mut Vec<T>, pred: F) -> Vec<T> {
    let mut i = 0;
    let mut collected = vec![];
    while i != vec.len() {
        if pred(&vec[i]) {
            collected.push(vec.remove(i));
        } else {
            i += 1;
        }
    }
    collected
}

// Given a `SchemaDefinition`, locate the first `@core` request, which must be for
// the core spec itself.
fn bootstrap(schema: &SchemaDefinition) -> Option<Feature> {
    schema
        .directives
        .iter()
        // Scan requests which parsed without error
        .filter_map(|dir| Feature::from_directive(dir).map(|res| (dir, res)))
        .filter_map(|(dir, res)| res.ok().map(|req| (dir, req)))
        .find_map(|(dir, req)| {
            if
            // We're looking for bootstrap @core directives, so select only
            // those requests whose requested name matches the directive name...
            dir.name == req.name &&
                // ...and which are requesting a the core spec...
                req.spec.identity == CORE.identity &&
                // ...at a supported version.
                CORE.version.satisfies(&req.spec.version)
            {
                Some(req)
            } else {
                None
            }
        })
}
/// Validation errors which may occur on a `Schema`
#[derive(Debug, Error, PartialEq, Eq)]
pub enum SchemaError {
    /// Machine schemas must contain exactly one schema definition.
    #[error("no schema definitions found in document")]
    NoSchemas,

    /// No reference to the core spec found
    #[error("no reference to the core spec found")]
    NoCore,

    /// Multiple schema definitions were found in the document
    /// Extra schema definitions are reported as these errors.
    #[error("{} extra schema definition ignored", .0)]
    ExtraSchema(Pos),

    /// Multiple versions of @using were requested
    /// by the document. One will be selected, the others
    /// will generate these errors.
    #[error("{} extra using spec ignored", .0)]
    ExtraUsing(Pos, Version),

    /// A request in the document failed to parse.
    #[error("{} bad spec url: {}", .0, .1)]
    BadUsingRequest(Pos, spec::SpecParseError),

    /// Document is using multiple specs with the same prefix.
    #[error("multiple requests using the {} prefix:\n{}", .0, self.requests())]
    OverlappingPrefix(Cow<'static, str>, Vec<Feature>),
}

impl SchemaError {
    /// Collect `Request`s referenced by this error and join their position, urls,
    /// and versions into an indented, "\n"-separated String for purposes of
    /// displaying the error.
    ///
    /// Returns the empty string if this error does not reference any `Request`s.
    fn requests(&self) -> String {
        match self {
            Self::OverlappingPrefix(_, requests) => requests
                .iter()
                .map(|req| {
                    format!(
                        "    {pos}: {url}/{ver}",
                        pos = req.position,
                        url = &req.spec.identity,
                        ver = &req.spec.version,
                    )
                })
                .collect::<Vec<String>>()
                .join("\n"),
            _ => "".to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Implementations, Schema};
    use graphql_parser::ParseError;
    use insta::{assert_debug_snapshot, assert_snapshot};

    macro_rules! assert_schema_snapshots {
        ($source:expr) => {
            assert_debug_snapshot!({
                let schema = Schema::parse($source)?;
                (schema.errors, schema.features)
            });
        };
    }

    #[test]
    pub fn it_identifies_one_spec() -> Result<(), ParseError> {
        assert_schema_snapshots!(
            r#"
            schema
                @core(feature: "https://lib.apollo.dev/core/v0.1")
                @core(feature: "https://spec.example.com/A/v1.0")
            {
                query: Query
            }
        "#
        );
        Ok(())
    }

    #[test]
    pub fn it_identifies_several_specs() -> Result<(), ParseError> {
        assert_schema_snapshots!(
            r#"
            schema
                @core(feature: "https://lib.apollo.dev/core/v0.1")
                @core(feature: "https://spec.example.com/A/v1.0")
                @core(feature: "https://spec.example.com/B/v0.0", as: "specB")
                @core(feature: "https://spec.example.com/C/v21.913")
            {
                query: Query
            }
        "#
        );
        Ok(())
    }

    #[test]
    pub fn it_can_load_using_with_a_different_prefix() -> Result<(), ParseError> {
        assert_schema_snapshots!(
            r#"
            schema
                @req(feature: "https://lib.apollo.dev/core/v0.1", as: "req")
                @req(feature: "https://spec.example.com/A/v1.0")
                @core(feature: "https://spec.example.com/B/v2.2")
                {
                    query: Query
                }
        "#
        );

        Ok(())
    }

    #[test]
    pub fn it_errors_when_no_schema_defs_are_present() -> Result<(), ParseError> {
        assert_schema_snapshots!(
            r#"
            type User {
                id: ID!
            }
        "#
        );
        Ok(())
    }

    #[test]
    pub fn it_errors_when_multiple_schema_defs_are_present() -> Result<(), ParseError> {
        assert_schema_snapshots!(
            r#"
            schema
                @core(feature: "https://lib.apollo.dev/core/v0.1")
                @core(feature: "https://spec.example.com/A/v2.2")
            {
                query: Query
            }
            
            schema {
                query: Query
            }
        "#
        );
        Ok(())
    }

    #[test]
    pub fn it_errors_when_default_prefixes_overlap() -> Result<(), ParseError> {
        assert_schema_snapshots!(
            r#"
            schema
                @core(feature: "https://lib.apollo.dev/core/v0.1")
                @core(feature: "https://spec.example.com/A/v1.0")
                @core(feature: "https://spec.example.com/B/v2.3")
                @core(feature: "https://somewhere-else.specs.com/A/v1.0")
            {
                query: Query
            }
        "#
        );
        Ok(())
    }

    #[test]
    pub fn it_errors_when_non_default_prefixes_overlap() -> Result<(), ParseError> {
        assert_schema_snapshots!(
            r#"
            schema
                @core(feature: "https://lib.apollo.dev/core/v0.1")
                @core(feature: "https://spec.example.com/A/v1.0")
                @core(feature: "https://somewhere-else.specs.com/myspec/v1.0", as: "A")
            {
                query: Query
            }
        "#
        );
        Ok(())
    }

    #[test]
    pub fn it_is_ok_when_prefixes_are_disambugated() -> Result<(), ParseError> {
        assert_schema_snapshots!(
            r#"
            schema
                @core(feature: "https://lib.apollo.dev/core/v0.1")
                @core(feature: "https://spec.example.com/A/v1.0")
                @core(feature: "https://spec.example.com/B/v2.3")
                @core(feature: "https://somewhere-else.specs.com/A/v1.0", as: "otherA")
            {
                query: Query
            }
        "#
        );
        Ok(())
    }

    #[test]
    fn it_iterates_over_activations() -> Result<(), ParseError> {
        assert_snapshot!({
            let implementations = Implementations::new()
                .provide(
                    "https://spec.example.com/A",
                    (1, 2),
                    "impl A v1.2".to_owned(),
                )
                .provide(
                    "https://spec.example.com/B",
                    (1, 2),
                    "impl B v1.2".to_owned(),
                );

            let schema = Schema::parse(
                r#"
                schema
                    @core(feature: "https://lib.apollo.dev/core/v0.1")
                    @core(feature: "https://spec.example.com/A/v1.0")
                    @core(feature: "https://spec.example.com/unknown/v1.0")
                {
                    query: Query
                }
                "#,
            )?;

            let activations = schema
                .activations(&implementations)
                .map(|(req, find)| (req, find.collect::<Vec<_>>()))
                .collect::<Vec<_>>();

            format!("{:#?}", activations)
        });

        Ok(())
    }

    #[test]
    fn it_takes_arbitrary_types_as_implementations() -> Result<(), ParseError> {
        let implementations = Implementations::new()
            .provide(
                "https://spec.example.com/A",
                (1, 2),
                Box::<&dyn Fn() -> String>::new(&|| "impl A v1.2".to_owned()),
            )
            .provide(
                "https://spec.example.com/B",
                (1, 2),
                Box::<&dyn Fn() -> String>::new(&|| "impl B v1.2".to_owned()),
            );
        let output = Schema::parse(
            r#"
            schema
                @core(feature: "https://lib.apollo.dev/core/v0.1")
                @core(feature: "https://spec.example.com/A/v1.0")
                @core(feature: "https://spec.example.com/unknown/v1.0")
            {
                query: Query
            }
        "#,
        )?
        .activations(&implementations)
        .map(|(_req, find)| find.last().map(|(_ver, f)| f()))
        .collect::<Vec<_>>();

        assert_eq!(output, vec![None, Some("impl A v1.2".to_owned()), None,]);

        Ok(())
    }
}
