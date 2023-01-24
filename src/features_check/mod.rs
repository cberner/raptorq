//! Shows a user-friendly compiler error on incompatible selected features.
//!
//! A neat checker shamelessly borrowed from `serde_json` crate.

#[allow(unused_macros)]
macro_rules! hide_from_rustfmt {
    ($mod:item) => {
        $mod
    };
}

#[cfg(not(any(feature = "std", feature = "metal")))]
hide_from_rustfmt! {
    mod error;
}
