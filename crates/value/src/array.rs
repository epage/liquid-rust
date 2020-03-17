//! Type representing a Liquid array, payload of the `BoxedValue::Array` variant

use std::fmt;

use kstring::KStringCow;

use crate::values::{DisplayCow, State};
use crate::{Value, ValueView};

/// A value::Array literal.
///
/// # Example
///
/// ```rust
/// # use liquid_value::ValueView;
/// #
/// # fn main() {
/// liquid_value::value!([1, "2", 3])
///     .as_array().unwrap();
/// # }
/// ```
#[macro_export(local_inner_macros)]
macro_rules! array {
    ($($value:tt)+) => {
        array_internal!($($value)+)
    };
}

#[macro_export(local_inner_macros)]
#[doc(hidden)]
macro_rules! array_internal {
    // Done with trailing comma.
    (@array [$($elems:expr,)*]) => {
        array_internal_vec![$($elems,)*]
    };

    // Done without trailing comma.
    (@array [$($elems:expr),*]) => {
        array_internal_vec![$($elems),*]
    };

    // Next element is `nil`.
    (@array [$($elems:expr,)*] nil $($rest:tt)*) => {
        array_internal!(@array [$($elems,)* value_internal!(nil)] $($rest)*)
    };

    // Next element is `true`.
    (@array [$($elems:expr,)*] true $($rest:tt)*) => {
        array_internal!(@array [$($elems,)* value_internal!(true)] $($rest)*)
    };

    // Next element is `false`.
    (@array [$($elems:expr,)*] false $($rest:tt)*) => {
        array_internal!(@array [$($elems,)* value_internal!(false)] $($rest)*)
    };

    // Next element is an array.
    (@array [$($elems:expr,)*] [$($array:tt)*] $($rest:tt)*) => {
        array_internal!(@array [$($elems,)* value_internal!([$($array)*])] $($rest)*)
    };

    // Next element is a map.
    (@array [$($elems:expr,)*] {$($map:tt)*} $($rest:tt)*) => {
        array_internal!(@array [$($elems,)* value_internal!({$($map)*})] $($rest)*)
    };

    // Next element is an expression followed by comma.
    (@array [$($elems:expr,)*] $next:expr, $($rest:tt)*) => {
        array_internal!(@array [$($elems,)* value_internal!($next),] $($rest)*)
    };

    // Last element is an expression with no trailing comma.
    (@array [$($elems:expr,)*] $last:expr) => {
        array_internal!(@array [$($elems,)* value_internal!($last)])
    };

    // Comma after the most recent element.
    (@array [$($elems:expr),*] , $($rest:tt)*) => {
        array_internal!(@array [$($elems,)*] $($rest)*)
    };

    // Unexpected token after most recent element.
    (@array [$($elems:expr),*] $unexpected:tt $($rest:tt)*) => {
        array_unexpected!($unexpected)
    };

    //////////////////////////////////////////////////////////////////////////
    // The main implementation.
    //
    // Must be invoked as: value_internal!($($value)+)
    //////////////////////////////////////////////////////////////////////////

    ([]) => {
        $crate::Array::default()
    };

    ([ $($tt:tt)+ ]) => {
        array_internal!(@array [] $($tt)+)
    };

    ($other:ident) => {
        $other
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! array_internal_vec {
    ($($content:tt)*) => {
        vec![$($content)*]
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! array_unexpected {
    () => {};
}

/// Accessor for arrays.
pub trait ArrayView: ValueView {
    /// Cast to ValueView
    fn as_value(&self) -> &dyn ValueView;

    /// Returns the number of elements.
    fn size(&self) -> i32;

    /// Returns an iterator .
    fn values<'k>(&'k self) -> Box<dyn Iterator<Item = &'k dyn ValueView> + 'k>;

    /// Access a contained `BoxedValue`.
    fn contains_key(&self, index: i32) -> bool;
    /// Access a contained `Value`.
    fn get(&self, index: i32) -> Option<&dyn ValueView>;
    /// Returns the first element.
    fn first(&self) -> Option<&dyn ValueView> {
        self.get(0)
    }
    /// Returns the last element.
    fn last(&self) -> Option<&dyn ValueView> {
        self.get(-1)
    }
}

/// Type representing a Liquid array, payload of the `BoxedValue::Array` variant
pub type Array = Vec<Value>;

impl<T: ValueView> ValueView for Vec<T> {
    fn as_debug(&self) -> &dyn fmt::Debug {
        self
    }

    fn render(&self) -> DisplayCow<'_> {
        DisplayCow::Owned(Box::new(ArrayRender { s: self }))
    }
    fn source(&self) -> DisplayCow<'_> {
        DisplayCow::Owned(Box::new(ArraySource { s: self }))
    }
    fn type_name(&self) -> &'static str {
        "array"
    }
    fn query_state(&self, state: State) -> bool {
        match state {
            State::Truthy => true,
            State::DefaultValue | State::Empty | State::Blank => self.is_empty(),
        }
    }

    fn to_kstr(&self) -> KStringCow<'_> {
        let s = ArrayRender { s: self }.to_string();
        KStringCow::from_string(s)
    }
    fn to_value(&self) -> Value {
        let a = self.iter().map(|v| v.to_value()).collect();
        Value::Array(a)
    }

    fn as_array(&self) -> Option<&dyn ArrayView> {
        Some(self)
    }
}

impl<T: ValueView> ArrayView for Vec<T> {
    fn as_value(&self) -> &dyn ValueView {
        self
    }

    fn size(&self) -> i32 {
        self.len() as i32
    }

    fn values<'k>(&'k self) -> Box<dyn Iterator<Item = &'k dyn ValueView> + 'k> {
        let i = self.as_slice().iter().map(|v| convert_value(v));
        Box::new(i)
    }

    fn contains_key(&self, index: i32) -> bool {
        let index = convert_index(index, self.size());
        index < self.size()
    }

    fn get(&self, index: i32) -> Option<&dyn ValueView> {
        let index = convert_index(index, self.size());
        let value = self.as_slice().get(index as usize);
        value.map(|v| convert_value(v))
    }
}

fn convert_value(s: &dyn ValueView) -> &dyn ValueView {
    s
}

fn convert_index(index: i32, max_size: i32) -> i32 {
    if 0 <= index {
        index
    } else {
        max_size + index
    }
}

struct ArraySource<'s, T: ValueView> {
    s: &'s Vec<T>,
}

impl<'s, T: ValueView> fmt::Display for ArraySource<'s, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;
        for item in self.s {
            write!(f, "{}, ", item.render())?;
        }
        write!(f, "]")?;
        Ok(())
    }
}

struct ArrayRender<'s, T: ValueView> {
    s: &'s Vec<T>,
}

impl<'s, T: ValueView> fmt::Display for ArrayRender<'s, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for item in self.s {
            write!(f, "{}", item.render())?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_array() {
        let arr = Array::new();
        println!("{}", arr.source());
        let array: &dyn ArrayView = &arr;
        println!("{}", array.source());
        let view: &dyn ValueView = array.as_value();
        println!("{}", view.source());
    }
}
