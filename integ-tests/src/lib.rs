/// Loads a valid toml file and does a snapshot assertion against `toml`
#[macro_export]
macro_rules! valid {
    ($name:ident) => {
        #[test]
        fn $name() {
            let toml_str =
                std::fs::read_to_string(dbg!(concat!("data/", stringify!($name), ".toml")))
                    .expect(concat!("failed to load ", stringify!($name), ".toml"));
            let valid_toml = toml_file::parse(&toml_str).expect("failed to parse toml");
            insta::assert_json_snapshot!(valid_toml);
        }
    };
    ($name:ident, $toml:literal) => {
        #[test]
        fn $name() {
            let valid_toml = toml_file::parse($toml).expect("failed to parse toml");
            insta::assert_json_snapshot!(valid_toml);
        }
    };
}

#[macro_export]
macro_rules! unexpected {
    ($name:ident, $err:expr, $toml:expr) => {{
        let file = $crate::File::new(stringify!($name), $toml);
        let error = $crate::emit_error(&file, $err);

        panic!("unexpected toml deserialization errors:\n{error}");
    }};
}

/// Loads a valid toml file, deserializes it to the specified type and asserts
/// the debug snapshot matches
#[macro_export]
macro_rules! valid_de {
    ($name:ident, $kind:ty) => {
        #[test]
        fn $name() {
            let toml_str =
                std::fs::read_to_string(dbg!(concat!("data/", stringify!($name), ".toml")))
                    .expect(concat!("failed to load ", stringify!($name), ".toml"));
            let mut valid_toml = toml_file::parse(&toml_str).expect("failed to parse toml");

            match $kind::deserialize(&mut valid_toml) {
                Ok(de) => {
                    insta::assert_debug_snapshot!(de);
                }
                Err(err) => {
                    $crate::unexpected!($name, err.errors, toml_str);
                }
            }
        }
    };
    ($name:ident, $kind:ty, $toml:literal) => {
        #[test]
        fn $name() {
            let mut valid_toml = toml_file::parse($toml).expect("failed to parse toml");

            match <$kind>::deserialize(&mut valid_toml) {
                Ok(de) => {
                    insta::assert_debug_snapshot!(de);
                }
                Err(err) => {
                    $crate::unexpected!($name, err.errors, $toml);
                }
            }
        }
    };
}

pub type File<'s> = codespan_reporting::files::SimpleFile<&'static str, &'s str>;

pub fn emit_error(f: &File, error: impl IntoIterator<Item = toml_file::Error>) -> String {
    let mut output = codespan_reporting::term::termcolor::NoColor::new(Vec::new());

    for err in error {
        let diag = err.to_diagnostic(());
        codespan_reporting::term::emit(
            &mut output,
            &codespan_reporting::term::Config::default(),
            f,
            &diag,
        )
        .expect("uhm...oops?");
    }

    String::from_utf8(output.into_inner()).unwrap()
}

/// Creates a codespan diagnostic for an error and asserts the emitted diagnostic
/// matches a snapshot
#[macro_export]
macro_rules! error_snapshot {
    ($name:ident, $err:expr, $toml:expr) => {
        let file = $crate::File::new(stringify!($name), $toml);
        let error = $crate::emit_error(&file, $err);
        insta::assert_snapshot!(error);
    };
}

/// Loads an invalid toml file and does a snapshot assertion on the error
#[macro_export]
macro_rules! invalid {
    ($name:ident) => {
        #[test]
        fn $name() {
            let toml_str =
                std::fs::read_to_string(dbg!(concat!("data/", stringify!($name), ".toml")))
                    .expect(concat!("failed to load ", stringify!($name), ".toml"));
            let error = toml_file::parse(toml_str).unwrap_err();
            $crate::error_snapshot!($name, error, &toml_str);
        }
    };
    ($name:ident, $toml:expr) => {
        #[test]
        fn $name() {
            let error = toml_file::parse($toml).unwrap_err();
            $crate::error_snapshot!($name, Some(error), $toml);
        }
    };
}
