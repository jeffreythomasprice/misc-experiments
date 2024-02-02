use std::{
    sync::{Arc},
};

use serde::Serialize;

pub struct TemplateService {
    page_template: Arc<mustache::Template>,
}

pub trait RenderableTemplate {
    fn render_to_string<T>(&self, data: &T) -> mustache::Result<String>
    where
        T: Serialize;
}

pub struct Snippet {
    template: mustache::Template,
}

pub struct Page {
    page_template: Arc<mustache::Template>,
    content_template: mustache::Template,
}

impl TemplateService {
    pub fn new() -> mustache::Result<TemplateService> {
        Ok(Self {
            page_template: Arc::new(mustache::compile_str(include_str!(
                "../templates/page.html"
            ))?),
        })
    }

    pub fn snippet(&self, source: &str) -> mustache::Result<Snippet> {
        Ok(Snippet {
            template: mustache::compile_str(source)?,
        })
    }

    pub fn page(&self, source: &str) -> mustache::Result<Page> {
        Ok(Page {
            page_template: self.page_template.clone(),
            content_template: mustache::compile_str(source)?,
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
