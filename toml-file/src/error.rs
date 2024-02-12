use crate::Span;
use std::fmt::{self, Debug, Display};

/// Error that can occur when deserializing TOML.
#[derive(Debug, Clone)]
pub struct Error {
    pub kind: ErrorKind,
    pub span: Span,
    pub line_info: Option<(usize, usize)>,
}

impl std::error::Error for Error {}

impl From<(ErrorKind, Span)> for Error {
    fn from((kind, span): (ErrorKind, Span)) -> Self {
        Self {
            kind,
            span,
            line_info: None,
        }
    }
}

/// Errors that can occur when deserializing a type.
#[derive(Debug, Clone)]
pub enum ErrorKind {
    /// EOF was reached when looking for a value.
    UnexpectedEof,

    /// An invalid character not allowed in a string was found.
    InvalidCharInString(char),

    /// An invalid character was found as an escape.
    InvalidEscape(char),

    /// An invalid character was found in a hex escape.
    InvalidHexEscape(char),

    /// An invalid escape value was specified in a hex escape in a string.
    ///
    /// Valid values are in the plane of unicode codepoints.
    InvalidEscapeValue(u32),

    /// An unexpected character was encountered, typically when looking for a
    /// value.
    Unexpected(char),

    /// An unterminated string was found where EOF was found before the ending
    /// EOF mark.
    UnterminatedString,

    /// A newline was found in a table key.
    NewlineInTableKey,

    /// A number failed to parse.
    InvalidNumber,

    /// Wanted one sort of token, but found another.
    Wanted {
        /// Expected token type.
        expected: &'static str,
        /// Actually found token type.
        found: &'static str,
    },

    /// A duplicate table definition was found.
    DuplicateTable { name: String, first: Span },

    /// Duplicate key in table.
    DuplicateKey {
        /// The duplicate key
        key: String,
        /// The span where the first key is located
        first: Span,
    },

    /// A previously defined table was redefined as an array.
    RedefineAsArray,

    /// Multiline strings are not allowed for key.
    MultilineStringKey,

    /// A custom error which could be generated when deserializing a particular
    /// type.
    Custom(String),

    /// A tuple with a certain number of elements was expected but something
    /// else was found.
    ExpectedTuple(usize),

    /// Expected table keys to be in increasing tuple index order, but something
    /// else was found.
    ExpectedTupleIndex {
        /// Expected index.
        expected: usize,
        /// Key that was specified.
        found: String,
    },

    /// An empty table was expected but entries were found.
    ExpectedEmptyTable,

    /// Dotted key attempted to extend something that is not a table.
    DottedKeyInvalidType,

    /// An unexpected key was encountered.
    ///
    /// Used when deserializing a struct with a limited set of fields.
    UnexpectedKeys {
        /// The unexpected keys.
        keys: Vec<(String, Span)>,
    },

    /// Unquoted string was found when quoted one was expected.
    UnquotedString,

    /// A required field is missing from a table
    MissingField(&'static str),

    /// A field in the table is deprecated and the new key should be used instead
    Deprecated {
        old: &'static str,
        new: &'static str,
    },

    /// An unexpected value was encountered
    UnexpectedValue { expected: &'static [&'static str] },
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEof => f.write_str("unexpected-eof"),
            Self::Custom(..) => f.write_str("custom"),
            Self::DottedKeyInvalidType => f.write_str("dotted-key-invalid-type"),
            Self::DuplicateKey { .. } => f.write_str("duplicate-key"),
            Self::DuplicateTable { .. } => f.write_str("duplicate-table"),
            Self::ExpectedEmptyTable => f.write_str("expected-empty-table"),
            Self::ExpectedTuple(..) => f.write_str("expected-tuple"),
            Self::ExpectedTupleIndex { .. } => f.write_str("expected-tuple-index"),
            Self::UnexpectedKeys { .. } => f.write_str("unexpected-keys"),
            Self::UnquotedString => f.write_str("unquoted-string"),
            Self::MultilineStringKey => f.write_str("multiline-string-key"),
            Self::RedefineAsArray => f.write_str("redefine-as-array"),
            Self::InvalidCharInString(..) => f.write_str("invalid-char-in-string"),
            Self::InvalidEscape(..) => f.write_str("invalid-escape"),
            Self::InvalidEscapeValue(..) => f.write_str("invalid-escape-value"),
            Self::InvalidHexEscape(..) => f.write_str("invalid-hex-escape"),
            //Self::NewlineInString => f.write_str("newline-in-string"),
            Self::Unexpected(..) => f.write_str("unexpected"),
            Self::UnterminatedString => f.write_str("unterminated-string"),
            Self::NewlineInTableKey => f.write_str("newline-in-table-key"),
            Self::InvalidNumber => f.write_str("invalid-number"),
            Self::Wanted { .. } => f.write_str("wanted"),
            Self::MissingField(..) => f.write_str("missing-field"),
            Self::Deprecated { .. } => f.write_str("deprecated"),
            Self::UnexpectedValue { .. } => f.write_str("unexpected-value"),
        }
    }
}

struct Escape(char);

impl fmt::Display for Escape {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use std::fmt::Write as _;

        if self.0.is_whitespace() {
            for esc in self.0.escape_default() {
                f.write_char(esc)?;
            }
            Ok(())
        } else {
            f.write_char(self.0)
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            ErrorKind::UnexpectedEof => f.write_str("unexpected eof encountered")?,
            ErrorKind::InvalidCharInString(c) => {
                write!(f, "invalid character in string: `{}`", Escape(*c))?
            }
            ErrorKind::InvalidEscape(c) => {
                write!(f, "invalid escape character in string: `{}`", Escape(*c))?
            }
            ErrorKind::InvalidHexEscape(c) => write!(
                f,
                "invalid hex escape character in string: `{}`",
                Escape(*c)
            )?,
            ErrorKind::InvalidEscapeValue(c) => write!(f, "invalid escape value: `{c}`")?,
            ErrorKind::Unexpected(c) => write!(f, "unexpected character found: `{}`", Escape(*c))?,
            ErrorKind::UnterminatedString => f.write_str("unterminated string")?,
            ErrorKind::NewlineInTableKey => f.write_str("found newline in table key")?,
            ErrorKind::Wanted { expected, found } => {
                write!(f, "expected {expected}, found {found}")?;
            }
            ErrorKind::InvalidNumber => f.write_str("invalid number")?,
            ErrorKind::DuplicateTable { name, .. } => {
                write!(f, "redefinition of table `{name}`")?;
            }
            ErrorKind::DuplicateKey { key, .. } => {
                write!(f, "duplicate key: `{key}`")?;
            }
            ErrorKind::RedefineAsArray => f.write_str("table redefined as array")?,
            ErrorKind::MultilineStringKey => {
                f.write_str("multiline strings are not allowed for key")?
            }
            ErrorKind::Custom(message) => f.write_str(message)?,
            ErrorKind::ExpectedTuple(l) => write!(f, "expected table with length {l}")?,
            ErrorKind::ExpectedTupleIndex { expected, found } => {
                write!(f, "expected table key `{expected}`, but was `{found}`")?
            }
            ErrorKind::ExpectedEmptyTable => f.write_str("expected empty table")?,
            ErrorKind::DottedKeyInvalidType => {
                f.write_str("dotted key attempted to extend non-table type")?;
            }
            ErrorKind::UnexpectedKeys { keys } => {
                write!(f, "unexpected keys in table: `{keys:?}`")?
            }
            ErrorKind::UnquotedString => {
                f.write_str("invalid TOML value, did you mean to use a quoted string?")?
            }
            ErrorKind::MissingField(field) => write!(f, "missing field '{field}' in table")?,
            ErrorKind::Deprecated { old, new } => {
                write!(f, "field '{old}' is deprecated, '{new}' has replaced it")?
            }
            ErrorKind::UnexpectedValue { expected } => write!(f, "expected '{expected:?}'")?,
        }

        // if !self.key.is_empty() {
        //     write!(f, " for key `")?;
        //     for (i, k) in self.key.iter().enumerate() {
        //         if i > 0 {
        //             write!(f, ".")?;
        //         }
        //         write!(f, "{}", k)?;
        //     }
        //     write!(f, "`")?;
        // }

        // if let Some(line) = self.line {
        //     write!(f, " at line {} column {}", line + 1, self.col + 1)?;
        // }

        Ok(())
    }
}

#[cfg(feature = "reporting")]
impl Error {
    pub fn to_diagnostic<FileId: Copy + PartialEq>(
        &self,
        fid: FileId,
    ) -> codespan_reporting::diagnostic::Diagnostic<FileId> {
        let diag =
            codespan_reporting::diagnostic::Diagnostic::error().with_code(self.kind.to_string());

        use codespan_reporting::diagnostic::Label;

        let diag = match &self.kind {
            ErrorKind::DuplicateKey { first, .. } => diag.with_labels(vec![
                Label::secondary(fid, *first).with_message("first key instance"),
                Label::primary(fid, self.span).with_message("duplicate key"),
            ]),
            ErrorKind::Unexpected(c) => diag.with_labels(vec![Label::primary(fid, self.span)
                .with_message(format!("unexpected character '{}'", Escape(*c)))]),
            ErrorKind::InvalidCharInString(c) => {
                diag.with_labels(vec![Label::primary(fid, self.span)
                    .with_message(format!("invalid character '{}' in string", Escape(*c)))])
            }
            ErrorKind::InvalidEscape(c) => diag.with_labels(vec![Label::primary(fid, self.span)
                .with_message(format!(
                    "invalid escape character '{}' in string",
                    Escape(*c)
                ))]),
            ErrorKind::InvalidEscapeValue(_) => diag
                .with_labels(vec![
                    Label::primary(fid, self.span).with_message("invalid escape value")
                ]),
            ErrorKind::InvalidNumber => diag.with_labels(vec![
                Label::primary(fid, self.span).with_message("unable to parse number")
            ]),
            ErrorKind::Wanted { expected, .. } => diag
                .with_labels(vec![
                    Label::primary(fid, self.span).with_message(format!("expected {expected}"))
                ]),
            ErrorKind::MultilineStringKey => diag.with_labels(vec![
                Label::primary(fid, self.span).with_message("multiline keys are not allowed")
            ]),
            ErrorKind::UnterminatedString => diag
                .with_labels(vec![Label::primary(fid, self.span)
                    .with_message("eof reached before string terminator")]),
            ErrorKind::DuplicateTable { first, .. } => diag.with_labels(vec![
                Label::secondary(fid, *first).with_message("first table instance"),
                Label::primary(fid, self.span).with_message("duplicate table"),
            ]),
            ErrorKind::InvalidHexEscape(c) => diag
                .with_labels(vec![Label::primary(fid, self.span)
                    .with_message(format!("invalid hex escape '{}'", Escape(*c)))]),
            ErrorKind::UnquotedString => diag.with_labels(vec![
                Label::primary(fid, self.span).with_message("string is not quoted")
            ]),
            ErrorKind::UnexpectedKeys { keys } => diag
                .with_message(format!("found {} unexpected keys", keys.len()))
                .with_labels(
                    keys.iter()
                        .map(|(_name, span)| Label::secondary(fid, *span))
                        .collect(),
                ),
            ErrorKind::MissingField(field) => diag.with_message(format!("missing field '{field}'")),
            ErrorKind::Deprecated { new, .. } => diag
                .with_message(format!(
                    "deprecated field enountered, '{new}' should be used instead"
                ))
                .with_labels(vec![
                    Label::primary(fid, self.span).with_message("deprecated field")
                ]),
            ErrorKind::UnexpectedValue { expected } => diag
                .with_message(format!("expected '{expected:?}'"))
                .with_labels(vec![
                    Label::primary(fid, self.span).with_message("unexpected value")
                ]),
            kind => unimplemented!("{kind}"),
        };

        diag
    }
}

#[derive(Debug)]
pub struct DeserError {
    pub errors: Vec<Error>,
}

impl std::error::Error for DeserError {}

impl From<Error> for DeserError {
    fn from(value: Error) -> Self {
        Self {
            errors: vec![value],
        }
    }
}

impl fmt::Display for DeserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for err in &self.errors {
            writeln!(f, "{err}")?;
        }

        Ok(())
    }
}
