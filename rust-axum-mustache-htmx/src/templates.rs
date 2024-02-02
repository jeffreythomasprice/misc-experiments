use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use serde::Serialize;

pub struct TemplateService {
    page_template: Arc<mustache::Template>,
    cache: Mutex<HashMap<String, Arc<mustache::Template>>>,
}

pub trait RenderableTemplate {
    fn render_to_string<T>(&self, data: &T) -> mustache::Result<String>
    where
        T: Serialize;
}

pub struct Snippet {
    template: Arc<mustache::Template>,
}

pub struct Page {
    page_template: Arc<mustache::Template>,
    content_template: Arc<mustache::Template>,
}

impl TemplateService {
    pub fn new() -> mustache::Result<TemplateService> {
        Ok(Self {
            page_template: Arc::new(mustache::compile_str(include_str!(
                "../templates/page.html"
            ))?),
            cache: Mutex::new(HashMap::new()),
        })
    }

    pub fn snippet(&self, source: &str) -> mustache::Result<Snippet> {
        Ok(Snippet {
            template: self.get(source)?,
        })
    }

    pub fn page(&self, source: &str) -> mustache::Result<Page> {
        Ok(Page {
            page_template: self.page_template.clone(),
            content_template: self.get(source)?,
        })
    }

    fn get(&self, source: &str) -> mustache::Result<Arc<mustache::Template>> {
        let mut cache = self.cache.lock().unwrap();
        Ok(match cache.entry(source.to_string()) {
            std::collections::hash_map::Entry::Occupied(value) => value.get().clone(),
            std::collections::hash_map::Entry::Vacant(entry) => {
                let result = Arc::new(mustache::compile_str(source)?);
                entry.insert(result.clone());
                result
            }
        })
    }
}

impl RenderableTemplate for Snippet {
    fn render_to_string<T>(&self, data: &T) -> mustache::Result<String>
    where
        T: Serialize,
    {
        self.template.render_to_string(data)
    }
}

impl RenderableTemplate for Page {
    fn render_to_string<T>(&self, data: &T) -> mustache::Result<String>
    where
        T: Serialize,
    {
        #[derive(Serialize)]
        struct Data {
            contents: String,
        }
        self.page_template.render_to_string(&Data {
            contents: self.content_template.render_to_string(data)?,
        })
    }
}
