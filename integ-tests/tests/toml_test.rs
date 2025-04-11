use std::str;

use toml_test_harness as toml_test;

fn main() {
    let mut harness = toml_test::DecoderHarness::new(Decoder);
    harness.version("1.0.0");
    // ignore all test cases that contain datetimes
    harness
        .ignore([
            "valid/datetime/",
            "valid/example.toml",
            "valid/spec-example-1.toml",
            "valid/spec-example-1-compact.toml",
            "valid/array/array.toml",
            "valid/comment/everywhere.toml",
            "valid/spec/table-7.toml",
            "valid/spec/local-date-0.toml",
            "valid/spec/local-date-time-0.toml",
            "valid/spec/local-time-0.toml",
            "valid/spec/offset-date-time-0.toml",
            "valid/spec/offset-date-time-1.toml",
        ])
        .unwrap();
    harness.test();
}

#[derive(Copy, Clone)]
pub struct Decoder;
impl toml_test_harness::Decoder for Decoder {
    fn name(&self) -> &str {
        "toml-span"
    }
    fn decode(&self, data: &[u8]) -> Result<toml_test::Decoded, toml_test::Error> {
        let str = str::from_utf8(data).map_err(toml_test::Error::new)?;
        let root_value = toml_span::parse(str).map_err(toml_test::Error::new)?;
        Ok(transcode_value(root_value))
    }
}
fn transcode_value(mut value: toml_span::Value<'_>) -> toml_test::Decoded {
    use toml_span::value::ValueInner;
    match value.take() {
        ValueInner::String(val) => toml_test::Decoded::Value(val.to_string().into()),
        ValueInner::Integer(val) => toml_test::Decoded::Value(val.into()),
        ValueInner::Float(val) => toml_test::Decoded::Value(val.into()),
        ValueInner::Boolean(val) => toml_test::Decoded::Value(val.into()),
        ValueInner::Array(ary) => transcode_array(ary),
        ValueInner::Table(tbl) => transcode_table(tbl),
    }
}
fn transcode_array(array: toml_span::value::Array<'_>) -> toml_test::Decoded {
    toml_test::Decoded::Array(array.into_iter().map(transcode_value).collect())
}
fn transcode_table(table: toml_span::value::Table<'_>) -> toml_test::Decoded {
    toml_test::Decoded::Table(
        table
            .into_iter()
            .map(|(key, val)| (key.name.to_string(), transcode_value(val)))
            .collect(),
    )
}
