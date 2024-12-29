use crate::{concurrent_hashmap::ConcurrentHashMap, HttpError};
use anyhow::anyhow;
use mustache::Template;
use serde::Serialize;
use std::{fmt::Debug, path::Path, sync::Arc};

#[derive(Clone)]
pub struct Templates {
    templates: ConcurrentHashMap<String, Arc<Template>>,
}

impl Templates {
    pub fn new() -> Self {
        Self {
            templates: ConcurrentHashMap::new(),
        }
    }

    pub async fn template_path<P>(&mut self, path: P) -> Result<Arc<Template>, HttpError>
    where
        P: AsRef<Path> + Debug,
    {
        let key = format!("{:?}", path);
        self.templates
            .get_or_insert(key, || async {
                Ok(Arc::new(mustache::compile_path(&path).map_err(|e| {
                    anyhow!(
                        "error compiling template from path: {:?}, error: {:?}",
                        path,
                        e
                    )
                })?))
            })
            .await
    }

    pub async fn template_path_to_string<P, T>(
        &mut self,
        path: P,
        data: &T,
    ) -> Result<String, HttpError>
    where
        P: AsRef<Path> + Debug,
        T: Serialize,
    {
        let template = self.template_path(path).await?;
        let result = template
            .render_to_string(data)
            .map_err(|e| anyhow!("error rendering template: {e:?}"))?;
        Ok(result)
    }
}
