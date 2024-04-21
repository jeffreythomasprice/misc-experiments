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

    pub async fn counter_page(&self, count: u64) -> anyhow::Result<(HeaderMap, String)> {
        #[derive(Serialize)]
        struct Data {
            count: u64,
        }
        self.page(
            &self
                .render_template_to_string("counter", "counter.html", &Data { count })
                .await?,
        )
        .await
    }

    pub async fn click_response(&self, count: u64) -> anyhow::Result<(HeaderMap, String)> {
        #[derive(Serialize)]
        struct Data {
            count: u64,
        }
        self.fragment(
            self.render_template_to_string(
                "click-response",
                "click-response.html",
                &Data { count },
            )
            .await?,
        )
    }

    async fn page<'a>(&self, content: &'a str) -> anyhow::Result<(HeaderMap, String)> {
        #[derive(Serialize)]
        struct Data<'a> {
            content: &'a str,
        }
        self.fragment(
            self.render_template_to_string("page", "page.html", &Data { content })
                .await?,
        )
    }

    fn fragment(&self, content: String) -> anyhow::Result<(HeaderMap, String)> {
        let mut headers = HeaderMap::new();
        headers.insert("content-type", "text/html; charset=utf-8".parse()?);
        Ok((headers, content))
    }

    async fn render_template_to_string<T>(
        &self,
        name: &str,
        path: &str,
        data: &T,
    ) -> anyhow::Result<String>
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
