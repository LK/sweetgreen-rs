use thiserror::Error;

#[derive(Debug, Error)]
pub enum SweetgreenError {
    #[error("failed to read/write auth state at {path}: {source}")]
    StateIo {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to parse auth state JSON at {path}: {source}")]
    StateJson {
        path: String,
        #[source]
        source: serde_json::Error,
    },

    #[error("http request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("graphql response parse failed: {0}")]
    GraphqlJson(#[from] serde_json::Error),

    #[error("graphql returned no data")]
    MissingGraphqlData,

    #[error("graphql returned errors: {0}")]
    GraphqlErrors(String),

    #[error("authentication error: {0}")]
    Auth(String),

    #[error("missing required auth state: {0}")]
    MissingAuthState(&'static str),

    #[error("invalid argument: {0}")]
    InvalidArgument(String),
}
