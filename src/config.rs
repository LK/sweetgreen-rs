use std::path::PathBuf;

use directories::ProjectDirs;

pub const DEFAULT_GRAPHQL_ENDPOINT: &str = "https://order.sweetgreen.com/graphql";
pub const DEFAULT_ORDER_ORIGIN: &str = "https://order.sweetgreen.com";
pub const DEFAULT_ORDER_REFERER: &str = "https://order.sweetgreen.com/";
pub const DEFAULT_ORDER_APP_VERSION: &str = "7.3.1";
pub const DEFAULT_APOLLO_CLIENT_NAME: &str = "sweetgreen";
pub const DEFAULT_APOLLO_CLIENT_VERSION: &str = "7.3.1-5f65503e";

pub const AZURE_TENANT_ID: &str = "985add87-c20d-40cb-966b-8fdb588791d7";
pub const AZURE_TENANT_HOST: &str = "account.sweetgreen.com";
pub const AZURE_POLICY_NAME: &str = "B2C_1A_SignUpOrSignIn";
pub const AZURE_CLIENT_ID: &str = "8319d1eb-5f0c-48b0-ba9e-588e89ae7f26";
pub const AZURE_REDIRECT_URI: &str =
    "https://order.sweetgreen.com/embedded-frames/frames/azure-auth/auth.html";

pub const AZURE_API_SCOPE: &str = "https://sweetgreenb2c.onmicrosoft.com/api/Orders.ReadWrite";
pub const AZURE_VENDOR_SCOPE: &str =
    "https://sweetgreenb2c.onmicrosoft.com/vendor/Orders.ReadWrite";
pub const AZURE_OPENID_SCOPE: &str = "openid";
pub const AZURE_PROFILE_SCOPE: &str = "profile";
pub const AZURE_OFFLINE_ACCESS_SCOPE: &str = "offline_access";

pub const AZURE_API_ACCESS_TOKEN_HEADER: &str = "ApiAuthorizationToken";
pub const AZURE_VENDOR_ACCESS_TOKEN_HEADER: &str = "AuthorizationToken";

pub const CSRF_HEADER: &str = "x-csrf-token";
pub const BROWSER_USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/145.0.0.0 Safari/537.36";

pub fn azure_authority_base() -> String {
    format!(
        "https://{}/{}/{}",
        AZURE_TENANT_HOST, AZURE_TENANT_ID, AZURE_POLICY_NAME
    )
}

pub fn azure_authorize_endpoint() -> String {
    format!("{}/oauth2/v2.0/authorize", azure_authority_base())
}

pub fn azure_token_endpoint() -> String {
    format!(
        "https://{}/{}/b2c_1a_signuporsignin/oauth2/v2.0/token",
        AZURE_TENANT_HOST, AZURE_TENANT_ID
    )
}

pub fn default_state_path() -> PathBuf {
    if let Some(home_dir) = std::env::var_os("HOME") {
        return PathBuf::from(home_dir)
            .join(".sweetgreen")
            .join("state.json");
    }

    if let Some(project_dirs) = ProjectDirs::from("com", "lk", "sweetgreen-rs") {
        return project_dirs
            .config_dir()
            .join("sweetgreen")
            .join("state.json");
    }

    PathBuf::from(".sweetgreen-rs-state.json")
}

pub fn default_cookie_path() -> PathBuf {
    if let Some(home_dir) = std::env::var_os("HOME") {
        return PathBuf::from(home_dir)
            .join(".sweetgreen")
            .join("cookies.json");
    }

    if let Some(project_dirs) = ProjectDirs::from("com", "lk", "sweetgreen-rs") {
        return project_dirs
            .config_dir()
            .join("sweetgreen")
            .join("cookies.json");
    }

    PathBuf::from(".sweetgreen-rs-cookies.json")
}
