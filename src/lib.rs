use std::fmt::Display;

use serde_derive::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;

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

    #[cfg(feature = "csv")]
    #[error("{0}")]
    CsvError(#[from] csv::Error),

    #[error("{0}")]
    UrlParsing(#[from] url::ParseError),
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum LinkAction {
    Redirect,
    Content,
}
impl Display for LinkAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            &LinkAction::Content => write!(f, "Content"),
            &LinkAction::Redirect => write!(f, "Redirect"),
        }
    }
}

pub struct LinkInfo {
    pub original: String,
    pub page: Option<String>,
    pub label: Option<String>,
    pub link: String,
    pub action: LinkAction,
}

impl LinkInfo {
    fn new(base: &Url, page: Option<String>, l: LinkMatchV2) -> Result<Self> {
        Ok(Self {
            original: l.link,
            page,
            label: l.link_label,
            action: l.action,
            link: base.join(&l.link_id)?.to_string(),
        })
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
    use url::Url;

    pub struct Client {
        #[allow(unused)]
        host: Url,
        api_endpoint: String,
        user: String,
        agent: Agent,
    }
    impl Client {
        pub fn connect(user_id: &str) -> Result<Client> {
            Self::connect_host("https://urlfreezer.com", user_id)
        }
        pub fn connect_host(host: &str, user_id: &str) -> Result<Client> {
            let agent: Agent = ureq::AgentBuilder::new().build();
            let host = Url::parse(host)?;
            let api_path = host.clone().join("/api/fetch_links_v2")?.to_string();
            Ok(Client {
                host,
                api_endpoint: api_path,
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
                .post(&self.api_endpoint)
                .send_json(ureq::json!(message))?
                .into_json()?;
            let base = value.base.parse()?;
            let p = page.map(|x| x.to_owned());
            Ok(value
                .links
                .into_iter()
                .map(|l| LinkInfo::new(&base, p.clone(), l))
                .collect::<Result<Vec<LinkInfo>>>()?)
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

        #[cfg(feature = "csv")]
        pub fn fetch_with_csv(
            &self,
            mut data: csv::Reader<impl std::io::Read>,
            mut out: csv::Writer<impl std::io::Write>,
        ) -> Result<()> {
            use super::CsvInRow;
            use super::CsvOutRow;
            for rec in data.deserialize::<CsvInRow>() {
                if let Ok(csv_row) = rec {
                    let page = if csv_row.page.is_empty() {
                        None
                    } else {
                        Some(csv_row.page.as_str())
                    };
                    let label = if csv_row.label.is_empty() {
                        None
                    } else {
                        Some(csv_row.label.as_str())
                    };
                    let res = self.fetch_link(&csv_row.link, page, label)?;
                    if let Some(r) = res {
                        out.serialize(CsvOutRow::from((csv_row, r)))?;
                    }
                }
            }
            Ok(())
        }
    }
}

#[cfg(feature = "async")]
pub mod non_blocking {
    use crate::{FetchLinksV2, FetchedLinksV2, LinkInfo, LinkToFetch, Result};
    use surf::Client as Surf;
    use url::Url;

    pub struct Client {
        #[allow(unused)]
        host: Url,
        api_endpoint: String,
        user: String,
        client: Surf,
    }
    impl Client {
        pub async fn connect(user_id: &str) -> Result<Self> {
            Self::connect_host("https://urlfreezer.com", user_id).await
        }
        pub async fn connect_host(host: &str, user_id: &str) -> Result<Self> {
            let host = Url::parse(host)?;
            let dest = host.clone().join("/api/fetch_links_v2")?.to_string();
            Ok(Self {
                host,
                api_endpoint: dest,
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
            let mut resp = self
                .client
                .post(&self.api_endpoint)
                .body_json(&message)?
                .await?;
            let value: FetchedLinksV2 = resp.body_json().await?;
            let base = value.base.parse()?;
            let p = page.map(|x| x.to_owned());
            Ok(value
                .links
                .into_iter()
                .map(|l| LinkInfo::new(&base, p.clone(), l))
                .collect::<Result<Vec<LinkInfo>>>()?)
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

        #[cfg(feature = "csv")]
        pub async fn fetch_with_csv(
            &self,
            mut data: csv::Reader<impl std::io::Read>,
            mut out: csv::Writer<impl std::io::Write>,
        ) -> Result<()> {
            use super::CsvInRow;
            use super::CsvOutRow;
            for rec in data.deserialize::<CsvInRow>() {
                if let Ok(csv_row) = rec {
                    let page = if csv_row.page.is_empty() {
                        None
                    } else {
                        Some(csv_row.page.as_str())
                    };
                    let label = if csv_row.label.is_empty() {
                        None
                    } else {
                        Some(csv_row.label.as_str())
                    };
                    let res = self.fetch_link(&csv_row.link, page, label).await?;
                    if let Some(r) = res {
                        out.serialize(CsvOutRow::from((csv_row, r)))?;
                    }
                }
            }
            Ok(())
        }
    }
    impl From<surf::Error> for crate::Error {
        fn from(e: surf::Error) -> Self {
            crate::Error::NonBlockingClient(e.to_string())
        }
    }
}

#[cfg(feature = "csv")]
#[derive(Deserialize, Debug)]
struct CsvInRow {
    page: String,
    link: String,
    label: String,
}

#[cfg(feature = "csv")]
#[derive(Serialize)]
struct CsvOutRow {
    page: String,
    original: String,
    label: String,
    link: String,
    action: String,
}

#[cfg(feature = "csv")]
impl From<(CsvInRow, LinkInfo)> for CsvOutRow {
    fn from((r, i): (CsvInRow, LinkInfo)) -> Self {
        CsvOutRow {
            page: r.page,
            original: r.link,
            label: r.label,
            link: i.link,
            action: format!("{}", i.action),
        }
    }
}
