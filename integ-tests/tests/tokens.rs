use pretty_assertions::assert_eq;
use std::borrow::Cow;
use toml_file::tokens::{Error, Token, Tokenizer};

fn err(input: &str, err: Error) {
    let mut t = Tokenizer::new(input);
    let token = t.next().unwrap_err();
    assert_eq!(token, err);
    assert!(t.next().unwrap().is_none());
}

#[test]
fn strings() {
    fn t(input: &str, eval: &str, emultiline: bool) {
        let mut t = Tokenizer::new(input);
        let (_, token) = t.next().unwrap().unwrap();

        if let Token::String {
            src,
            val,
            multiline,
            ..
        } = token
        {
            assert_eq!(src, input);
            assert_eq!(val, Cow::Borrowed(eval));
            assert_eq!(multiline, emultiline);
        } else {
            panic!("not a string");
        }
        assert!(t.next().unwrap().is_none());
    }

    // Literal strings
    t("''", "", false);
    t("''''''", "", true);
    t("'''\n'''", "", true);
    t("'a'", "a", false);
    t("'\"a'", "\"a", false);
    t("''''a'''", "'a", true);
    t("'''\n'a\n'''", "'a\n", true);
    t("'''a\n'a\r\n'''", "a\n'a\n", true);

    // Basic strings
    t(r#""""#, "", false);
    t(r#""""""""#, "", true);
    t(r#""a""#, "a", false);
    t(r#""""a""""#, "a", true);
    t(r#""\t""#, "\t", false);
    t(r#""\u0000""#, "\0", false);
    t(r#""\U00000000""#, "\0", false);
    t(r#""\U000A0000""#, "\u{A0000}", false);
    t(r#""\\t""#, "\\t", false);
    t("\"\t\"", "\t", false);
    t("\"\"\"\n\t\"\"\"", "\t", true);
    t("\"\"\"\\\n\"\"\"", "", true);
    t(
        "\"\"\"\\\n     \t   \t  \\\r\n  \t \n  \t \r\n\"\"\"",
        "",
        true,
    );
    t(r#""\r""#, "\r", false);
    t(r#""\n""#, "\n", false);
    t(r#""\b""#, "\u{8}", false);
    t(r#""a\fa""#, "a\u{c}a", false);
    t(r#""\"a""#, "\"a", false);
    t("\"\"\"\na\"\"\"", "a", true);
    t("\"\"\"\n\"\"\"", "", true);
    t(r#""""a\"""b""""#, "a\"\"\"b", true);

    // Invalid strings
    err(r#""\a"#, Error::InvalidEscape(2, 'a'));
    err("\"\\\n", Error::InvalidEscape(2, '\n'));
    err("\"\\\r\n", Error::InvalidEscape(2, '\n'));
    err("\"\\", Error::UnterminatedString(0));
    err("\"\u{0}", Error::InvalidCharInString(1, '\u{0}'));
    err(r#""\U00""#, Error::InvalidHexEscape(5, '"'));
    err(r#""\U00"#, Error::UnterminatedString(0));
    err(r#""\uD800"#, Error::InvalidEscapeValue(2, 4, 0xd800));
    err(
        r#""\UFFFFFFFF"#,
        Error::InvalidEscapeValue(2, 8, 0xffff_ffff),
    );
}

#[test]
fn keylike() {
    fn t(input: &str) {
        let mut t = Tokenizer::new(input);
        let (_, token) = t.next().unwrap().unwrap();
        assert_eq!(token, Token::Keylike(input));
        assert!(t.next().unwrap().is_none());
    }
    t("foo");
    t("0bar");
    t("bar0");
    t("1234");
    t("a-b");
    t("a_B");
    t("-_-");
    t("___");
}

#[test]
fn all() {
    fn t(input: &str, expected: &[((usize, usize), Token, &str)]) {
        let mut tokens = Tokenizer::new(input);
        let mut actual: Vec<((usize, usize), Token, &str)> = Vec::new();
        while let Some((span, token)) = tokens.next().unwrap() {
            actual.push((span.into(), token, &input[span.start..span.end]));
        }
        for (a, b) in actual.iter().zip(expected) {
            assert_eq!(a, b);
        }
        assert_eq!(actual.len(), expected.len());
    }

    t(
        " a ",
        &[
            ((0, 1), Token::Whitespace(" "), " "),
            ((1, 2), Token::Keylike("a"), "a"),
            ((2, 3), Token::Whitespace(" "), " "),
        ],
    );

    t(
        " a\t [[]] \t [] {} , . =\n# foo \r\n#foo \n ",
        &[
            ((0, 1), Token::Whitespace(" "), " "),
            ((1, 2), Token::Keylike("a"), "a"),
            ((2, 4), Token::Whitespace("\t "), "\t "),
            ((4, 5), Token::LeftBracket, "["),
            ((5, 6), Token::LeftBracket, "["),
            ((6, 7), Token::RightBracket, "]"),
            ((7, 8), Token::RightBracket, "]"),
            ((8, 11), Token::Whitespace(" \t "), " \t "),
            ((11, 12), Token::LeftBracket, "["),
            ((12, 13), Token::RightBracket, "]"),
            ((13, 14), Token::Whitespace(" "), " "),
            ((14, 15), Token::LeftBrace, "{"),
            ((15, 16), Token::RightBrace, "}"),
            ((16, 17), Token::Whitespace(" "), " "),
            ((17, 18), Token::Comma, ","),
            ((18, 19), Token::Whitespace(" "), " "),
            ((19, 20), Token::Period, "."),
            ((20, 21), Token::Whitespace(" "), " "),
            ((21, 22), Token::Equals, "="),
            ((22, 23), Token::Newline, "\n"),
            ((23, 29), Token::Comment("# foo "), "# foo "),
            ((29, 31), Token::Newline, "\r\n"),
            ((31, 36), Token::Comment("#foo "), "#foo "),
            ((36, 37), Token::Newline, "\n"),
            ((37, 38), Token::Whitespace(" "), " "),
        ],
    );
}

#[test]
fn bare_cr_bad() {
    err("\r", Error::Unexpected(0, '\r'));
    err("'\n", Error::NewlineInString(1));
    err("'\u{0}", Error::InvalidCharInString(1, '\u{0}'));
    err("'", Error::UnterminatedString(0));
    err("\u{0}", Error::Unexpected(0, '\u{0}'));
}

#[test]
fn bad_comment() {
    let mut t = Tokenizer::new("#\u{0}");
    t.next().unwrap().unwrap();
    assert_eq!(t.next(), Err(Error::Unexpected(1, '\u{0}')));
    assert!(t.next().unwrap().is_none());
}
