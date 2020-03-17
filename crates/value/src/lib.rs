//! Liquid Value type.

#![warn(missing_docs)]
#![warn(missing_debug_implementations)]
#![warn(unused_extern_crates)]

#[macro_use]
extern crate serde;

#[macro_use]
mod macros;

mod ser;

pub mod array;
pub mod find;
pub mod object;
pub mod scalar;
pub mod values;

pub use crate::object::{to_object, Object, ObjectView};
pub use crate::scalar::{Scalar, ScalarCow};
pub use crate::values::{to_value, Value, ValueCow, ValueView};
