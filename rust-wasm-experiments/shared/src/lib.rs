use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonResponse {
    foo: String,
    bar: i32,
}

impl JsonResponse {
    pub fn new(foo: &str, bar: i32) -> Self {
        Self {
            foo: foo.into(),
            bar,
        }
    }
}
