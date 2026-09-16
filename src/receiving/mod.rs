pub mod types;

pub use types::{
    AttachmentMeta, DnsRecord, EmailAddress, Mailbox, ReceivedEmail, ReceivedEmailList,
    ReceivingDomain,
};

use serde::Deserialize;

use crate::client::PauboxClient;
use crate::error::PauboxError;

use types::{CreateDomainRequest, CreateMailboxRequest};

#[derive(Deserialize)]
struct DataWrapper<T> {
    data: T,
}

impl PauboxClient {
    pub async fn list_receiving_domains(&self) -> Result<Vec<ReceivingDomain>, PauboxError> {
        let url = self.base_url.join("receiving/domains")?;

        let resp = self
            .http
            .get(url)
            .header("Authorization", self.auth_header())
            .header("Accept", "application/json")
            .send()
            .await?;

        let wrapper = handle_response::<DataWrapper<Vec<ReceivingDomain>>>(resp).await?;
        Ok(wrapper.data)
    }

    pub async fn create_receiving_domain(
        &self,
        slug: Option<&str>,
    ) -> Result<ReceivingDomain, PauboxError> {
        let url = self.base_url.join("receiving/domains")?;
        let body = CreateDomainRequest {
            slug: slug.map(String::from),
        };

        let resp = self
            .http
            .post(url)
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        let wrapper = handle_response::<DataWrapper<ReceivingDomain>>(resp).await?;
        Ok(wrapper.data)
    }

    pub async fn get_receiving_domain(
        &self,
        domain_id: i64,
    ) -> Result<ReceivingDomain, PauboxError> {
        let url = self
            .base_url
            .join(&format!("receiving/domains/{domain_id}"))?;

        let resp = self
            .http
            .get(url)
            .header("Authorization", self.auth_header())
            .header("Accept", "application/json")
            .send()
            .await?;

        let wrapper = handle_response::<DataWrapper<ReceivingDomain>>(resp).await?;
        Ok(wrapper.data)
    }

    pub async fn delete_receiving_domain(&self, domain_id: i64) -> Result<(), PauboxError> {
        let url = self
            .base_url
            .join(&format!("receiving/domains/{domain_id}"))?;

        let resp = self
            .http
            .delete(url)
            .header("Authorization", self.auth_header())
            .send()
            .await?;

        handle_empty_response(resp).await
    }

    pub async fn list_receiving_mailboxes(
        &self,
        domain_id: i64,
    ) -> Result<Vec<Mailbox>, PauboxError> {
        let url = self
            .base_url
            .join(&format!("receiving/domains/{domain_id}/mailboxes"))?;

        let resp = self
            .http
            .get(url)
            .header("Authorization", self.auth_header())
            .header("Accept", "application/json")
            .send()
            .await?;

        let wrapper = handle_response::<DataWrapper<Vec<Mailbox>>>(resp).await?;
        Ok(wrapper.data)
    }

    pub async fn create_receiving_mailbox(
        &self,
        domain_id: i64,
        name: &str,
        password: &str,
        quota_bytes: Option<i64>,
    ) -> Result<Mailbox, PauboxError> {
        let url = self
            .base_url
            .join(&format!("receiving/domains/{domain_id}/mailboxes"))?;
        let body = CreateMailboxRequest {
            name: name.to_owned(),
            password: password.to_owned(),
            quota_bytes,
        };

        let resp = self
            .http
            .post(url)
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        let wrapper = handle_response::<DataWrapper<Mailbox>>(resp).await?;
        Ok(wrapper.data)
    }

    pub async fn get_receiving_mailbox(
        &self,
        domain_id: i64,
        mailbox_id: i64,
    ) -> Result<Mailbox, PauboxError> {
        let url = self.base_url.join(&format!(
            "receiving/domains/{domain_id}/mailboxes/{mailbox_id}"
        ))?;

        let resp = self
            .http
            .get(url)
            .header("Authorization", self.auth_header())
            .header("Accept", "application/json")
            .send()
            .await?;

        let wrapper = handle_response::<DataWrapper<Mailbox>>(resp).await?;
        Ok(wrapper.data)
    }

    pub async fn delete_receiving_mailbox(
        &self,
        domain_id: i64,
        mailbox_id: i64,
    ) -> Result<(), PauboxError> {
        let url = self.base_url.join(&format!(
            "receiving/domains/{domain_id}/mailboxes/{mailbox_id}"
        ))?;

        let resp = self
            .http
            .delete(url)
            .header("Authorization", self.auth_header())
            .send()
            .await?;

        handle_empty_response(resp).await
    }

    pub async fn list_received_emails(
        &self,
        limit: Option<u32>,
        after: Option<&str>,
        before: Option<&str>,
    ) -> Result<ReceivedEmailList, PauboxError> {
        let mut url = self.base_url.join("receiving")?;

        if limit.is_some() || after.is_some() || before.is_some() {
            let mut pairs = url.query_pairs_mut();
            if let Some(l) = limit {
                pairs.append_pair("limit", &l.to_string());
            }
            if let Some(a) = after {
                pairs.append_pair("after", a);
            }
            if let Some(b) = before {
                pairs.append_pair("before", b);
            }
        }

        let resp = self
            .http
            .get(url)
            .header("Authorization", self.auth_header())
            .header("Accept", "application/json")
            .send()
            .await?;

        handle_response::<ReceivedEmailList>(resp).await
    }

    pub async fn get_received_email(&self, email_id: &str) -> Result<ReceivedEmail, PauboxError> {
        let url = self.base_url.join(&format!("receiving/{email_id}"))?;

        let resp = self
            .http
            .get(url)
            .header("Authorization", self.auth_header())
            .header("Accept", "application/json")
            .send()
            .await?;

        let wrapper = handle_response::<DataWrapper<ReceivedEmail>>(resp).await?;
        Ok(wrapper.data)
    }

    pub async fn get_received_email_attachment(
        &self,
        email_id: &str,
        blob_id: &str,
    ) -> Result<Vec<u8>, PauboxError> {
        let url = self
            .base_url
            .join(&format!("receiving/{email_id}/attachments/{blob_id}"))?;

        let resp = self
            .http
            .get(url)
            .header("Authorization", self.auth_header())
            .send()
            .await?;

        let status = resp.status();
        if status.is_success() {
            let bytes = resp.bytes().await?;
            Ok(bytes.to_vec())
        } else {
            let status_u16 = status.as_u16();
            let body = resp.text().await.unwrap_or_default();
            if status_u16 == 401 {
                Err(PauboxError::Auth(body))
            } else {
                Err(PauboxError::Http {
                    status: status_u16,
                    body,
                })
            }
        }
    }
}

async fn handle_response<T>(resp: reqwest::Response) -> Result<T, PauboxError>
where
    T: serde::de::DeserializeOwned,
{
    let status = resp.status();
    if status.is_success() {
        let text = resp.text().await?;
        let parsed = serde_json::from_str::<T>(&text)?;
        Ok(parsed)
    } else {
        let status_u16 = status.as_u16();
        let body = resp.text().await.unwrap_or_default();
        if status_u16 == 401 {
            Err(PauboxError::Auth(body))
        } else {
            Err(PauboxError::Http {
                status: status_u16,
                body,
            })
        }
    }
}

async fn handle_empty_response(resp: reqwest::Response) -> Result<(), PauboxError> {
    let status = resp.status();
    if status.is_success() {
        Ok(())
    } else {
        let status_u16 = status.as_u16();
        let body = resp.text().await.unwrap_or_default();
        if status_u16 == 401 {
            Err(PauboxError::Auth(body))
        } else {
            Err(PauboxError::Http {
                status: status_u16,
                body,
            })
        }
    }
}
