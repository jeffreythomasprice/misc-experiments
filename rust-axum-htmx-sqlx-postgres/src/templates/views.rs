use serde::Serialize;

use crate::{db, errors::HttpError};

use super::Templates;

#[derive(Serialize)]
pub struct Index {
    content: String,
}

impl Index {
    pub fn new(content: String) -> Self {
        Self { content }
    }

    pub async fn render(&self, templates: &mut Templates) -> Result<String, HttpError> {
        templates.template_path_to_string("templates/index.html", self).await
    }
}

#[derive(Serialize)]
pub struct Counter {
    clicks: u64,
}

impl Counter {
    pub fn new(clicks: u64) -> Self {
        Counter { clicks }
    }

    pub async fn render(&self, templates: &mut Templates) -> Result<String, HttpError> {
        templates.template_path_to_string("templates/counter.html", self).await
    }
}

#[derive(Serialize)]
pub struct Messages {}

impl Messages {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn render(&self, templates: &mut Templates) -> Result<String, HttpError> {
        templates.template_path_to_string("templates/messages.html", self).await
    }
}

#[derive(Serialize)]
pub struct NewMessage {
    content: String,
}

impl NewMessage {
    pub fn new(message: db::messages::Message) -> Self {
        Self {
            content: format!("{}: {}", message.sender, message.message),
        }
    }

    pub async fn render(&self, templates: &mut Templates) -> Result<String, HttpError> {
        templates.template_path_to_string("templates/new-message.html", self).await
    }
}
