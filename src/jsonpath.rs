//!  Use JsonPath to select fields in json response
//!
use crate::error::{Result, ScraperError};
use jsonpath_lib as jsonpath;
use serde::de::DeserializeOwned;

/// Json Response
pub struct Json {
    pub(crate) value: serde_json::Value,
}

impl Json {
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
}
