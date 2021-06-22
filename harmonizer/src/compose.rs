/*!
# Compose graphql schemas
*/

use crate::js::Js;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use thiserror::Error;

/// The `ServiceDefinition` represents everything we need to know about a
/// service (subgraph) for its GraphQL runtime responsibilities.  It is not
/// at all different from the notion of [`ServiceDefinition` in TypeScript]
/// used in Apollo Gateway's operation.
///
/// Since we'll be running this within a JavaScript environment these properties
/// will be serialized into camelCase, to match the JavaScript expectations.
///
/// [`ServiceDefinition` in TypeScript]: https://github.com/apollographql/federation/blob/d2e34909/federation-js/src/composition/types.ts#L49-L53
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceDefinition {
    /// The name of the service (subgraph).  We use this name internally to
    /// in the representation of the composed schema and for designations
    /// within the human-readable QueryPlan.
    pub name: String,
    /// The routing/runtime URL where the subgraph can be found that will
    /// be able to fulfill the requests it is responsible for.
    pub url: String,
    /// The Schema Definition Language (SDL)
    pub type_defs: String,
}

impl ServiceDefinition {
    /// Create a new [`ServiceDefinition`]
    pub fn new<N: Into<String>, U: Into<String>, D: Into<String>>(
        name: N,
        url: U,
        type_defs: D,
    ) -> ServiceDefinition {
        ServiceDefinition {
            name: name.into(),
            url: url.into(),
            type_defs: type_defs.into(),
        }
    }
}

/// An ordered stack of the services (subgraphs) that, when composed in order
/// by the composition algorithm, will represent the supergraph.
pub type ServiceList = Vec<ServiceDefinition>;

/// An error which occurred during JavaScript composition.
///
/// The shape of this error is meant to mimick that of the error created within
/// JavaScript, which is a [`GraphQLError`] from the [`graphql-js`] library.
///
/// [`graphql-js']: https://npm.im/graphql
/// [`GraphQLError`]: https://github.com/graphql/graphql-js/blob/3869211/src/error/GraphQLError.js#L18-L75
#[derive(Debug, Error, Serialize, Deserialize, PartialEq)]
pub struct CompositionError {
    /// A human-readable description of the error that prevented composition.
    pub message: Option<String>,
    /// [`CompositionErrorExtensions`]
    pub extensions: Option<CompositionErrorExtensions>,
}

impl Display for CompositionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(msg) = &self.message {
            f.write_fmt(format_args!("{code}: {msg}", code = self.code(), msg = msg))
        } else {
            f.write_str(self.code())
        }
    }
}

/// Mimicking the JavaScript-world from which this error comes, this represents
/// the `extensions` property of a JavaScript [`GraphQLError`] from the
/// [`graphql-js`] library. Such errors are created when errors have prevented
/// successful composition, which is accomplished using [`errorWithCode`]. An
/// [example] of this can be seen within the `federation-js` JavaScript library.
///
/// [`graphql-js']: https://npm.im/graphql
/// [`GraphQLError`]: https://github.com/graphql/graphql-js/blob/3869211/src/error/GraphQLError.js#L18-L75
/// [`errorWithCode`]: https://github.com/apollographql/federation/blob/d7ca0bc2/federation-js/src/composition/utils.ts#L200-L216
/// [example]: https://github.com/apollographql/federation/blob/d7ca0bc2/federation-js/src/composition/validate/postComposition/executableDirectivesInAllServices.ts#L47-L53
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct CompositionErrorExtensions {
    /// An Apollo Federation composition error code.
    ///
    /// A non-exhaustive list of error codes that this includes, is:
    ///
    ///   - EXTERNAL_TYPE_MISMATCH
    ///   - EXTERNAL_UNUSED
    ///   - KEY_FIELDS_MISSING_ON_BASE
    ///   - KEY_MISSING_ON_BASE
    ///   - KEY_NOT_SPECIFIED
    ///   - PROVIDES_FIELDS_MISSING_EXTERNAL
    ///
    /// ...and many more!  See the `federation-js` composition library for
    /// more details (and search for `errorWithCode`).
    pub code: String,
}

/// An error that was received during composition within JavaScript.
impl CompositionError {
    /// Retrieve the error code from an error received during composition.
    pub fn code(&self) -> &str {
        match self.extensions {
            Some(ref ext) => &*ext.code,
            None => "UNKNOWN",
        }
    }
}

/// The `compose` function receives a [`ServiceList`] and invokes JavaScript
/// composition on it.
///
pub fn compose(service_list: ServiceList) -> Result<String, Vec<CompositionError>> {
    Js::new()
        .with_parameter("serviceList", service_list)
        .execute("do_plan", include_str!("../js/do_compose.js"))
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        use crate::compose::{compose, ServiceDefinition};

        insta::assert_snapshot!(compose(vec![
            ServiceDefinition::new(
                "users",
                "undefined",
                "
            type User {
              id: ID
              name: String
            }

            type Query {
              users: [User!]
            }
          "
            ),
            ServiceDefinition::new(
                "movies",
                "undefined",
                "
            type Movie {
              title: String
              name: String
            }

            extend type User {
              favorites: [Movie!]
            }

            type Query {
              movies: [Movie!]
            }
          "
            )
        ])
        .unwrap());
    }
}
