use std::{
    cell::RefCell,
    collections::HashMap,
    sync::{Arc, Mutex},
};

use mustache::Template;
use serde::Serialize;

pub struct Templates {
    templates: Mutex<RefCell<HashMap<String, Arc<Template>>>>,
}

impl Templates {
    pub fn new() -> mustache::Result<Templates> {
        Ok(Self {
            templates: Mutex::new(RefCell::new(HashMap::new())),
        })
    }

    pub fn login_form(&self) -> mustache::Result<String> {
        #[derive(Serialize)]
        struct Data {}

        self.render_page(
            "login form",
            include_str!("../templates/login-form.html"),
            &Data {},
        )
    }

    fn render_page<T>(&self, name: &str, source: &str, data: &T) -> mustache::Result<String>
    where
        T: Serialize,
    {
        #[derive(Serialize)]
        struct Data {
            contents: String,
        }

        let template = self.get_template("page", include_str!("../templates/page.html"))?;
        template.render_to_string(&Data {
            contents: self.render_snippet(name, source, data)?,
        })
    }

    fn render_snippet<T>(&self, name: &str, source: &str, data: &T) -> mustache::Result<String>
    where
        T: Serialize,
    {
        let template = self.get_template(name, source)?;
        template.render_to_string(data)
    }

    fn get_template(&self, name: &str, source: &str) -> mustache::Result<Arc<Template>> {
        let mut templates = self.templates.lock().unwrap();
        let templates = templates.get_mut();
        match templates.get(name) {
            Some(result) => Ok(result.clone()),
            None => {
                let result = Arc::new(mustache::compile_str(source)?);
                templates.insert(name.to_string(), result.clone());
                Ok(result)
            }
        }
    }
}
