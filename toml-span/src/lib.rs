#![allow(missing_docs)]

pub mod de;
pub mod de_helpers;
mod error;
pub mod span;
pub mod tokens;
pub mod value;

pub use de::parse;
pub use error::{DeserError, Error, ErrorKind};
pub use span::Span;
pub use value::Value;

pub trait Deserialize<'de>: Sized {
    fn deserialize(value: &mut Value<'de>) -> Result<Self, DeserError>;
}

pub trait DeserializeOwned: for<'de> Deserialize<'de> {}
impl<T> DeserializeOwned for T where T: for<'de> Deserialize<'de> {}
