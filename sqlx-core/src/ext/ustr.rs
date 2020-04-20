use std::borrow::Borrow;
use std::fmt::{self, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::sync::Arc;

// U meaning micro
// a micro-string is either a reference-counted string or a static string
// this guarantees these are cheap to clone everywhere
#[derive(Debug, Clone, Eq)]
pub(crate) enum UStr {
    Static(&'static str),
    Shared(Arc<str>),
}

impl Deref for UStr {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        match self {
            UStr::Static(s) => s,
            UStr::Shared(s) => s,
        }
    }
}

impl Hash for UStr {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Forward the hash to the string representation of this
        // A derive(Hash) encodes the enum discriminant
        (&**self).hash(state);
    }
}

impl Borrow<str> for UStr {
    #[inline]
    fn borrow(&self) -> &str {
        &**self
    }
}

impl PartialEq<UStr> for UStr {
    fn eq(&self, other: &UStr) -> bool {
        (**self).eq(&**other)
    }
}

impl From<&'static str> for UStr {
    #[inline]
    fn from(s: &'static str) -> Self {
        UStr::Static(s)
    }
}

impl From<String> for UStr {
    #[inline]
    fn from(s: String) -> Self {
        UStr::Shared(s.into())
    }
}

impl Display for UStr {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.pad(self)
    }
}
