use leptos::window;
use log::*;

pub fn get_local_storage_string(name: &str) -> Option<String> {
    let storage = window().local_storage().unwrap_or_else(|e| {
        error!("failed to get local storage instance: {e:?}");
        None
    })?;
    storage.get_item(name).unwrap_or_else(|e| {
        error!("failed to get item from location storage, name: {name}, error: {e:?}");
        None
    })
}

pub fn set_local_storage_string(name: &str, value: Option<String>) {
    match window().local_storage() {
        Ok(Some(storage)) => {
            if let Err(e) = match value {
                Some(value) => storage.set_item(name, &value),
                None => storage.delete(name),
            } {
                error!("error updating local storage for name: {name}, error: {e:?}");
            }
        }
        Ok(None) => {
            error!("no local storage available");
        }
        Err(e) => {
            error!("failed to get local storage instance: {e:?}");
        }
    }
}
