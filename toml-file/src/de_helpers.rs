use crate::{
    span::Spanned,
    value::{self, Table, Value, ValueInner},
    DeserError, Deserialize, Error, ErrorKind, Span,
};
use std::{fmt::Display, str::FromStr};

#[inline]
pub fn expected(expected: &'static str, found: ValueInner<'_>, span: Span) -> Error {
    Error {
        kind: ErrorKind::Wanted {
            expected,
            found: found.type_str(),
        },
        span,
        line_info: None,
    }
}

#[inline]
pub fn parse<T, E>(value: &mut Value<'_>) -> Result<T, Error>
where
    T: FromStr<Err = E>,
    E: Display,
{
    let s = value.take_string(None)?;
    match s.parse() {
        Ok(v) => Ok(v),
        Err(err) => Err(Error {
            kind: ErrorKind::Custom(format!("failed to parse string: {err}")),
            span: value.span,
            line_info: None,
        }),
    }
}

pub struct TableHelper<'de> {
    pub table: Table<'de>,
    pub errors: Vec<Error>,
}

impl<'de> From<Table<'de>> for TableHelper<'de> {
    fn from(table: Table<'de>) -> Self {
        Self {
            table,
            errors: Vec::new(),
        }
    }
}

impl<'de> TableHelper<'de> {
    pub fn new(value: &mut Value<'de>) -> Result<Self, DeserError> {
        let table = match value.take() {
            ValueInner::Table(table) => table,
            other => return Err(expected("a table", other, value.span).into()),
        };

        Ok(Self {
            errors: Vec::new(),
            table,
        })
    }

    #[inline]
    pub fn contains(&self, name: &'static str) -> bool {
        self.table.contains_key(&name.into())
    }

    #[inline]
    pub fn take(&mut self, name: &'static str) -> Option<(value::Key<'de>, Value<'de>)> {
        self.table.remove_entry(&name.into())
    }

    #[inline]
    pub fn required<T: Deserialize<'de>>(&mut self, name: &'static str) -> Result<T, Error> {
        Ok(self.required_s(name)?.value)
    }

    pub fn required_s<T: Deserialize<'de>>(
        &mut self,
        name: &'static str,
    ) -> Result<Spanned<T>, Error> {
        let Some(mut val) = self.table.remove(&name.into()) else {
            let missing = Error {
                kind: ErrorKind::MissingField(name),
                span: Default::default(),
                line_info: None,
            };
            self.errors.push(missing.clone());
            return Err(missing);
        };

        Spanned::<T>::deserialize(&mut val).map_err(|mut errs| {
            let err = errs.errors.last().unwrap().clone();
            self.errors.append(&mut errs.errors);
            err
        })
    }

    pub fn with_default<T: Deserialize<'de>>(
        &mut self,
        name: &'static str,
        def: impl FnOnce() -> T,
    ) -> (T, Span) {
        let Some(mut val) = self.table.remove(&name.into()) else {
            return (def(), Span::default());
        };

        match T::deserialize(&mut val) {
            Ok(v) => (v, val.span),
            Err(mut err) => {
                self.errors.append(&mut err.errors);
                (def(), Span::default())
            }
        }
    }

    pub fn optional<T: Deserialize<'de>>(&mut self, name: &'static str) -> Option<T> {
        self.optional_s(name).map(|v| v.value)
    }

    pub fn optional_s<T: Deserialize<'de>>(&mut self, name: &'static str) -> Option<Spanned<T>> {
        let Some(mut val) = self.table.remove(&name.into()) else {
            return None;
        };

        match Spanned::<T>::deserialize(&mut val) {
            Ok(v) => Some(v),
            Err(mut err) => {
                self.errors.append(&mut err.errors);
                None
            }
        }
    }

    pub fn parse<T, E>(&mut self, name: &'static str) -> T
    where
        T: FromStr<Err = E> + Default,
        E: Display,
    {
        let Some(mut val) = self.table.remove(&name.into()) else {
            self.errors.push(Error {
                kind: ErrorKind::MissingField(name),
                span: Default::default(),
                line_info: None,
            });
            return T::default();
        };

        match parse(&mut val) {
            Ok(v) => v,
            Err(err) => {
                self.errors.push(err);
                T::default()
            }
        }
    }

    pub fn parse_opt<T, E>(&mut self, name: &'static str) -> Option<T>
    where
        T: FromStr<Err = E>,
        E: Display,
    {
        let Some(mut val) = self.table.remove(&name.into()) else {
            return None;
        };

        match parse(&mut val) {
            Ok(v) => Some(v),
            Err(err) => {
                self.errors.push(err);
                None
            }
        }
    }

    pub fn finalize(mut self, original: Option<&mut Value<'de>>) -> Result<(), DeserError> {
        if let Some(original) = original {
            original.set(ValueInner::Table(self.table));
        } else if !self.table.is_empty() {
            let keys = self
                .table
                .into_keys()
                .map(|key| (key.name.into(), key.span))
                .collect();

            self.errors.push(Error {
                span: Default::default(),
                kind: ErrorKind::UnexpectedKeys { keys },
                line_info: None,
            })
        }

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(DeserError {
                errors: self.errors,
            })
        }
    }
}

impl<'de> Deserialize<'de> for String {
    fn deserialize(value: &mut Value<'de>) -> Result<Self, DeserError> {
        value
            .take_string(None)
            .map(|s| s.into())
            .map_err(DeserError::from)
    }
}

impl<'de> Deserialize<'de> for std::borrow::Cow<'de, str> {
    fn deserialize(value: &mut Value<'de>) -> Result<Self, DeserError> {
        value.take_string(None).map_err(DeserError::from)
    }
}

impl<'de> Deserialize<'de> for bool {
    fn deserialize(value: &mut Value<'de>) -> Result<Self, DeserError> {
        match value.take() {
            ValueInner::Boolean(b) => Ok(b),
            other => Err(expected("a bool", other, value.span).into()),
        }
    }
}

macro_rules! integer {
    ($num:ty) => {
        impl<'de> Deserialize<'de> for $num {
            fn deserialize(value: &mut Value<'de>) -> Result<Self, DeserError> {
                match value.take() {
                    ValueInner::Integer(i) => {
                        let i = i.try_into().map_err(|_| {
                            DeserError::from(Error {
                                kind: ErrorKind::InvalidNumber,
                                span: value.span,
                                line_info: None,
                            })
                        })?;

                        Ok(i)
                    }
                    other => Err(expected(stringify!($num), other, value.span).into()),
                }
            }
        }
    };
}

integer!(u8);
integer!(u16);
integer!(u32);
integer!(u64);
integer!(i8);
integer!(i16);
integer!(i32);
integer!(i64);
integer!(usize);
integer!(isize);

impl<'de> Deserialize<'de> for f32 {
    fn deserialize(value: &mut Value<'de>) -> Result<Self, DeserError> {
        match value.take() {
            ValueInner::Float(f) => Ok(f as f32),
            other => Err(expected("a float", other, value.span).into()),
        }
    }
}

impl<'de> Deserialize<'de> for f64 {
    fn deserialize(value: &mut Value<'de>) -> Result<Self, DeserError> {
        match value.take() {
            ValueInner::Float(f) => Ok(f),
            other => Err(expected("a float", other, value.span).into()),
        }
    }
}

impl<'de, T> Deserialize<'de> for Vec<T>
where
    T: Deserialize<'de>,
{
    fn deserialize(value: &mut value::Value<'de>) -> Result<Self, DeserError> {
        match value.take() {
            ValueInner::Array(arr) => {
                let mut errors = Vec::new();
                let mut s = Vec::new();
                for mut v in arr {
                    match T::deserialize(&mut v) {
                        Ok(v) => s.push(v),
                        Err(mut err) => errors.append(&mut err.errors),
                    }
                }

                if errors.is_empty() {
                    Ok(s)
                } else {
                    Err(DeserError { errors })
                }
            }
            other => Err(expected("an array", other, value.span).into()),
        }
    }
}
