use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, Deserialize)]
pub struct GraphqlResponse<T> {
    pub data: Option<T>,
    #[serde(default)]
    pub errors: Vec<GraphqlError>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GraphqlError {
    pub message: String,
}
