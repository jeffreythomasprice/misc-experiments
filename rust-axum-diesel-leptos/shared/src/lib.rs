use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Example {
    pub foo: String,
    pub bar: i32,
}
