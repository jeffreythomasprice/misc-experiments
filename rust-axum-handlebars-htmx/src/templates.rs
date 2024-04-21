use std::sync::Arc;

use anyhow::anyhow;
use axum::http::HeaderMap;
use handlebars::Handlebars;
use include_dir::{include_dir, Dir};
use serde::Serialize;
use tokio::sync::RwLock;

static TEMPLATES_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/templates");

#[derive(Clone)]
pub struct Templates {
    h: Arc<RwLock<Handlebars<'static>>>,
}

impl Templates {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            h: Arc::new(RwLock::new(Handlebars::new())),
        })
    }

    pub async fn html_page<'a>(&self, content: &'a str) -> anyhow::Result<(HeaderMap, String)> {
        #[derive(Serialize)]
        struct Data<'a> {
            content: &'a str,
        }
        self.html_fragment(self.render("page", "page.html", &Data { content }).await?)
    }

    pub fn html_fragment(&self, content: String) -> anyhow::Result<(HeaderMap, String)> {
        let mut headers = HeaderMap::new();
        headers.insert("content-type", "text/html; charset=utf-8".parse()?);
        Ok((headers, content))
    }

    pub async fn render<T>(&self, name: &str, path: &str, data: &T) -> anyhow::Result<String>
    where
        T: Serialize,
    {
        {
            let h = self.h.read().await;
            if h.has_template(name) {
                return Ok(h.render(name, data)?);
            }
        }

        let file = TEMPLATES_DIR
            .get_file(path)
            .ok_or_else(|| anyhow!("no such file: {path}"))?;
        let file_contents = file
            .contents_utf8()
            .ok_or_else(|| anyhow!("failed to parse template contents as utf8: {path}"))?;

        let mut h = self.h.write().await;
        (*h).register_template_string(name, file_contents)?;
        Ok(h.render(name, data)?)
    }
}
