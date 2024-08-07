pub mod users;

#[derive(Clone)]
pub struct APIService {
    base_url: String,
}

impl APIService {
    pub fn new(base_url: &str) -> Self {
        APIService {
            base_url: base_url.to_owned(),
        }
    }
}
