use crate::{
    value::{self, Table, Value, ValueInner},
    DeserError, Deserialize, Error, ErrorKind,
};

pub struct TableHelper<'de> {
    table: value::Table<'de>,
    errors: Vec<Error>,
}

impl<'de> TableHelper<'de> {
    pub fn new(value: &mut Value<'de>) -> Result<Self, DeserError> {
        let table = match value.take() {
            ValueInner::Table(table) => table,
            other => {
                return Err(Error {
                    kind: ErrorKind::Wanted {
                        expected: "a table",
                        found: other.type_str(),
                    },
                    span: value.span,
                    line_info: None,
                }
                .into());
            }
        };

        Ok(Self {
            errors: Vec::new(),
            table,
        })
    }

    pub fn required<T: Deserialize + Default>(&mut self, name: &'static str) -> T {
        let Some(mut val) = self.table.remove(&name.into()) else {
            self.errors.push(Error {
                kind: ErrorKind::MissingField(name),
                span: Default::default(),
                line_info: None,
            });
            return T::default();
        };

        match T::deserialize(&mut val) {
            Ok(val) => val,
            Err(mut err) => {
                self.errors.append(&mut err.errors);
                T::default()
            }
        }
    }

    pub fn optional<T: Deserialize>(&mut self, name: &'static str) -> Option<T> {
        let Some(mut val) = self.table.remove(&name.into()) else {
            return None;
        };

        match T::deserialize(&mut val) {
            Ok(val) => Some(val),
            Err(mut err) => {
                self.errors.append(&mut err.errors);
                None
            }
        }
    }

    pub fn finalize(mut self, fail_on_unknown_fields: bool) -> Result<(), DeserError> {
        if fail_on_unknown_fields && !self.table.is_empty() {
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

impl Deserialize for String {
    fn deserialize<'de>(value: &mut Value<'de>) -> Result<Self, DeserError> {
        match value.take() {
            ValueInner::String(s) => Ok(s.into()),
            other => Err(Error {
                kind: ErrorKind::Wanted {
                    expected: "a string",
                    found: other.type_str(),
                },
                span: value.span,
                line_info: None,
            }
            .into()),
        }
    }
}

macro_rules! integer {
    ($num:ty) => {
        impl Deserialize for $num {
            fn deserialize<'de>(value: &mut Value<'de>) -> Result<Self, DeserError> {
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
                    other => Err(Error {
                        kind: ErrorKind::Wanted {
                            expected: stringify!($num),
                            found: other.type_str(),
                        },
                        span: value.span,
                        line_info: None,
                    }
                    .into()),
                }
            }
        }
    };
}

integer!(u32);
