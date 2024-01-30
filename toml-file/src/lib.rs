pub mod de;
pub mod de_helpers;
mod error;
pub mod span;
pub mod tokens;
pub mod value;

pub use de::parse;
pub use error::{Error, ErrorKind};
pub use span::Span;

pub struct DeserError {
    pub errors: Vec<Error>,
}

impl From<Error> for DeserError {
    fn from(value: Error) -> Self {
        Self {
            errors: vec![value],
        }
    }
}

pub trait Deserialize: Sized {
    fn deserialize<'de>(value: &mut value::Value<'de>) -> Result<Self, DeserError>;
}
