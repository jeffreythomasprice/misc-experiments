use std::{
    collections::HashMap,
    fmt::Debug,
    sync::{Arc, Mutex},
};

use include_dir::include_dir;
use mustache::Template;

use poem::http::StatusCode;
use serde::Serialize;
use tracing::*;

use crate::http_utils::HttpError;

static TEMPLATES_DIR: include_dir::Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/templates");

#[derive(Debug)]
pub enum TemplateError {
    Compile,
    Render,
}

impl From<TemplateError> for HttpError {
    fn from(_value: TemplateError) -> Self {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

#[derive(Clone)]
pub struct TemplateService {
    templates: Arc<Mutex<HashMap<String, Arc<Template>>>>,
}

impl TemplateService {
    pub fn new() -> Self {
        Self {
            templates: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn get(&self, path: &str) -> Result<Arc<Template>, TemplateError> {
        let mut templates = self.templates.lock().unwrap();
        Ok(match templates.entry(path.to_owned()) {
            std::collections::hash_map::Entry::Occupied(e) => e.get().clone(),
            std::collections::hash_map::Entry::Vacant(e) => {
                let template_file = TEMPLATES_DIR.get_file(path).ok_or_else(|| {
                    error!("failed to find template: {path}");
                    TemplateError::Compile
                })?;
                let source = template_file.contents_utf8().ok_or_else(|| {
                    error!("not a utf8 file: {path}");
                    TemplateError::Compile
                })?;
                let result = Arc::new(mustache::compile_str(source).map_err(|e| {
                    error!("failed to compile template {path}: {e:?}");
                    TemplateError::Compile
                })?);
                e.insert(result.clone());
                result
            }
        })
    }

    pub fn render<T>(&self, path: &str, data: &T) -> Result<String, TemplateError>
    where
        T: Serialize,
    {
        let template = self.get(path)?;
        template.render_to_string(data).map_err(|e| {
            error!("failed to render template {path}: {e:?}");
            TemplateError::Render
        })
    }

    pub fn render_page(&self, content: &str) -> Result<String, TemplateError> {
        #[derive(Serialize)]
        struct Data<'a> {
            content: &'a str,
        }
        self.render("page.html", &Data { content })
    }
}
