use once_cell::sync::Lazy;
use std::sync::Mutex;

#[derive(Debug, Clone, PartialEq)]
pub struct AccountInfo {
    pub name: String,
    pub peer_id: String,
    pub address: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AuthState {
    Checking,
    Unregistered,
    Unauthenticated,
    Authenticated(AccountInfo),
}

impl Default for AuthState {
    fn default() -> Self {
        AuthState::Checking
    }
}

impl AuthState {
    pub fn get_account_info(&self) -> Option<&AccountInfo> {
        match self {
            AuthState::Authenticated(info) => Some(info),
            _ => None,
        }
    }
}

static AUTH_STATE: Lazy<Mutex<AuthState>> = Lazy::new(|| Mutex::new(AuthState::Checking));

pub struct AuthContext;

impl AuthContext {
    pub fn set_authenticated(info: AccountInfo) {
        if let Ok(mut state) = AUTH_STATE.lock() {
            *state = AuthState::Authenticated(info);
        }
    }

    pub fn set_unauthenticated() {
        if let Ok(mut state) = AUTH_STATE.lock() {
            *state = AuthState::Unauthenticated;
        }
    }

    pub fn set_unregistered() {
        if let Ok(mut state) = AUTH_STATE.lock() {
            *state = AuthState::Unregistered;
        }
    }

    pub fn get_state() -> AuthState {
        AUTH_STATE
            .lock()
            .map(|s| (*s).clone())
            .unwrap_or(AuthState::Checking)
    }

    pub fn reset() {
        if let Ok(mut state) = AUTH_STATE.lock() {
            *state = AuthState::Checking;
        }
    }
}

pub fn use_auth() -> AuthContext {
    AuthContext
}
