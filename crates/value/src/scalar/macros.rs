/// A value::Scalar literal.
///
/// # Example
///
/// ```rust
/// # use liquid_value::ValueView;
/// #
/// # fn main() {
/// liquid_value::scalar::scalar!(5)
///     .to_integer().unwrap();
/// liquid_value::scalar::scalar!("foo")
///     .to_kstr();
/// # }
/// ```
#[macro_export(local_inner_macros)]
macro_rules! scalar {
    ($value:literal) => {
        $crate::Scalar::new($value)
    };

    ($other:ident) => {
        $other
    };

    // Any Serialize type: numbers, strings, struct literals, variables etc.
    // Must be below every other rule.
    ($other:expr) => {
        $crate::to_scalar(&$other).unwrap()
    };
}
