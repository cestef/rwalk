use color_eyre::eyre::{Context, Result};
use lazy_static::lazy_static;
use scraper::{Html, Selector};
use std::fmt;
use url::Url;
lazy_static! {
    static ref ABSOLUTE_URL_REGEX: regex::Regex = regex::Regex::new(r"(https?:\/\/(www\.)?[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}\b([-a-zA-Z0-9()@:%_\+.~#?&//=]*))").unwrap();
    static ref RELATIVE_URL_REGEX: regex::Regex = regex::Regex::new(r"^/.*").unwrap();
}

const ATTRIBUTES: [&str; 4] = ["href", "src", "data-src", "content"];

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum DocumentType {
    Html,
    PlainText,
}

pub struct Document {
    pub base: Url,
    pub body: String,
    pub document_type: DocumentType,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LinkType {
    Internal,
    External,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Link {
    pub url: Url,
    pub link_type: LinkType,
}
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct GraphNode {
    pub url: Url,
    pub document_type: DocumentType,
}
impl GraphNode {
    pub fn new(url: Url, document_type: DocumentType) -> Self {
        Self { url, document_type }
    }
}

impl fmt::Display for GraphNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({:?})", self.url, self.document_type)
    }
}

impl Link {
    pub fn new(url: Url, link_type: LinkType) -> Self {
        Self { url, link_type }
    }
}

impl fmt::Display for Link {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.link_type {
            LinkType::Internal => write!(f, "Internal: {}", self.url),
            LinkType::External => write!(f, "External: {}", self.url),
        }
    }
}

pub fn is_same_domain(url: &Url, base: &Url, allow_subdomain: bool) -> Result<bool> {
    let url_domain = url.domain().ok_or_else(|| {
        color_eyre::eyre::eyre!("Could not parse domain from URL: {}", url.to_string())
    })?;

    let base_domain = base.domain().ok_or_else(|| {
        color_eyre::eyre::eyre!("Could not parse domain from URL: {}", base.to_string())
    })?;

    if allow_subdomain {
        Ok(url_domain == base_domain || url_domain.ends_with(&format!(".{}", base_domain)))
    } else {
        Ok(url_domain == base_domain)
    }
}

impl Document {
    pub fn parse(base: &Url, body: &str) -> Self {
        let document_type = if body.trim().starts_with("<!DOCTYPE html>") {
            DocumentType::Html
        } else {
            DocumentType::PlainText
        };

        Self {
            base: base.clone(),
            body: body.to_string(),
            document_type,
        }
    }

    pub fn links(&self, allow_subdomain: bool) -> Result<Vec<Link>> {
        match self.document_type {
            DocumentType::Html => {
                let html = Html::parse_document(&self.body);

                let mut links = Vec::new();

                for attribute in ATTRIBUTES.iter() {
                    for element in
                        html.select(&Selector::parse(&format!("[{}]", attribute)).unwrap())
                    {
                        let value = element.value().attr(attribute).unwrap_or_default();

                        let maybe_absolute_url = ABSOLUTE_URL_REGEX.find(value);
                        let maybe_relative_url = RELATIVE_URL_REGEX.find(value);

                        let link = match (maybe_absolute_url, maybe_relative_url) {
                            (Some(absolute_url), _) => {
                                let url = Url::parse(absolute_url.as_str()).context(format!(
                                    "Could not parse URL: {}",
                                    absolute_url.as_str()
                                ))?;
                                if is_same_domain(&url, &self.base, allow_subdomain)? {
                                    Link::new(url, LinkType::Internal)
                                } else {
                                    Link::new(url, LinkType::External)
                                }
                            }
                            (_, Some(relative_url)) => {
                                let url = self.base.join(relative_url.as_str())?;
                                if is_same_domain(&url, &self.base, allow_subdomain)? {
                                    Link::new(url, LinkType::Internal)
                                } else {
                                    Link::new(url, LinkType::External)
                                }
                            }
                            _ => continue,
                        };

                        links.push(link);
                    }
                }

                // Remove duplicates
                links.sort_unstable();
                links.dedup();

                Ok(links)
            }
            DocumentType::PlainText => {
                // Match links in plain text
                let mut links = Vec::new();

                for url in ABSOLUTE_URL_REGEX.find_iter(&self.body) {
                    let url = Url::parse(url.as_str())?;
                    if is_same_domain(&url, &self.base, allow_subdomain)? {
                        links.push(Link::new(url, LinkType::Internal));
                    } else {
                        links.push(Link::new(url, LinkType::External));
                    }
                }

                links.sort_unstable();
                links.dedup();

                Ok(links)
            }
        }
    }
}
