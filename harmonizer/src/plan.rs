/*!
# Create a query plan
*/

use crate::js::Js;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use thiserror::Error;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
/// Options for the query plan
pub struct QueryPlanOptions {
    /// Use auto fragmentation
    pub auto_fragmentization: bool,
}

/// Default options for query planning
impl QueryPlanOptions {
    /// Default query plan options
    pub fn default() -> QueryPlanOptions {
        QueryPlanOptions {
            auto_fragmentization: false,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
/// The query to be planned
pub struct OperationalContext {
    /// The graphQL schema
    pub schema: String,
    /// The graphQL query
    pub query: String,
    /// The operation
    pub operation: String,
}

#[derive(Debug, Error, Serialize, Deserialize, PartialEq)]
/// Container for planning errors
pub struct PlanningErrors {
    /// The contained errors
    pub errors: Vec<PlanningError>,
}

impl Display for PlanningErrors {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "Planning errors: {}",
            self.errors
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        ))
    }
}

/// An error which occurred during JavaScript planning.
///
/// The shape of this error is meant to mimick that of the error created within
/// JavaScript.
///
/// [`graphql-js']: https://npm.im/graphql
/// [`GraphQLError`]: https://github.com/graphql/graphql-js/blob/3869211/src/error/GraphQLError.js#L18-L75
#[derive(Debug, Error, Serialize, Deserialize, PartialEq)]
pub struct PlanningError {
    /// A human-readable description of the error that prevented planning.
    pub message: Option<String>,
    /// [`planningErrorExtensions`]
    pub extensions: Option<PlanningErrorExtensions>,
}

impl Display for PlanningError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(msg) = &self.message {
            f.write_fmt(format_args!("{code}: {msg}", code = self.code(), msg = msg))
        } else {
            f.write_str(self.code())
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
/// Error codes
pub struct PlanningErrorExtensions {
    /// The error code
    pub code: String,
}

/// An error that was received during planning within JavaScript.
impl PlanningError {
    /// Retrieve the error code from an error received during planning.
    pub fn code(&self) -> &str {
        match self.extensions {
            Some(ref ext) => &*ext.code,
            None => "UNKNOWN",
        }
    }
}

/// Create the query plan by calling in to JS.
///
pub fn plan(
    context: OperationalContext,
    options: QueryPlanOptions,
) -> Result<String, PlanningErrors> {
    Js::new()
        .with_parameter("context", context)
        .with_parameter("options", options)
        .execute("do_plan", include_str!("../js/do_plan.js"))
        .map_err(|errors| PlanningErrors { errors })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCHEMA: &str = include_str!("testdata/schema.graphql");
    const QUERY: &str = include_str!("testdata/query.graphql");

    #[test]
    fn it_works() {
        insta::assert_snapshot!(plan(
            OperationalContext {
                schema: SCHEMA.to_string(),
                query: QUERY.to_string(),
                operation: "".to_string()
            },
            QueryPlanOptions::default()
        )
        .unwrap());
    }

    #[test]
    fn invalid_schema_is_caught() {
        let result = Err(PlanningErrors {
            errors: vec![PlanningError {
                message: Some("Syntax Error: Unexpected Name \"Garbage\".".to_string()),
                extensions: None,
            }],
        });
        assert_eq!(
            result,
            plan(
                OperationalContext {
                    schema: "Garbage".to_string(),
                    query: QUERY.to_string(),
                    operation: "".to_string(),
                },
                QueryPlanOptions::default(),
            )
        );
    }

    #[test]
    fn invalid_query_is_caught() {
        let result = Err(PlanningErrors {
            errors: vec![PlanningError {
                message: Some("Syntax Error: Unexpected Name \"Garbage\".".to_string()),
                extensions: None,
            }],
        });
        assert_eq!(
            result,
            plan(
                OperationalContext {
                    schema: SCHEMA.to_string(),
                    query: "Garbage".to_string(),
                    operation: "".to_string(),
                },
                QueryPlanOptions::default(),
            )
        );
    }
}
