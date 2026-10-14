use crate::component::account::User;
use dioxus_logger::tracing::warn;

const STORAGE_KEY: &str = "mirams.session";

fn storage() -> Option<web_sys::Storage> {
    match web_sys::window()?.local_storage() {
        Ok(storage) => storage,
        Err(_) => {
            warn!("Browser storage is unavailable; login will last for this page only");
            None
        }
    }
}

/// Restore the login before the router makes its first authenticated requests.
pub fn load_user() -> Option<User> {
    let storage = storage()?;
    let saved = match storage.get_item(STORAGE_KEY) {
        Ok(saved) => saved?,
        Err(_) => {
            warn!("Could not read the saved login");
            return None;
        }
    };
    match serde_json::from_str::<User>(&saved) {
        Ok(user) if !user.username.is_empty() && !user.api_token.is_empty() => Some(user),
        _ => {
            warn!("Discarding an invalid saved login");
            let _ = storage.remove_item(STORAGE_KEY);
            None
        }
    }
}

/// Persist successful logins, or remove the saved login on logout.
pub fn save_user(user: Option<&User>) {
    let Some(storage) = storage() else {
        return;
    };
    let result = match user {
        Some(user) => {
            let Ok(saved) = serde_json::to_string(user) else {
                warn!("Could not serialize the login");
                return;
            };
            storage.set_item(STORAGE_KEY, &saved)
        }
        None => storage.remove_item(STORAGE_KEY),
    };
    if result.is_err() {
        warn!("Could not update the saved login");
    }
}
