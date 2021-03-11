//! Spec url handling
//!
//! `Spec`s are parsed from URL strings and extract the spec's:
//!   - **identity**, which is the URL excluding the version specifier,
//!   - **name**, which is the second-to-last path segment of the URL,
//!     (typically the name of the bare directive exported by the spec), and
//!   - **version**, specified in the last URL path segment.
//!
//! # Example:
//! ```
//! use core_schema::*;
//! assert_eq!(
//!   Spec::parse("https://spec.example.com/specA/v1.0")?,
//!   Spec::new("https://spec.example.com/specA", "specA", (1, 0))
//! );
//! Ok::<(), SpecParseError>(())
//! ```
use std::borrow::Cow;

use thiserror::Error;
use url::Url;

use crate::{version, version::Version};

/// Specs contain the `identity`, `name`, and `version` extracted from
/// a spec URL.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spec {
    pub identity: Cow<'static, str>,
    pub name: Cow<'static, str>,
    pub version: Version,
}

#[derive(Debug, PartialEq, Eq, Clone, Error)]
pub enum SpecParseError {
    #[error("error parsing url")]
    UrlParseError(url::ParseError),

    #[error("error parsing version specifier")]
    VersionParseError,

    #[error("url does not have a path")]
    NoPath,

    #[error("url does specify a version")]
    NoVersion,

    #[error("url does not specify a prefix")]
    NoDefaultPrefix,
}

impl Spec {
    /// Construct a Spec with identity, name, version
    pub fn new<I, N, V>(identity: I, name: N, version: V) -> Self
    where
        I: Into<Cow<'static, str>>,
        N: Into<Cow<'static, str>>,
        V: Into<Version>,
    {
        Self {
            identity: identity.into(),
            name: name.into(),
            version: version.into(),
        }
    }

    /// Parse a spec URL
    pub fn parse(input: &str) -> Result<Spec, SpecParseError> {
        use SpecParseError::*;

        let mut parsed = Url::parse(input)?;
        let mut path: Vec<_> = parsed
            .path_segments()
            .ok_or(NoPath)?
            .map(|x| x.to_owned())
            .collect();
        let version = path.pop().ok_or(NoVersion)?;
        let version = Version::parse(&version)?;
        let name = path.last().ok_or(NoDefaultPrefix)?;
        parsed.set_fragment(None);
        let _ = parsed.set_username("");
        parsed.set_query(None);
        let _ = parsed.set_password(None);
        parsed.set_path(&path.join("/"));

        Ok(Self::new(parsed.to_string(), name.to_string(), version))
    }
}

impl From<url::ParseError> for SpecParseError {
    fn from(error: url::ParseError) -> Self {
        Self::UrlParseError(error)
    }
}

impl From<version::VersionParseError> for SpecParseError {
    fn from(_: version::VersionParseError) -> Self {
        Self::VersionParseError
    }
}
