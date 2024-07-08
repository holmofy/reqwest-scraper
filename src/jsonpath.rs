use crate::error::Result;
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
            .map_err(|e| e.into())
    }

    /// Use jsonpath to select json string fields
    pub fn select_as_str(&self, path: &str) -> Result<String> {
        jsonpath::Selector::new()
            .str_path(path)?
            .value(&self.value)
            .select_as_str()
            .map_err(|e| e.into())
    }
}
