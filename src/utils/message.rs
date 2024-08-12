use serde::Serialize;
use validator::ValidationErrors;

#[derive(Serialize)]
pub struct Message {
    pub content: String,
    pub tags: String,
}

pub async fn handle_errors(errors: ValidationErrors) -> Vec<Message> {
    let mut messages = Vec::new();
    for error in errors.field_errors() {
        if let Some(message) = error.1.get(0).and_then(|m| m.message.as_ref()) {
            messages.push(Message {
                content: message.to_string(),
                tags: "danger".to_string(), // Example tag
            });
        }
    }
    return messages;
}

