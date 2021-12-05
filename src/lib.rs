use serde_derive::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[cfg(feature = "blocking")]
    #[error("{0}")]
    BlockingClient(#[from] ureq::Error),

    #[cfg(feature = "async")]
    #[error("{0}")]
    NonBlockingClient(String),

    #[error("{0}")]
    Io(#[from] std::io::Error),
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum LinkAction {
    Redirect,
    Content,
}

pub struct LinkInfo {
    pub original: String,
    pub page: Option<String>,
    pub label: Option<String>,
    pub link: String,
    pub action: LinkAction,
}

impl LinkInfo {
    fn new(base: String, page: Option<String>, l: LinkMatchV2) -> Self {
        Self {
            original: l.link,
            page,
            label: l.link_label,
            action: l.action,
            link: format!("{}/{}", base, l.link_id),
        }
    }
}

#[derive(Deserialize)]
struct FetchedLinksV2 {
    links: Vec<LinkMatchV2>,
    base: String,
}

#[derive(Deserialize)]
struct LinkMatchV2 {
    link: String,
    link_label: Option<String>,
    link_id: String,
    action: LinkAction,
}

pub struct LinkToFetch {
    link: String,
    label: Option<String>,
}
impl LinkToFetch {
    pub fn new(link: &str, label: Option<&str>) -> LinkToFetch {
        LinkToFetch {
            link: link.to_owned(),
            label: label.map(|l| l.to_owned()),
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct FetchLinkData {
    link: String,
    link_label: Option<String>,
}
impl From<&LinkToFetch> for FetchLinkData {
    fn from(l: &LinkToFetch) -> Self {
        Self {
            link: l.link.clone(),
            link_label: l.label.clone(),
        }
    }
}

#[derive(Deserialize, Serialize)]
struct FetchLinksV2 {
    user: String,
    page: Option<String>,
    links: Vec<FetchLinkData>,
}
impl FetchLinksV2 {
    fn new(user: String, links: &[LinkToFetch], page: Option<String>) -> Self {
        Self {
            user,
            links: links.iter().map(From::from).collect(),
            page,
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(feature = "blocking")]
pub mod blocking {

    use crate::{FetchLinksV2, FetchedLinksV2, LinkInfo, LinkToFetch, Result};
    use ureq::Agent;

    pub struct Client {
        host: String,
        user: String,
        agent: Agent,
    }
    impl Client {
        pub fn connect(user_id: &str) -> Result<Client> {
            Self::connect_host("https://urlfreezer.com", user_id)
        }
        pub fn connect_host(host: &str, user_id: &str) -> Result<Client> {
            let agent: Agent = ureq::AgentBuilder::new().build();
            Ok(Client {
                host: host.to_owned(),
                user: user_id.to_owned(),
                agent,
            })
        }

        pub fn fetch_links(
            &self,
            links: &[LinkToFetch],
            page: Option<&str>,
        ) -> Result<Vec<LinkInfo>> {
            let message = FetchLinksV2::new(self.user.clone(), links, page.map(|s| s.to_owned()));
            let value: FetchedLinksV2 = self
                .agent
                .post(&format!("{}/api/fetch_links_v2", self.host))
                .send_json(ureq::json!(message))?
                .into_json()?;
            let base = value.base;
            let p = page.map(|x| x.to_owned());
            Ok(value
                .links
                .into_iter()
                .map(|l| LinkInfo::new(base.clone(), p.clone(), l))
                .collect())
        }
        pub fn fetch_link(
            &self,
            link: &str,
            page: Option<&str>,
            label: Option<&str>,
        ) -> Result<Option<LinkInfo>> {
            Ok(self
                .fetch_links(&[LinkToFetch::new(link, label)], page)?
                .into_iter()
                .next())
        }
    }
}

#[cfg(feature = "async")]
pub mod non_blocking {
    use crate::{FetchLinksV2, FetchedLinksV2, LinkInfo, LinkToFetch, Result};
    use surf::Client as Surf;
    pub struct Client {
        host: String,
        user: String,
        client: Surf,
    }
    impl Client {
        pub async fn connect(user_id: &str) -> Result<Self> {
            Self::connect_host("https://urlfreezer.com", user_id).await
        }
        pub async fn connect_host(host: &str, user_id: &str) -> Result<Self> {
            Ok(Self {
                host: host.to_owned(),
                user: user_id.to_owned(),
                client: Surf::new(),
            })
        }

        pub async fn fetch_links(
            &self,
            links: &[LinkToFetch],
            page: Option<&str>,
        ) -> Result<Vec<LinkInfo>> {
            let message = FetchLinksV2::new(self.user.clone(), links, page.map(|s| s.to_owned()));
            let dest = format!("{}/api/fetch_links_v2", self.host);
            let mut resp = self.client.post(&dest).body_json(&message)?.await?;
            let value: FetchedLinksV2 = resp.body_json().await?;
            let base = value.base;
            let p = page.map(|x| x.to_owned());
            Ok(value
                .links
                .into_iter()
                .map(|l| LinkInfo::new(base.clone(), p.clone(), l))
                .collect())
        }
        pub async fn fetch_link(
            &self,
            link: &str,
            page: Option<&str>,
            label: Option<&str>,
        ) -> Result<Option<LinkInfo>> {
            Ok(self
                .fetch_links(&[LinkToFetch::new(link, label)], page)
                .await?
                .into_iter()
                .next())
        }
    }
    impl From<surf::Error> for crate::Error {
        fn from(e: surf::Error) -> Self {
            crate::Error::NonBlockingClient(e.to_string())
        }
    }
}
