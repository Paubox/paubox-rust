//! Request types for submitting Paubox form responses.

use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use serde::Serialize;

use crate::error::PauboxError;

/// A form submission to be sent via [`crate::forms::FormsClient::submit_form`].
///
/// # Example
/// ```
/// use paubox::forms::{FormSubmission, FormAttachment};
/// use serde_json::json;
///
/// let submission = FormSubmission::builder()
///     .form_data(json!({"first_name": "Jane", "last_name": "Doe"}))
///     .build()
///     .unwrap();
/// ```
#[derive(Debug, Clone, Serialize)]
pub struct FormSubmission {
    /// Key-value pairs matching the form's field schema (`form_json`).
    /// The shape varies per form; use the `form_json` field from
    /// [`crate::forms::Form`] to discover field names.
    pub form_data: serde_json::Value,

    /// Optional file attachments.  Maximum total request size is 250 MB.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub attachments: Vec<FormAttachment>,
}

impl FormSubmission {
    /// Return a new [`FormSubmissionBuilder`].
    pub fn builder() -> FormSubmissionBuilder {
        FormSubmissionBuilder::default()
    }
}

/// A file attachment included with a form submission.
#[derive(Debug, Clone, Serialize)]
pub struct FormAttachment {
    /// Filename (e.g. `"consent.pdf"`).
    pub name: String,
    /// Base64-encoded file contents.
    pub content: String,
}

impl FormAttachment {
    /// Create an attachment by base64-encoding raw bytes.
    pub fn from_bytes(name: impl Into<String>, data: &[u8]) -> Self {
        Self {
            name: name.into(),
            content: B64.encode(data),
        }
    }

    /// Create an attachment from an already-base64-encoded string.
    pub fn from_base64(name: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            content: content.into(),
        }
    }
}

// ---------------------------------------------------------------------------
// Builder
// ---------------------------------------------------------------------------

/// Builder for [`FormSubmission`].
#[derive(Debug, Default)]
pub struct FormSubmissionBuilder {
    form_data: Option<serde_json::Value>,
    attachments: Vec<FormAttachment>,
}

impl FormSubmissionBuilder {
    /// Set the form field data.  Must not be null or empty.
    pub fn form_data(mut self, data: serde_json::Value) -> Self {
        self.form_data = Some(data);
        self
    }

    /// Add a single attachment.
    pub fn attachment(mut self, att: FormAttachment) -> Self {
        self.attachments.push(att);
        self
    }

    /// Add multiple attachments.
    pub fn attachments(mut self, atts: impl IntoIterator<Item = FormAttachment>) -> Self {
        self.attachments.extend(atts);
        self
    }

    /// Consume the builder, validating required fields.
    ///
    /// # Errors
    /// Returns [`PauboxError::Validation`] if `form_data` was not set or is
    /// `null`.
    pub fn build(self) -> Result<FormSubmission, PauboxError> {
        let form_data = self
            .form_data
            .ok_or_else(|| PauboxError::Validation("form_data is required".into()))?;
        if form_data.is_null() {
            return Err(PauboxError::Validation("form_data must not be null".into()));
        }
        Ok(FormSubmission {
            form_data,
            attachments: self.attachments,
        })
    }
}
