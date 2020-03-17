/// A value::Value literal.
///
/// # Example
///
/// ```rust
/// # use liquid_value::ValueView;
/// #
/// # fn main() {
/// liquid_value::value!(5)
///     .as_scalar().unwrap()
///     .to_integer().unwrap();
/// liquid_value::value!("foo")
///     .as_scalar().unwrap()
///     .to_kstr();
/// liquid_value::value!([1, "2", 3])
///     .as_array().unwrap();
/// liquid_value::value!({"foo": 5})
///     .as_object().unwrap();
/// # }
/// ```
#[macro_export(local_inner_macros)]
macro_rules! value {
    ($($value:tt)+) => {
        value_internal!($($value)+)
    };
}

#[macro_export(local_inner_macros)]
#[doc(hidden)]
macro_rules! value_internal {
    //////////////////////////////////////////////////////////////////////////
    // The main implementation.
    //
    // Must be invoked as: value_internal!($($value)+)
    //////////////////////////////////////////////////////////////////////////

    (nil) => {
        $crate::Value::Nil
    };

    (true) => {
        $crate::Value::scalar(true)
    };

    (false) => {
        $crate::Value::scalar(false)
    };

    ([]) => {
        $crate::Value::Array(::std::default::Default::default())
    };

    ([ $($tt:tt)+ ]) => {
        $crate::Value::Array(array_internal!(@array [] $($tt)+))
    };

    ({}) => {
        $crate::Value::Object(::std::default::Default::default())
    };

    ({ $($tt:tt)+ }) => {
        $crate::Value::Object({
            let mut object = $crate::Object::new();
            object_internal!(@object object () ($($tt)+) ($($tt)+));
            object
        })
    };

    ($other:ident) => {
        $other
    };

    // Any Serialize type: numbers, strings, struct literals, variables etc.
    // Must be below every other rule.
    ($other:expr) => {
        $crate::to_value(&$other).unwrap()
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! value_unexpected {
    () => {};
}
