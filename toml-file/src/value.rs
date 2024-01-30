use crate::Span;
use std::{borrow::Cow, fmt};

#[cfg(feature = "serde")]
mod impl_serde;

pub struct Value<'de> {
    value: Option<ValueInner<'de>>,
    pub span: Span,
}

impl<'de> Value<'de> {
    #[inline]
    pub fn new(value: ValueInner<'de>) -> Self {
        Self::with_span(value, Span::default())
    }

    #[inline]
    pub fn with_span(value: ValueInner<'de>, span: Span) -> Self {
        Self {
            value: Some(value),
            span,
        }
    }

    #[inline]
    pub fn take(&mut self) -> ValueInner<'de> {
        self.value.take().expect("the value has already been taken")
    }
}

impl<'de> AsRef<ValueInner<'de>> for Value<'de> {
    fn as_ref(&self) -> &ValueInner<'de> {
        self.value
            .as_ref()
            .expect("the value has already been taken")
    }
}

impl<'de> fmt::Debug for Value<'de> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.value)
    }
}

#[derive(Clone)]
pub struct Key<'de> {
    pub name: Cow<'de, str>,
    pub span: Span,
}

impl<'de> From<&'de str> for Key<'de> {
    fn from(k: &'de str) -> Self {
        Self {
            name: Cow::Borrowed(k),
            span: Span::default(),
        }
    }
}

impl<'de> fmt::Debug for Key<'de> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl<'de> Ord for Key<'de> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}

impl<'de> PartialOrd for Key<'de> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<'de> PartialEq for Key<'de> {
    fn eq(&self, other: &Self) -> bool {
        self.name.eq(&other.name)
    }
}

impl<'de> Eq for Key<'de> {}

pub type Table<'de> = std::collections::BTreeMap<Key<'de>, Value<'de>>;
pub type Array<'de> = Vec<Value<'de>>;

#[derive(Debug)]
pub enum ValueInner<'de> {
    String(Cow<'de, str>),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Array(Array<'de>),
    Table(Table<'de>),
}

impl<'de> ValueInner<'de> {
    pub fn type_str(&self) -> &'static str {
        match self {
            Self::String(..) => "string",
            Self::Integer(..) => "integer",
            Self::Float(..) => "float",
            Self::Boolean(..) => "boolean",
            Self::Array(..) => "array",
            Self::Table(..) => "table",
        }
    }
}
