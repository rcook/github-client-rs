use crate::object_model::Repo;
use anyhow::{anyhow, Error};
use reqwest::header::{ACCEPT, USER_AGENT};
use reqwest::Client;
use reqwest::{IntoUrl, Url};
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::result::Result as StdResult;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GitHubClientError {
    #[error(transparent)]
    Other(#[from] Error),
}

pub struct GitHubClient {
    url: Url,
    token: String,
}

impl GitHubClient {
    pub fn new<U>(url: U, token: &str) -> StdResult<Self, GitHubClientError>
    where
        U: IntoUrl,
    {
        Ok(Self {
            url: url
                .into_url()
                .map_err(|e| GitHubClientError::Other(anyhow!(e)))?,
            token: String::from(token),
        })
    }

    pub async fn list_repos(&self) -> StdResult<Vec<Repo>, GitHubClientError> {
        let mut repos = self
            .get_paged::<Repo>(
                self.url
                    .join("/user/repos")
                    .map_err(|e| GitHubClientError::Other(anyhow!(e)))?,
            )
            .await?;
        repos.sort_by(|a, b| a.full_name.cmp(&b.full_name));
        Ok(repos)
    }

    async fn get_paged<T>(&self, mut url: Url) -> StdResult<Vec<T>, GitHubClientError>
    where
        T: DeserializeOwned,
    {
        let mut all_items = Vec::new();
        loop {
            let client = Client::new();
            let response = client
                .get(url)
                .header(USER_AGENT, "github-client")
                .header(ACCEPT, "application/vnd.github+json")
                .header("X-GitHub-Api-Version", "2022-11-28")
                .bearer_auth(self.token.clone())
                .send()
                .await
                .map_err(|e| GitHubClientError::Other(anyhow!(e)))?
                .error_for_status()
                .map_err(|e| GitHubClientError::Other(anyhow!(e)))?;

            let next_link_url = if let Some(link_header) = response.headers().get("link") {
                let link_header_str = link_header
                    .to_str()
                    .map_err(|e| GitHubClientError::Other(anyhow!(e)))?;
                let links = Self::parse_link_header(link_header_str);
                if let Some(next_link) = links.get("next") {
                    Some(
                        next_link
                            .parse::<Url>()
                            .map_err(|e| GitHubClientError::Other(anyhow!(e)))?,
                    )
                } else {
                    None
                }
            } else {
                None
            };

            let items = response
                .json::<Vec<T>>()
                .await
                .map_err(|e| GitHubClientError::Other(anyhow!(e)))?;
            all_items.extend(items);

            if let Some(u) = next_link_url {
                url = u;
            } else {
                break;
            }
        }

        Ok(all_items)
    }

    fn parse_link_header(s: &str) -> HashMap<String, String> {
        fn parse_url_part(s: &str) -> Option<String> {
            s.strip_prefix('<')
                .and_then(|s0| s0.strip_suffix('>'))
                .map(|s1| s1.to_string())
        }

        fn parse_rel_part(s: &str) -> Option<String> {
            s.strip_prefix("rel=\"")
                .and_then(|s0| s0.strip_suffix('"'))
                .map(|s1| s1.to_string())
        }

        s.split(',')
            .filter_map(|part| {
                part.split_once(';').and_then(|(u, r)| {
                    parse_url_part(u.trim())
                        .and_then(|u0| parse_rel_part(r.trim()).map(|r0| (r0, u0)))
                })
            })
            .collect::<HashMap<_, _>>()
    }
}
