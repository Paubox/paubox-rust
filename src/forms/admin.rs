//! Administrative Forms API — manage forms and export submissions.
//!
//! Every endpoint in this module requires a [`FormsClient`] constructed with
//! a scoped API key ([`FormsClient::with_api_key`] or
//! [`FormsClient::builder`]).  The key must carry the `"forms"` scope; see
//! the [module docs](crate::forms) for details on the authentication model.

use serde::{Deserialize, Serialize};
use url::Url;

use super::{handle_bytes, handle_response, handle_unit, Form, FormsClient};
use crate::error::PauboxError;

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

/// One page of forms returned by [`FormsClient::list_forms`].
#[derive(Debug, Clone, Deserialize)]
pub struct FormPage {
    /// The forms on this page.
    pub results: Vec<Form>,
    /// Pagination metadata for the full result set.
    pub page_info: PageInfo,
}

/// Pagination metadata attached to a [`FormPage`].
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct PageInfo {
    /// Total number of forms matching the query (across all pages).
    pub count: i64,
    /// Total number of pages.
    pub pages: u32,
    /// The 1-based index of this page.
    pub page: u32,
    /// Number of items requested per page.
    pub items: u32,
}

/// Aggregate form statistics returned by [`FormsClient::form_stats`].
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct FormStats {
    /// Number of active, non-archived, non-deleted forms.
    pub active_form_count: i64,
    /// Total submissions received across all forms.
    pub total_submission_count: i64,
    /// Submissions received in the last 7 days.
    pub submissions_last_7_days: i64,
}

/// One page of submissions returned by [`FormsClient::list_submissions`].
#[derive(Debug, Clone, Deserialize)]
pub struct SubmissionPage {
    /// The submissions on this page.
    pub data: Vec<Submission>,
    /// Total number of submissions matching the query (across all pages).
    pub total: i64,
    /// The 1-based index of this page.
    pub page: i64,
    /// Number of items requested per page.
    pub items: i64,
}

/// A single form submission returned by [`FormsClient::list_submissions`].
#[derive(Debug, Clone, Deserialize)]
pub struct Submission {
    /// Unique identifier (UUID) of the submission.
    pub id: String,

    /// UUID of the form this submission belongs to.
    pub form_id: String,

    /// The submitted field values as a **JSON-encoded string**.  Use
    /// [`Submission::form_data_json`] to parse it into a
    /// [`serde_json::Value`].
    pub form_data: String,

    /// Where the submission payload is stored (e.g. `"database"`).
    pub storage_type: String,

    /// External storage URL for the payload, if any.
    #[serde(default)]
    pub storage_url: Option<String>,

    /// Email address of the respondent, if captured.
    #[serde(default)]
    pub submitter_email: Option<String>,

    /// Comma-separated list of notification recipients, if any.
    #[serde(default)]
    pub recipients: Option<String>,

    /// Storage key of an uploaded attachment, if any.
    #[serde(default)]
    pub attachment: Option<String>,

    /// Original filename of an uploaded attachment, if any.
    #[serde(default)]
    pub attachment_name: Option<String>,

    /// Download URL of an uploaded attachment, if any.
    #[serde(default)]
    pub attachment_url: Option<String>,

    /// MIME type of an uploaded attachment, if any.
    #[serde(default)]
    pub attachment_type: Option<String>,

    /// RFC 3339 timestamp when the submission was received.
    pub created_at: String,
}

impl Submission {
    /// Parse [`Submission::form_data`] (a JSON-encoded string) into a
    /// [`serde_json::Value`].
    ///
    /// # Errors
    /// Returns [`PauboxError::Deserialize`] if `form_data` is not valid JSON.
    pub fn form_data_json(&self) -> Result<serde_json::Value, PauboxError> {
        Ok(serde_json::from_str(&self.form_data)?)
    }
}

/// Wrapper for the `GET api/forms/:id` response, which nests the form under
/// a `"data"` key.
#[derive(Deserialize)]
struct FormDataWrapper {
    data: Form,
}

/// Wrapper for the `POST api/forms` response.
#[derive(Deserialize)]
struct CreateFormResponse {
    id: String,
}

/// Request body for `POST api/forms/copy`.
#[derive(Serialize)]
struct CopyFormRequest<'a> {
    form_id: &'a str,
    title: &'a str,
}

// ---------------------------------------------------------------------------
// CreateForm
// ---------------------------------------------------------------------------

/// A new form definition to be created via [`FormsClient::create_form`].
///
/// Construct with [`CreateForm::builder`]; `title`, `customer_id`, and
/// `form_json` are required, `version` defaults to `1`, and everything else
/// is optional.
///
/// # Example
/// ```
/// use paubox::forms::CreateForm;
/// use serde_json::json;
///
/// let form = CreateForm::builder()
///     .title("Patient intake")
///     .customer_id(42)
///     .form_json(json!({"fields": []}))
///     .build()
///     .unwrap();
/// ```
#[derive(Debug, Clone, Serialize)]
pub struct CreateForm {
    title: String,
    form_json: serde_json::Value,
    customer_id: i64,
    version: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    form_html: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    form_css: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    recipient: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    signable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    signature_confirmation_label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    subscription_list_id: Option<String>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    type_: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    active: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    submission_count: Option<i64>,
}

impl CreateForm {
    /// Return a new [`CreateFormBuilder`].
    pub fn builder() -> CreateFormBuilder {
        CreateFormBuilder::default()
    }
}

/// Builder for [`CreateForm`].
#[derive(Debug, Default)]
pub struct CreateFormBuilder {
    title: Option<String>,
    form_json: Option<serde_json::Value>,
    customer_id: Option<i64>,
    version: Option<i64>,
    description: Option<String>,
    form_html: Option<String>,
    form_css: Option<String>,
    recipient: Option<String>,
    signable: Option<bool>,
    signature_confirmation_label: Option<String>,
    subscription_list_id: Option<String>,
    type_: Option<String>,
    active: Option<bool>,
    submission_count: Option<i64>,
}

impl CreateFormBuilder {
    /// Set the form title (**required**).
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the JSON schema describing the form fields (**required**).
    pub fn form_json(mut self, form_json: serde_json::Value) -> Self {
        self.form_json = Some(form_json);
        self
    }

    /// Set the ID of the customer account that will own the form
    /// (**required**).
    pub fn customer_id(mut self, customer_id: i64) -> Self {
        self.customer_id = Some(customer_id);
        self
    }

    /// Set the form version.  Defaults to `1` when not set.
    pub fn version(mut self, version: i64) -> Self {
        self.version = Some(version);
        self
    }

    /// Set an optional description.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set rendered HTML for embedding the form in a web page.
    pub fn form_html(mut self, form_html: impl Into<String>) -> Self {
        self.form_html = Some(form_html.into());
        self
    }

    /// Set CSS styles for the form.
    pub fn form_css(mut self, form_css: impl Into<String>) -> Self {
        self.form_css = Some(form_css.into());
        self
    }

    /// Set the notification recipients as a comma-separated email string.
    pub fn recipient(mut self, recipient: impl Into<String>) -> Self {
        self.recipient = Some(recipient.into());
        self
    }

    /// Set whether the form supports electronic signatures.
    pub fn signable(mut self, signable: bool) -> Self {
        self.signable = Some(signable);
        self
    }

    /// Set the label shown next to the signature confirmation checkbox.
    pub fn signature_confirmation_label(mut self, label: impl Into<String>) -> Self {
        self.signature_confirmation_label = Some(label.into());
        self
    }

    /// Connect a Paubox Marketing contact list by ID.
    pub fn subscription_list_id(mut self, id: impl Into<String>) -> Self {
        self.subscription_list_id = Some(id.into());
        self
    }

    /// Set the form type (serialized as `type` on the wire).
    pub fn type_(mut self, type_: impl Into<String>) -> Self {
        self.type_ = Some(type_.into());
        self
    }

    /// Set whether the form is immediately active.
    pub fn active(mut self, active: bool) -> Self {
        self.active = Some(active);
        self
    }

    /// Set the initial submission count.
    pub fn submission_count(mut self, count: i64) -> Self {
        self.submission_count = Some(count);
        self
    }

    /// Consume the builder, validating required fields.
    ///
    /// # Errors
    /// Returns [`PauboxError::Validation`] if `title`, `customer_id`, or
    /// `form_json` was not set.
    pub fn build(self) -> Result<CreateForm, PauboxError> {
        let title = self
            .title
            .ok_or_else(|| PauboxError::Validation("title is required".into()))?;
        let customer_id = self
            .customer_id
            .ok_or_else(|| PauboxError::Validation("customer_id is required".into()))?;
        let form_json = self
            .form_json
            .ok_or_else(|| PauboxError::Validation("form_json is required".into()))?;

        Ok(CreateForm {
            title,
            form_json,
            customer_id,
            version: self.version.unwrap_or(1),
            description: self.description,
            form_html: self.form_html,
            form_css: self.form_css,
            recipient: self.recipient,
            signable: self.signable,
            signature_confirmation_label: self.signature_confirmation_label,
            subscription_list_id: self.subscription_list_id,
            type_: self.type_,
            active: self.active,
            submission_count: self.submission_count,
        })
    }
}

// ---------------------------------------------------------------------------
// UpdateForm
// ---------------------------------------------------------------------------

/// A PATCH-style partial update for [`FormsClient::update_form`].
///
/// Every field is optional; fields left unset are omitted from the request
/// body and keep their current value on the server.
///
/// # Example
/// ```
/// use paubox::forms::UpdateForm;
///
/// let update = UpdateForm::default()
///     .title("New title")
///     .active(false);
/// ```
#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateForm {
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    form_json: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    vanity_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    recipient: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    active: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    subscription_list_id: Option<String>,
}

impl UpdateForm {
    /// Set a new form title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set a new description.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Replace the JSON schema describing the form fields.
    pub fn form_json(mut self, form_json: serde_json::Value) -> Self {
        self.form_json = Some(form_json);
        self
    }

    /// Set a new vanity URL slug.
    pub fn vanity_url(mut self, vanity_url: impl Into<String>) -> Self {
        self.vanity_url = Some(vanity_url.into());
        self
    }

    /// Set the notification recipients as a comma-separated email string.
    pub fn recipient(mut self, recipient: impl Into<String>) -> Self {
        self.recipient = Some(recipient.into());
        self
    }

    /// Activate or deactivate the form.
    pub fn active(mut self, active: bool) -> Self {
        self.active = Some(active);
        self
    }

    /// Connect a Paubox Marketing contact list by ID.
    pub fn subscription_list_id(mut self, id: impl Into<String>) -> Self {
        self.subscription_list_id = Some(id.into());
        self
    }
}

// ---------------------------------------------------------------------------
// List parameters
// ---------------------------------------------------------------------------

/// Query parameters for [`FormsClient::list_forms`].  All fields are
/// optional on the wire; unset fields use the server defaults.  Note that
/// [`FormListParams::customer_id`] is effectively required for API-key
/// callers — see [`FormsClient::list_forms`].
///
/// # Example
/// ```
/// use paubox::forms::FormListParams;
///
/// let params = FormListParams::default()
///     .customer_id(42)
///     .search("intake")
///     .active(true)
///     .order_by("updated_at")
///     .order("asc")
///     .page(2)
///     .items(25);
/// ```
#[derive(Debug, Clone, Default)]
pub struct FormListParams {
    customer_id: Option<i64>,
    form_id: Option<String>,
    search: Option<String>,
    order: Option<String>,
    order_by: Option<String>,
    archived: Option<bool>,
    active: Option<bool>,
    page: Option<u32>,
    items: Option<u32>,
}

impl FormListParams {
    /// Filter to forms owned by this customer ID.
    ///
    /// Effectively **required** for API-key callers: the server only
    /// authorizes [`FormsClient::list_forms`] when this matches (or is
    /// related to) the customer the API key belongs to, and returns
    /// HTTP 403 when the parameter is omitted.
    pub fn customer_id(mut self, customer_id: i64) -> Self {
        self.customer_id = Some(customer_id);
        self
    }

    /// Filter to a single form by its UUID.
    pub fn form_id(mut self, form_id: impl Into<String>) -> Self {
        self.form_id = Some(form_id.into());
        self
    }

    /// Filter to forms whose title or description contains this text.
    pub fn search(mut self, search: impl Into<String>) -> Self {
        self.search = Some(search.into());
        self
    }

    /// Sort direction: `"asc"` or `"desc"` (server default: `"desc"`).
    pub fn order(mut self, order: impl Into<String>) -> Self {
        self.order = Some(order.into());
        self
    }

    /// Sort column: `"title"`, `"updated_at"`, or `"submission_count"`
    /// (server default: `created_at`).
    pub fn order_by(mut self, order_by: impl Into<String>) -> Self {
        self.order_by = Some(order_by.into());
        self
    }

    /// Filter by archived state.
    pub fn archived(mut self, archived: bool) -> Self {
        self.archived = Some(archived);
        self
    }

    /// Filter by active state.
    pub fn active(mut self, active: bool) -> Self {
        self.active = Some(active);
        self
    }

    /// Select a page (1-based; server default: 1).
    pub fn page(mut self, page: u32) -> Self {
        self.page = Some(page);
        self
    }

    /// Items per page (server default: 50, capped at 100).
    pub fn items(mut self, items: u32) -> Self {
        self.items = Some(items);
        self
    }

    /// Append the set parameters to `url` as query pairs.
    fn apply(&self, url: &mut Url) {
        {
            let mut pairs = url.query_pairs_mut();
            if let Some(v) = self.customer_id {
                pairs.append_pair("customer_id", &v.to_string());
            }
            if let Some(v) = &self.form_id {
                pairs.append_pair("form_id", v);
            }
            if let Some(v) = &self.search {
                pairs.append_pair("search", v);
            }
            if let Some(v) = &self.order {
                pairs.append_pair("order", v);
            }
            if let Some(v) = &self.order_by {
                pairs.append_pair("order_by", v);
            }
            if let Some(v) = self.archived {
                pairs.append_pair("archived", if v { "true" } else { "false" });
            }
            if let Some(v) = self.active {
                pairs.append_pair("active", if v { "true" } else { "false" });
            }
            if let Some(v) = self.page {
                pairs.append_pair("page", &v.to_string());
            }
            if let Some(v) = self.items {
                pairs.append_pair("items", &v.to_string());
            }
        }
        strip_empty_query(url);
    }
}

/// Query parameters for [`FormsClient::list_submissions`].  All fields are
/// optional; unset fields use the server defaults.
///
/// # Example
/// ```
/// use paubox::forms::SubmissionListParams;
///
/// let params = SubmissionListParams::default()
///     .order_by("submitter_email")
///     .order("asc")
///     .page(1)
///     .items(100);
/// ```
#[derive(Debug, Clone, Default)]
pub struct SubmissionListParams {
    submission_id: Option<String>,
    order: Option<String>,
    order_by: Option<String>,
    page: Option<i64>,
    items: Option<i64>,
}

impl SubmissionListParams {
    /// Filter to a single submission by its UUID.
    pub fn submission_id(mut self, submission_id: impl Into<String>) -> Self {
        self.submission_id = Some(submission_id.into());
        self
    }

    /// Sort direction: `"asc"` or `"desc"`.
    pub fn order(mut self, order: impl Into<String>) -> Self {
        self.order = Some(order.into());
        self
    }

    /// Sort column: `"submitter_email"` (server default: `created_at`).
    pub fn order_by(mut self, order_by: impl Into<String>) -> Self {
        self.order_by = Some(order_by.into());
        self
    }

    /// Select a page (1-based).
    pub fn page(mut self, page: i64) -> Self {
        self.page = Some(page);
        self
    }

    /// Items per page (capped at 100 by the server).
    pub fn items(mut self, items: i64) -> Self {
        self.items = Some(items);
        self
    }

    /// Append the set parameters to `url` as query pairs.
    fn apply(&self, url: &mut Url) {
        {
            let mut pairs = url.query_pairs_mut();
            if let Some(v) = &self.submission_id {
                pairs.append_pair("submission_id", v);
            }
            if let Some(v) = &self.order {
                pairs.append_pair("order", v);
            }
            if let Some(v) = &self.order_by {
                pairs.append_pair("order_by", v);
            }
            if let Some(v) = self.page {
                pairs.append_pair("page", &v.to_string());
            }
            if let Some(v) = self.items {
                pairs.append_pair("items", &v.to_string());
            }
        }
        strip_empty_query(url);
    }
}

/// [`Url::query_pairs_mut`] leaves an empty query (a bare trailing `?`) when
/// no pairs were appended; remove it so parameter-less requests keep a clean
/// URL.
fn strip_empty_query(url: &mut Url) {
    if url.query() == Some("") {
        url.set_query(None);
    }
}

// ---------------------------------------------------------------------------
// Endpoint methods
// ---------------------------------------------------------------------------

impl FormsClient {
    /// List forms, filtered and paginated by `params`.
    ///
    /// Requires an API key with the `"forms"` scope.
    ///
    /// For API-key callers, [`FormListParams::customer_id`] is effectively
    /// **required**: the server authorizes this endpoint only when the
    /// `customer_id` parameter matches (or is related to) the customer the
    /// key belongs to, and rejects requests that omit it with HTTP 403.
    ///
    /// # Errors
    /// - [`PauboxError::Auth`] — the client has no API key (checked before
    ///   any network call), or the server rejected the key (HTTP 401)
    /// - [`PauboxError::Http`] — missing or cross-customer `customer_id`
    ///   (403) or any other non-2xx response
    /// - [`PauboxError::Request`] — network failure
    /// - [`PauboxError::Deserialize`] — unexpected response shape
    ///
    /// # Example
    /// ```no_run
    /// use paubox::forms::{FormsClient, FormListParams};
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = FormsClient::with_api_key("key-with-forms-scope");
    /// let params = FormListParams::default()
    ///     .customer_id(42) // required for API-key callers
    ///     .active(true)
    ///     .items(25);
    /// let page = client.list_forms(&params).await?;
    /// println!("{} of {} forms", page.results.len(), page.page_info.count);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_forms(&self, params: &FormListParams) -> Result<FormPage, PauboxError> {
        let auth = self.bearer_auth()?;
        let mut url = self.base_url.join("api/forms")?;
        params.apply(&mut url);

        let resp = self
            .http
            .get(url)
            .header("Authorization", auth)
            .header("Accept", "application/json")
            .send()
            .await?;

        handle_response::<FormPage>(resp).await
    }

    /// Create a new form and return its ID.
    ///
    /// Requires an API key with the `"forms"` scope.
    ///
    /// # Errors
    /// Same variants as [`FormsClient::list_forms`].
    ///
    /// # Example
    /// ```no_run
    /// use paubox::forms::{CreateForm, FormsClient};
    /// use serde_json::json;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = FormsClient::with_api_key("key-with-forms-scope");
    /// let form = CreateForm::builder()
    ///     .title("Patient intake")
    ///     .customer_id(42)
    ///     .form_json(json!({"fields": []}))
    ///     .build()?;
    /// let id = client.create_form(&form).await?;
    /// println!("Created form {}", id);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_form(&self, form: &CreateForm) -> Result<String, PauboxError> {
        let auth = self.bearer_auth()?;
        let url = self.base_url.join("api/forms")?;

        let resp = self
            .http
            .post(url)
            .header("Authorization", auth)
            .header("Content-Type", "application/json")
            .json(form)
            .send()
            .await?;

        Ok(handle_response::<CreateFormResponse>(resp).await?.id)
    }

    /// Retrieve a form by its UUID via the authenticated endpoint.
    ///
    /// Unlike the public [`FormsClient::get_form`], this endpoint requires an
    /// API key with the `"forms"` scope and also returns inactive or archived
    /// forms.
    ///
    /// # Errors
    /// Same variants as [`FormsClient::list_forms`]; a missing form surfaces
    /// as [`PauboxError::Http`] (the server currently reports it with
    /// status 500).
    ///
    /// # Example
    /// ```no_run
    /// use paubox::forms::FormsClient;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = FormsClient::with_api_key("key-with-forms-scope");
    /// let form = client.get_form_by_id("550e8400-e29b-41d4-a716-446655440000").await?;
    /// println!("{}: archived={}", form.title, form.archived);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_form_by_id(&self, id: &str) -> Result<Form, PauboxError> {
        let auth = self.bearer_auth()?;
        let url = self.base_url.join(&format!("api/forms/{}", id))?;

        let resp = self
            .http
            .get(url)
            .header("Authorization", auth)
            .header("Accept", "application/json")
            .send()
            .await?;

        Ok(handle_response::<FormDataWrapper>(resp).await?.data)
    }

    /// Apply a partial update to a form.  Fields left unset on `update` keep
    /// their current values.
    ///
    /// Requires an API key with the `"forms"` scope.
    ///
    /// # Errors
    /// Same variants as [`FormsClient::list_forms`]; a missing form is
    /// [`PauboxError::Http`] with status 404.
    ///
    /// # Example
    /// ```no_run
    /// use paubox::forms::{FormsClient, UpdateForm};
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = FormsClient::with_api_key("key-with-forms-scope");
    /// let update = UpdateForm::default().title("Renamed form").active(false);
    /// client.update_form("550e8400-e29b-41d4-a716-446655440000", &update).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn update_form(&self, id: &str, update: &UpdateForm) -> Result<(), PauboxError> {
        let auth = self.bearer_auth()?;
        let url = self.base_url.join(&format!("api/forms/{}", id))?;

        let resp = self
            .http
            .put(url)
            .header("Authorization", auth)
            .header("Content-Type", "application/json")
            .json(update)
            .send()
            .await?;

        handle_unit(resp).await
    }

    /// Archive a form (also deactivates it).
    ///
    /// Requires an API key with the `"forms"` scope.
    ///
    /// # Errors
    /// Same variants as [`FormsClient::list_forms`].
    ///
    /// # Example
    /// ```no_run
    /// use paubox::forms::FormsClient;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = FormsClient::with_api_key("key-with-forms-scope");
    /// client.archive_form("550e8400-e29b-41d4-a716-446655440000").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn archive_form(&self, id: &str) -> Result<(), PauboxError> {
        self.post_form_action(id, "archive").await
    }

    /// Unarchive a previously archived form.
    ///
    /// Requires an API key with the `"forms"` scope.
    ///
    /// # Errors
    /// Same variants as [`FormsClient::list_forms`].
    ///
    /// # Example
    /// ```no_run
    /// use paubox::forms::FormsClient;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = FormsClient::with_api_key("key-with-forms-scope");
    /// client.unarchive_form("550e8400-e29b-41d4-a716-446655440000").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn unarchive_form(&self, id: &str) -> Result<(), PauboxError> {
        self.post_form_action(id, "unarchive").await
    }

    /// Shared POST for the archive/unarchive endpoints.
    async fn post_form_action(&self, id: &str, action: &str) -> Result<(), PauboxError> {
        let auth = self.bearer_auth()?;
        let url = self
            .base_url
            .join(&format!("api/forms/{}/{}", id, action))?;

        let resp = self
            .http
            .post(url)
            .header("Authorization", auth)
            .header("Accept", "application/json")
            .send()
            .await?;

        handle_unit(resp).await
    }

    /// Duplicate an existing form under a new title and return the copy.
    ///
    /// Requires an API key with the `"forms"` scope.
    ///
    /// # Errors
    /// Same variants as [`FormsClient::list_forms`]; a missing original form
    /// is [`PauboxError::Http`] with status 404.
    ///
    /// # Example
    /// ```no_run
    /// use paubox::forms::FormsClient;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = FormsClient::with_api_key("key-with-forms-scope");
    /// let copy = client
    ///     .copy_form("550e8400-e29b-41d4-a716-446655440000", "Intake (copy)")
    ///     .await?;
    /// println!("New form {}", copy.id);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn copy_form(&self, form_id: &str, title: &str) -> Result<Form, PauboxError> {
        let auth = self.bearer_auth()?;
        let url = self.base_url.join("api/forms/copy")?;

        let resp = self
            .http
            .post(url)
            .header("Authorization", auth)
            .header("Content-Type", "application/json")
            .json(&CopyFormRequest { form_id, title })
            .send()
            .await?;

        handle_response::<Form>(resp).await
    }

    /// Retrieve aggregate form statistics.
    ///
    /// When `customer_id` is `None` the server defaults to the customer that
    /// owns the API key.  Requires an API key with the `"forms"` scope.
    ///
    /// # Errors
    /// Same variants as [`FormsClient::list_forms`].
    ///
    /// # Example
    /// ```no_run
    /// use paubox::forms::FormsClient;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = FormsClient::with_api_key("key-with-forms-scope");
    /// let stats = client.form_stats(None).await?;
    /// println!(
    ///     "{} active forms, {} total submissions",
    ///     stats.active_form_count, stats.total_submission_count
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub async fn form_stats(&self, customer_id: Option<i64>) -> Result<FormStats, PauboxError> {
        let auth = self.bearer_auth()?;
        let mut url = self.base_url.join("api/forms/stats")?;
        if let Some(id) = customer_id {
            url.query_pairs_mut()
                .append_pair("customer_id", &id.to_string());
        }

        let resp = self
            .http
            .get(url)
            .header("Authorization", auth)
            .header("Accept", "application/json")
            .send()
            .await?;

        handle_response::<FormStats>(resp).await
    }

    /// List submissions for a form, filtered and paginated by `params`.
    ///
    /// Requires an API key with the `"forms"` scope.
    ///
    /// # Errors
    /// Same variants as [`FormsClient::list_forms`].
    ///
    /// # Example
    /// ```no_run
    /// use paubox::forms::{FormsClient, SubmissionListParams};
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = FormsClient::with_api_key("key-with-forms-scope");
    /// let page = client
    ///     .list_submissions(
    ///         "550e8400-e29b-41d4-a716-446655440000",
    ///         &SubmissionListParams::default().items(100),
    ///     )
    ///     .await?;
    /// for sub in &page.data {
    ///     println!("{}: {:?}", sub.id, sub.form_data_json()?);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_submissions(
        &self,
        form_id: &str,
        params: &SubmissionListParams,
    ) -> Result<SubmissionPage, PauboxError> {
        let auth = self.bearer_auth()?;
        let mut url = self
            .base_url
            .join(&format!("api/forms/{}/submissions", form_id))?;
        params.apply(&mut url);

        let resp = self
            .http
            .get(url)
            .header("Authorization", auth)
            .header("Accept", "application/json")
            .send()
            .await?;

        handle_response::<SubmissionPage>(resp).await
    }

    /// Export **all** submissions for a form as raw CSV bytes.
    ///
    /// Requires an API key with the `"forms"` scope.
    ///
    /// # Errors
    /// Same variants as [`FormsClient::list_forms`].
    ///
    /// # Example
    /// ```no_run
    /// use paubox::forms::FormsClient;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = FormsClient::with_api_key("key-with-forms-scope");
    /// let csv = client
    ///     .export_submissions_csv("550e8400-e29b-41d4-a716-446655440000")
    ///     .await?;
    /// std::fs::write("submissions.csv", csv)?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn export_submissions_csv(&self, form_id: &str) -> Result<Vec<u8>, PauboxError> {
        let auth = self.bearer_auth()?;
        let url = self
            .base_url
            .join(&format!("api/forms/{}/submissions/submission-csv", form_id))?;

        let resp = self
            .http
            .get(url)
            .header("Authorization", auth)
            .send()
            .await?;

        handle_bytes(resp).await
    }

    /// Export a single submission as raw CSV bytes.
    ///
    /// Requires an API key with the `"forms"` scope.
    ///
    /// # Errors
    /// Same variants as [`FormsClient::list_forms`]; a missing submission is
    /// [`PauboxError::Http`] with status 404.
    ///
    /// # Example
    /// ```no_run
    /// use paubox::forms::FormsClient;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = FormsClient::with_api_key("key-with-forms-scope");
    /// let csv = client
    ///     .export_submission_csv("form-uuid", "submission-uuid")
    ///     .await?;
    /// std::fs::write("submission.csv", csv)?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn export_submission_csv(
        &self,
        form_id: &str,
        submission_id: &str,
    ) -> Result<Vec<u8>, PauboxError> {
        let auth = self.bearer_auth()?;
        let url = self.base_url.join(&format!(
            "api/forms/{}/submissions/submission-csv/{}",
            form_id, submission_id
        ))?;

        let resp = self
            .http
            .get(url)
            .header("Authorization", auth)
            .send()
            .await?;

        handle_bytes(resp).await
    }

    /// Export a single submission as raw PDF bytes.
    ///
    /// Requires an API key with the `"forms"` scope.
    ///
    /// # Errors
    /// Same variants as [`FormsClient::list_forms`]; a missing form or
    /// submission surfaces as [`PauboxError::Http`] (the server currently
    /// reports it with status 500).
    ///
    /// # Example
    /// ```no_run
    /// use paubox::forms::FormsClient;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = FormsClient::with_api_key("key-with-forms-scope");
    /// let pdf = client
    ///     .export_submission_pdf("form-uuid", "submission-uuid")
    ///     .await?;
    /// std::fs::write("submission.pdf", pdf)?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn export_submission_pdf(
        &self,
        form_id: &str,
        submission_id: &str,
    ) -> Result<Vec<u8>, PauboxError> {
        let auth = self.bearer_auth()?;
        let url = self.base_url.join(&format!(
            "api/forms/{}/submissions/{}/submission-pdf",
            form_id, submission_id
        ))?;

        let resp = self
            .http
            .get(url)
            .header("Authorization", auth)
            .send()
            .await?;

        handle_bytes(resp).await
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn base() -> Url {
        Url::parse("https://apx.paubox.com/forms/").expect("valid URL")
    }

    // --- CreateForm builder validation -----------------------------------

    #[test]
    fn create_form_builder_happy_path_defaults_version_to_1() {
        let form = CreateForm::builder()
            .title("Intake")
            .customer_id(42)
            .form_json(json!({"fields": []}))
            .build()
            .expect("required fields set");
        let wire = serde_json::to_value(&form).expect("serializes");
        assert_eq!(wire["title"], "Intake");
        assert_eq!(wire["customer_id"], 42);
        assert_eq!(wire["version"], 1);
        assert_eq!(wire["form_json"], json!({"fields": []}));
    }

    #[test]
    fn create_form_builder_missing_title_fails() {
        let err = CreateForm::builder()
            .customer_id(42)
            .form_json(json!({}))
            .build()
            .expect_err("title missing");
        assert!(matches!(err, PauboxError::Validation(msg) if msg.contains("title")));
    }

    #[test]
    fn create_form_builder_missing_customer_id_fails() {
        let err = CreateForm::builder()
            .title("Intake")
            .form_json(json!({}))
            .build()
            .expect_err("customer_id missing");
        assert!(matches!(err, PauboxError::Validation(msg) if msg.contains("customer_id")));
    }

    #[test]
    fn create_form_builder_missing_form_json_fails() {
        let err = CreateForm::builder()
            .title("Intake")
            .customer_id(42)
            .build()
            .expect_err("form_json missing");
        assert!(matches!(err, PauboxError::Validation(msg) if msg.contains("form_json")));
    }

    #[test]
    fn create_form_serializes_optional_fields_and_renames_type() {
        let form = CreateForm::builder()
            .title("Intake")
            .customer_id(42)
            .form_json(json!({}))
            .version(3)
            .type_("marketing_form")
            .signable(true)
            .recipient("a@x.com,b@x.com")
            .build()
            .expect("required fields set");
        let wire = serde_json::to_value(&form).expect("serializes");
        assert_eq!(wire["version"], 3);
        assert_eq!(wire["type"], "marketing_form");
        assert_eq!(wire["signable"], true);
        assert_eq!(wire["recipient"], "a@x.com,b@x.com");
        // Unset optionals must be omitted entirely.
        assert!(wire.get("description").is_none());
        assert!(wire.get("form_html").is_none());
        assert!(wire.get("active").is_none());
    }

    // --- UpdateForm serialization -----------------------------------------

    #[test]
    fn update_form_default_serializes_to_empty_object() {
        let wire = serde_json::to_value(UpdateForm::default()).expect("serializes");
        assert_eq!(wire, json!({}));
    }

    #[test]
    fn update_form_only_set_fields_are_serialized() {
        let update = UpdateForm::default().title("Renamed").active(false);
        let wire = serde_json::to_value(&update).expect("serializes");
        assert_eq!(wire, json!({"title": "Renamed", "active": false}));
    }

    // --- Query parameter construction --------------------------------------

    #[test]
    fn form_list_params_default_adds_no_query() {
        let mut url = base().join("api/forms").expect("join");
        FormListParams::default().apply(&mut url);
        assert_eq!(url.query(), None);
    }

    #[test]
    fn form_list_params_all_fields_appear_in_query() {
        let mut url = base().join("api/forms").expect("join");
        FormListParams::default()
            .customer_id(7)
            .form_id("f-1")
            .search("intake")
            .order("asc")
            .order_by("title")
            .archived(false)
            .active(true)
            .page(2)
            .items(25)
            .apply(&mut url);
        let query = url.query().expect("query set");
        assert_eq!(
            query,
            "customer_id=7&form_id=f-1&search=intake&order=asc&order_by=title\
             &archived=false&active=true&page=2&items=25"
        );
    }

    #[test]
    fn submission_list_params_all_fields_appear_in_query() {
        let mut url = base().join("api/forms/f-1/submissions").expect("join");
        SubmissionListParams::default()
            .submission_id("s-1")
            .order("desc")
            .order_by("submitter_email")
            .page(3)
            .items(100)
            .apply(&mut url);
        assert_eq!(
            url.query().expect("query set"),
            "submission_id=s-1&order=desc&order_by=submitter_email&page=3&items=100"
        );
    }

    #[test]
    fn query_values_are_percent_encoded() {
        let mut url = base().join("api/forms").expect("join");
        FormListParams::default().search("a b&c").apply(&mut url);
        assert_eq!(url.query().expect("query set"), "search=a+b%26c");
    }

    // --- Submission deserialization and form_data_json ---------------------

    #[test]
    fn submission_deserializes_and_parses_form_data() {
        let sub: Submission = serde_json::from_value(json!({
            "id": "s-1",
            "form_id": "f-1",
            "form_data": "{\"first_name\":\"Jane\"}",
            "storage_type": "database",
            "storage_url": null,
            "submitter_email": "jane@example.com",
            "recipients": null,
            "attachment": null,
            "attachment_name": null,
            "attachment_url": null,
            "attachment_type": null,
            "created_at": "2026-01-01T00:00:00Z"
        }))
        .expect("deserializes");
        assert_eq!(sub.submitter_email.as_deref(), Some("jane@example.com"));
        let parsed = sub.form_data_json().expect("valid JSON");
        assert_eq!(parsed["first_name"], "Jane");
    }

    #[test]
    fn submission_form_data_json_rejects_invalid_json() {
        let sub: Submission = serde_json::from_value(json!({
            "id": "s-1",
            "form_id": "f-1",
            "form_data": "not json",
            "storage_type": "database",
            "created_at": "2026-01-01T00:00:00Z"
        }))
        .expect("deserializes");
        assert!(matches!(
            sub.form_data_json(),
            Err(PauboxError::Deserialize(_))
        ));
    }

    // --- Form deserialization with the new defaulted fields ----------------

    #[test]
    fn form_parses_legacy_payload_without_new_fields() {
        // Fixture matching the pre-scoped-API-key wire shape: none of the
        // newly added fields present.
        let form: Form = serde_json::from_value(json!({
            "id": "f-1",
            "title": "Intake",
            "description": null,
            "form_html": null,
            "form_json": null,
            "form_css": null,
            "active": true,
            "signable": false,
            "submission_count": 0,
            "customer_id": 42,
            "created_at": "2026-01-01T00:00:00Z",
            "updated_at": "2026-01-01T00:00:00Z"
        }))
        .expect("legacy payload still parses");
        assert_eq!(form.version, 0);
        assert!(!form.archived);
        assert!(!form.deleted);
        assert!(form.type_.is_none());
    }

    #[test]
    fn form_parses_full_server_payload() {
        let form: Form = serde_json::from_value(json!({
            "id": "f-1",
            "title": "Intake",
            "description": "desc",
            "form_html": null,
            "form_json": {"fields": []},
            "form_css": null,
            "vanity_url": "my-intake",
            "version": 2,
            "active": true,
            "customer_id": 42,
            "old_form_id": 99,
            "created_at": "2026-01-01T00:00:00Z",
            "updated_at": "2026-01-02T00:00:00Z",
            "recipient": "a@x.com",
            "signable": true,
            "signature_confirmation_label": "I agree",
            "submission_count": 5,
            "type": "marketing_form",
            "subscription_list_id": "list-1",
            "deleted": false,
            "archived": true
        }))
        .expect("full payload parses");
        assert_eq!(form.version, 2);
        assert_eq!(form.vanity_url.as_deref(), Some("my-intake"));
        assert_eq!(form.type_.as_deref(), Some("marketing_form"));
        assert_eq!(form.subscription_list_id.as_deref(), Some("list-1"));
        assert_eq!(form.old_form_id, Some(99));
        assert!(form.archived);
        assert!(!form.deleted);
    }
}
