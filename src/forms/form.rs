//! Types for form definitions returned by the Paubox Forms API.

use serde::Deserialize;

/// A Paubox form definition returned by [`crate::forms::FormsClient::get_form`].
///
/// Forms are created in the Paubox dashboard.  This struct represents the
/// metadata and rendering data for a form retrieved by its UUID.
#[derive(Debug, Clone, Deserialize)]
pub struct Form {
    /// Unique identifier (UUID) of the form.
    pub id: String,

    /// Human-readable form title.
    pub title: String,

    /// Optional description.
    pub description: Option<String>,

    /// Rendered HTML for embedding the form in a web page.
    pub form_html: Option<String>,

    /// JSON schema describing the form fields.  Use this to validate
    /// `form_data` keys before calling
    /// [`crate::forms::FormsClient::submit_form`].
    pub form_json: Option<serde_json::Value>,

    /// CSS styles for the form.
    pub form_css: Option<String>,

    /// Whether the form is currently accepting submissions.
    pub active: bool,

    /// Whether the form supports electronic signatures.
    pub signable: bool,

    /// Total number of submissions received so far.
    pub submission_count: u64,

    /// ID of the Paubox customer account that owns this form.
    pub customer_id: u64,

    /// ISO 8601 timestamp when the form was created.
    pub created_at: String,

    /// ISO 8601 timestamp of the most recent update.
    pub updated_at: String,
}
