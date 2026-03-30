use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphqlRequest<'a, V>
where
    V: Serialize,
{
    pub operation_name: &'a str,
    pub query: &'a str,
    pub variables: V,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GraphqlResponse<T> {
    pub data: Option<T>,
    #[serde(default)]
    pub errors: Vec<GraphqlError>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GraphqlError {
    pub message: String,
    #[serde(default)]
    pub locations: Vec<GraphqlErrorLocation>,
    #[serde(default)]
    pub path: Vec<Value>,
    #[serde(default)]
    pub extensions: BTreeMap<String, Value>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GraphqlErrorLocation {
    pub line: u64,
    pub column: u64,
}

impl GraphqlError {
    pub fn summary(&self) -> String {
        if self.locations.is_empty()
            && self.path.is_empty()
            && self.extensions.is_empty()
            && self.extra.is_empty()
        {
            return self.message.clone();
        }

        serde_json::to_string(self).unwrap_or_else(|_| self.message.clone())
    }
}
