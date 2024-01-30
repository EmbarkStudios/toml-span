use integ_tests::valid_de;
use toml_file::{de_helpers::*, value::Value, DeserError, Deserialize, Error};

#[derive(Debug)]
struct Boop {
    s: String,
    os: Option<u32>,
}

impl Deserialize for Boop {
    fn deserialize<'de>(value: &mut Value<'de>) -> Result<Self, DeserError> {
        let mut mh = TableHelper::new(value)?;

        let s = mh.required("s");
        let os = mh.optional("os");

        mh.finalize(true)?;

        Ok(Self { s, os })
    }
}

valid_de!(basic_table, Boop, "s = 'boop string'\nos = 20");
