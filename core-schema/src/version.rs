use std::{fmt::Display, num::ParseIntError, str::FromStr};

use lazy_static::lazy_static;
use regex::Regex;
use thiserror::Error;

/// Versions are a (major, minor) u64 pair.
///
/// Versions implement `PartialOrd` and `Ord`, which orders them by major and then
/// minor version. Be aware that this ordering does *not* imply compatibility. For
/// example, `Version(2, 0) > Version(1, 9)`, but an implementation of `Version(2, 0)`
/// *cannot* satisfy a request for `Version(1, 9)`. To check for version compatibility,
/// use [the `satisfies` method](#satisfies).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version(pub u64, pub u64);

impl Version {
    /// Parse a version specifier of the form "v(major).(minor)"
    ///
    /// # Example
    /// ```
    /// use core_schema::Version;
    /// assert_eq!(Version::parse("v1.0")?, Version(1, 0));
    /// assert_eq!(Version::parse("v0.1")?, Version(0, 1));
    /// assert_eq!(Version::parse("v987.65432")?, Version(987, 65432));
    /// # Ok::<(), core_schema::VersionParseError>(())
    /// ```
    pub fn parse(input: &str) -> Result<Version, VersionParseError> {
        lazy_static! {
            static ref VERSION_RE: Regex = Regex::new(r#"^v(\d+)\.(\d+)$"#).unwrap();
        }

        if let Some(ver) = VERSION_RE.captures(input) {
            Ok(Version(ver[1].parse()?, ver[2].parse()?))
        } else {
            Err(VersionParseError)
        }
    }

    /// Return true if and only if this Version satisfies the `required` version
    ///
    /// # Example
    /// ```
    /// use core_schema::Version;
    /// assert!(Version(1, 0).satisfies(&Version(1, 0)));
    /// assert!(Version(1, 2).satisfies(&Version(1, 0)));
    /// assert!(!Version(2, 0).satisfies(&Version(1, 9)));
    /// ```
    pub fn satisfies(&self, required: &Version) -> bool {
        let Version(major, minor) = *self;
        let Version(r_major, r_minor) = *required;
        r_major == major
            && (if major == 0 {
                r_minor == minor
            } else {
                r_minor <= minor
            })
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "v{major}.{minor}",
            major = self.0,
            minor = self.1
        ))
    }
}

impl FromStr for Version {
    type Err = VersionParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Version::parse(s)
    }
}

impl From<(u64, u64)> for Version {
    fn from((major, minor): (u64, u64)) -> Self {
        Self(major, minor)
    }
}

#[derive(Error, Debug, Clone, Copy)]
#[error("error parsing version specifier")]
pub struct VersionParseError;

impl From<ParseIntError> for VersionParseError {
    fn from(_: ParseIntError) -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::{Version, VersionParseError};

    #[test]
    fn it_parses_valid_version_specifiers() -> Result<(), VersionParseError> {
        assert_eq!(Version::parse("v0.0")?, Version(0, 0));
        assert_eq!(Version::parse("v1.0")?, Version(1, 0));
        assert_eq!(Version::parse("v99.0")?, Version(99, 0));
        assert_eq!(Version::parse("v2.3")?, Version(2, 3));
        assert_eq!(Version::parse("v12.34")?, Version(12, 34));
        assert_eq!(Version::parse("v987.654")?, Version(987, 654));
        Ok(())
    }

    #[test]
    fn it_errors_on_invalid_specifiers() {
        assert!(matches!(Version::parse("bloop"), Err(VersionParseError)));
        assert!(matches!(Version::parse("v0."), Err(VersionParseError)));
        assert!(matches!(Version::parse("v0.?"), Err(VersionParseError)));
        assert!(matches!(Version::parse("v1.x"), Err(VersionParseError)));
        assert!(matches!(
            Version::parse("v0.1-tags_are_not_supported"),
            Err(VersionParseError)
        ));
    }

    #[test]
    fn it_still_parses_version_specifiers_which_are_slightly_out_of_spec(
    ) -> Result<(), VersionParseError> {
        assert_eq!(Version::parse("v01.0002")?, Version(1, 2));
        Ok(())
    }

    #[test]
    fn it_compares_minor_version_differences_ok() {
        assert!(Version(1, 5).satisfies(&Version(1, 0)));
        assert!(!Version(1, 0).satisfies(&Version(1, 1)));
    }

    #[test]
    fn it_compares_zerodot_series_version_differences_ok() {
        assert!(Version(0, 1).satisfies(&Version(0, 1)));
        assert!(!Version(0, 2).satisfies(&Version(0, 1)));
    }

    #[test]
    fn it_compares_major_version_differences_ok() {
        assert!(!Version(2, 2).satisfies(&Version(1, 2)));
    }

    #[test]
    fn it_formats_itself() {
        assert_eq!(format!("{}", Version(0, 1)), "v0.1");
        assert_eq!(format!("{}", Version(1234, 5678)), "v1234.5678");
    }
}
