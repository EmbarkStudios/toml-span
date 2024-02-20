use crate::{Error, ErrorKind, Span};
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

    #[inline]
    pub fn set(&mut self, value: ValueInner<'de>) {
        self.value = Some(value);
    }

    /// Returns true if the value is a table and is non-empty
    #[inline]
    pub fn has_keys(&self) -> bool {
        self.value.as_ref().map_or(false, |val| {
            if let ValueInner::Table(table) = val {
                !table.is_empty()
            } else {
                false
            }
        })
    }

    #[inline]
    pub fn has_key(&self, key: &str) -> bool {
        self.value.as_ref().map_or(false, |val| {
            if let ValueInner::Table(table) = val {
                table.contains_key(&key.into())
            } else {
                false
            }
        })
    }

    #[inline]
    pub fn take_string(&mut self, msg: Option<&'static str>) -> Result<Cow<'de, str>, Error> {
        match self.take() {
            ValueInner::String(s) => Ok(s),
            other => Err(Error {
                kind: ErrorKind::Wanted {
                    expected: msg.unwrap_or("a string"),
                    found: other.type_str(),
                },
                span: self.span,
                line_info: None,
            }),
        }
    }

    #[inline]
    pub fn as_str(&self) -> Option<&str> {
        self.value.as_ref().and_then(|v| v.as_str())
    }

    #[inline]
    pub fn as_table(&self) -> Option<&Table<'de>> {
        self.value.as_ref().and_then(|v| v.as_table())
    }

    #[inline]
    pub fn as_array(&self) -> Option<&Array<'de>> {
        self.value.as_ref().and_then(|v| v.as_array())
    }

    #[inline]
    pub fn as_integer(&self) -> Option<i64> {
        self.value.as_ref().and_then(|v| v.as_integer())
    }

    #[inline]
    pub fn as_float(&self) -> Option<f64> {
        self.value.as_ref().and_then(|v| v.as_float())
    }

    #[inline]
    pub fn as_bool(&self) -> Option<bool> {
        self.value.as_ref().and_then(|v| v.as_bool())
    }

    pub fn pointer(&self, pointer: &'de str) -> Option<&Self> {
        if pointer.is_empty() {
            return Some(self);
        } else if !pointer.starts_with('/') {
            return None;
        }

        pointer
            .split('/')
            .skip(1)
            // Don't support / or ~ in key names unless someone actually opens
            // an issue about it
            //.map(|x| x.replace("~1", "/").replace("~0", "~"))
            .try_fold(self, |target, token| {
                (match &target.value {
                    Some(ValueInner::Table(tab)) => tab.get(&token.into()),
                    Some(ValueInner::Array(list)) => parse_index(token).and_then(|x| list.get(x)),
                    _ => None,
                })
                .filter(|v| v.value.is_some())
            })
    }

    pub fn pointer_mut(&mut self, pointer: &'de str) -> Option<&mut Self> {
        if pointer.is_empty() {
            return Some(self);
        } else if !pointer.starts_with('/') {
            return None;
        }

        pointer
            .split('/')
            .skip(1)
            // Don't support / or ~ in key names unless someone actually opens
            // an issue about it
            //.map(|x| x.replace("~1", "/").replace("~0", "~"))
            .try_fold(self, |target, token| {
                (match &mut target.value {
                    Some(ValueInner::Table(tab)) => tab.get_mut(&token.into()),
                    Some(ValueInner::Array(list)) => {
                        parse_index(token).and_then(|x| list.get_mut(x))
                    }
                    _ => None,
                })
                .filter(|v| v.value.is_some())
            })
    }
}

fn parse_index(s: &str) -> Option<usize> {
    if s.starts_with('+') || (s.starts_with('0') && s.len() != 1) {
        return None;
    }
    s.parse().ok()
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

    #[inline]
    pub fn as_str(&self) -> Option<&str> {
        if let Self::String(s) = self {
            Some(s.as_ref())
        } else {
            None
        }
    }

    #[inline]
    pub fn as_table(&self) -> Option<&Table<'de>> {
        if let ValueInner::Table(t) = self {
            Some(t)
        } else {
            None
        }
    }

    #[inline]
    pub fn as_array(&self) -> Option<&Array<'de>> {
        if let ValueInner::Array(a) = self {
            Some(a)
        } else {
            None
        }
    }

    #[inline]
    pub fn as_integer(&self) -> Option<i64> {
        if let ValueInner::Integer(i) = self {
            Some(*i)
        } else {
            None
        }
    }

    #[inline]
    pub fn as_float(&self) -> Option<f64> {
        if let ValueInner::Float(f) = self {
            Some(*f)
        } else {
            None
        }
    }

    #[inline]
    pub fn as_bool(&self) -> Option<bool> {
        if let ValueInner::Boolean(b) = self {
            Some(*b)
        } else {
            None
        }
    }
}
