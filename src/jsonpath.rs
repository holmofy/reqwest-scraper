//!  Use JsonPath to select fields in json response
//!
use crate::error::{Result, ScraperError};
use jsonpath_lib as jsonpath;
use serde::de::DeserializeOwned;

/// Json Response
#[derive(Debug)]
pub struct Json {
    value: serde_json::Value,
}

impl Json {
    /// constructor
    pub fn new(json: &str) -> Result<Self> {
        let value = serde_json::from_str(json)?;
        Ok(Self { value })
    }

    /// Use jsonpath to select json fragments and convert them into structures
    pub fn select<T: DeserializeOwned>(&self, path: &str) -> Result<Vec<T>> {
        jsonpath::Selector::new()
            .str_path(path)?
            .value(&self.value)
            .select_as()
            .map_err(ScraperError::from)
    }

    /// Use jsonpath to select json string fields
    pub fn select_one<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let result = jsonpath::Selector::new()
            .str_path(path)?
            .value(&self.value)
            .select()?;
        let v = result
            .first()
            .ok_or_else(|| {
                ScraperError::JsonPathMatchError(format!(
                    "The \"{}\" jsonpath did not find data in json",
                    path
                ))
            })?
            .to_owned();
        Ok(serde_json::from_value::<T>(v.to_owned())?)
    }

    /// Use jsonpath to select json fields as string
    pub fn select_as_str(&self, path: &str) -> Result<String> {
        jsonpath::Selector::new()
            .str_path(path)?
            .value(&self.value)
            .select_as_str()
            .map_err(ScraperError::from)
    }

    /// Use jsonpath to select one json fields as string
    pub fn select_one_as_str(&self, path: &str) -> Result<String> {
        let result = jsonpath::Selector::new()
            .str_path(path)?
            .value(&self.value)
            .select()?;
        let v = result
            .first()
            .ok_or_else(|| {
                ScraperError::JsonPathMatchError(format!(
                    "The \"{}\" jsonpath did not find data in json",
                    path
                ))
            })?
            .to_owned();
        Ok(v.to_string())
    }
}

mod tests {

    #[test]
    #[allow(clippy::get_first)]
    fn test_jsonpath() {
        use super::Json;
        use serde::Deserialize;
        let json = r#"
        {
            "time":"2020.10.12 21:22:34",
            "data":[
                {"a":"A1","B":"b1","c":1},
                {"a":"A2","B":"b2","c":2}
            ]
        }
        "#;

        let json = Json::new(json).unwrap();

        assert_eq!(
            json.select_one_as_str("$.time").unwrap(),
            r#""2020.10.12 21:22:34""#
        );

        #[derive(Deserialize)]
        struct DataItem {
            a: String,
            #[serde(rename = "B")]
            b: String,
            c: i32,
        }

        let data: Vec<DataItem> = json.select("$.data[*]").unwrap();
        assert_eq!(data.len(), 2);

        let d1 = data.get(0).unwrap();
        assert_eq!(d1.a, "A1");
        assert_eq!(d1.b, "b1");
        assert_eq!(d1.c, 1);

        let d2 = data.get(1).unwrap();
        assert_eq!(d2.a, "A2");
        assert_eq!(d2.b, "b2");
        assert_eq!(d2.c, 2);
    }
}
