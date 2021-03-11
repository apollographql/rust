//! Constants, mainly supported versions of the `using` spec which we can
//! reference during bootstrapping.

use std::borrow::Cow;

use crate::{spec::Spec, Version};

pub const CORE: Spec = Spec {
    identity: Cow::Borrowed("https://lib.apollo.dev/core"),
    name: Cow::Borrowed("core"),
    version: Version(0, 1),
};
