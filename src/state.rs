use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::SweetgreenError;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuthMode {
    #[default]
    Local,
    Browser,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct AuthState {
    pub auth_mode: AuthMode,
    pub email: Option<String>,
    pub session_id: Option<String>,
    pub csrf_token: Option<String>,
    pub refresh_token: Option<String>,
    pub api_authorization_token: Option<String>,
    pub vendor_authorization_token: Option<String>,
}

impl AuthState {
    pub fn merge(&mut self, patch: AuthStatePatch) {
        if let Some(auth_mode) = patch.auth_mode {
            self.auth_mode = auth_mode;
        }

        if let Some(email) = patch.email {
            self.email = email;
        }

        if let Some(session_id) = patch.session_id {
            self.session_id = session_id;
        }

        if let Some(csrf_token) = patch.csrf_token {
            self.csrf_token = csrf_token;
        }

        if let Some(refresh_token) = patch.refresh_token {
            self.refresh_token = refresh_token;
        }

        if let Some(api_token) = patch.api_authorization_token {
            self.api_authorization_token = api_token;
        }

        if let Some(vendor_token) = patch.vendor_authorization_token {
            self.vendor_authorization_token = vendor_token;
        }
    }

    pub fn redacted(&self) -> RedactedAuthState {
        RedactedAuthState {
            auth_mode: self.auth_mode,
            email: self.email.clone(),
            has_session_id: self.session_id.is_some(),
            has_csrf_token: self.csrf_token.is_some(),
            has_refresh_token: self.refresh_token.is_some(),
            has_api_authorization_token: self.api_authorization_token.is_some(),
            has_vendor_authorization_token: self.vendor_authorization_token.is_some(),
            session_id_preview: preview(self.session_id.as_deref()),
            csrf_token_preview: preview(self.csrf_token.as_deref()),
            refresh_token_preview: preview(self.refresh_token.as_deref()),
            api_authorization_token_preview: preview(self.api_authorization_token.as_deref()),
            vendor_authorization_token_preview: preview(self.vendor_authorization_token.as_deref()),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct AuthStatePatch {
    pub auth_mode: Option<AuthMode>,
    pub email: Option<Option<String>>,
    pub session_id: Option<Option<String>>,
    pub csrf_token: Option<Option<String>>,
    pub refresh_token: Option<Option<String>>,
    pub api_authorization_token: Option<Option<String>>,
    pub vendor_authorization_token: Option<Option<String>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RedactedAuthState {
    pub auth_mode: AuthMode,
    pub email: Option<String>,
    pub has_session_id: bool,
    pub has_csrf_token: bool,
    pub has_refresh_token: bool,
    pub has_api_authorization_token: bool,
    pub has_vendor_authorization_token: bool,
    pub session_id_preview: Option<String>,
    pub csrf_token_preview: Option<String>,
    pub refresh_token_preview: Option<String>,
    pub api_authorization_token_preview: Option<String>,
    pub vendor_authorization_token_preview: Option<String>,
}

#[derive(Debug, Clone)]
pub struct StateStore {
    path: PathBuf,
}

impl StateStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn load(&self) -> Result<AuthState, SweetgreenError> {
        if !self.path.exists() {
            return Ok(AuthState::default());
        }

        let raw = fs::read_to_string(&self.path).map_err(|source| SweetgreenError::StateIo {
            path: self.path.display().to_string(),
            source,
        })?;

        let state = serde_json::from_str(&raw).map_err(|source| SweetgreenError::StateJson {
            path: self.path.display().to_string(),
            source,
        })?;

        Ok(state)
    }

    pub fn save(&self, state: &AuthState) -> Result<(), SweetgreenError> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|source| SweetgreenError::StateIo {
                path: parent.display().to_string(),
                source,
            })?;
        }

        let raw =
            serde_json::to_string_pretty(state).map_err(|source| SweetgreenError::StateJson {
                path: self.path.display().to_string(),
                source,
            })?;

        fs::write(&self.path, raw).map_err(|source| SweetgreenError::StateIo {
            path: self.path.display().to_string(),
            source,
        })
    }

    pub fn update<F>(&self, f: F) -> Result<AuthState, SweetgreenError>
    where
        F: FnOnce(&mut AuthState),
    {
        let mut state = self.load()?;
        f(&mut state);
        self.save(&state)?;
        Ok(state)
    }

    pub fn clear(&self) -> Result<(), SweetgreenError> {
        if !self.path.exists() {
            return Ok(());
        }

        fs::remove_file(&self.path).map_err(|source| SweetgreenError::StateIo {
            path: self.path.display().to_string(),
            source,
        })
    }
}

fn preview(value: Option<&str>) -> Option<String> {
    value.map(|value| {
        if value.len() <= 8 {
            return "********".to_string();
        }

        let prefix = &value[..4];
        let suffix = &value[value.len() - 4..];
        format!("{prefix}...{suffix}")
    })
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::{AuthMode, AuthStatePatch, StateStore};

    #[test]
    fn state_store_round_trip() {
        let dir = tempdir().unwrap();
        let store = StateStore::new(dir.path().join("state.json"));

        let state = store
            .update(|state| {
                state.merge(AuthStatePatch {
                    auth_mode: Some(AuthMode::Browser),
                    email: Some(Some("person@example.com".into())),
                    session_id: Some(Some("abc123".into())),
                    csrf_token: Some(Some("csrf123".into())),
                    refresh_token: Some(Some("refresh123".into())),
                    api_authorization_token: Some(Some("api123".into())),
                    vendor_authorization_token: Some(Some("vendor123".into())),
                });
            })
            .unwrap();

        assert_eq!(state.session_id.as_deref(), Some("abc123"));

        let loaded = store.load().unwrap();
        assert_eq!(loaded.auth_mode, AuthMode::Browser);
        assert_eq!(loaded.csrf_token.as_deref(), Some("csrf123"));
        assert_eq!(loaded.refresh_token.as_deref(), Some("refresh123"));
        assert_eq!(loaded.api_authorization_token.as_deref(), Some("api123"));
    }
}
