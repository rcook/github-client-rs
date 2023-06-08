use crate::object_model::Repo;
use anyhow::{anyhow, Error};
use futures_util::future::try_join_all;
use reqwest::header::{ACCEPT, USER_AGENT};
use reqwest::{Client, IntoUrl, Response, Url};
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::result::Result as StdResult;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GitHubClientError {
    #[error(transparent)]
    Other(#[from] Error),
}

type GitHubClientResult<T> = StdResult<T, GitHubClientError>;

pub struct GitHubClient {
    url: Url,
    token: String,
}

impl GitHubClient {
    pub fn new<U>(url: U, token: &str) -> GitHubClientResult<Self>
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

    pub async fn list_repos(&self) -> GitHubClientResult<Vec<Repo>> {
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

    async fn get_paged<T>(&self, url: Url) -> GitHubClientResult<Vec<T>>
    where
        T: DeserializeOwned + Send + 'static,
    {
        async fn get_items<T>(
            client: &Client,
            token: &str,
            url: &Url,
            page_number: Option<usize>,
        ) -> GitHubClientResult<(Vec<T>, Option<LinkUrls>)>
        where
            T: DeserializeOwned,
        {
            let mut request_builder = client.get(url.clone());
            if let Some(x) = page_number {
                request_builder = request_builder.query(&[("page", x)]);
            }

            let response = request_builder
                .header(USER_AGENT, "github-client")
                .header(ACCEPT, "application/vnd.github+json")
                .header("X-GitHub-Api-Version", "2022-11-28")
                .bearer_auth(String::from(token))
                .send()
                .await
                .map_err(|e| GitHubClientError::Other(anyhow!(e)))?
                .error_for_status()
                .map_err(|e| GitHubClientError::Other(anyhow!(e)))?;

            let link_urls = LinkUrls::from_response(&response)?;

            Ok((
                response
                    .json::<Vec<T>>()
                    .await
                    .map_err(|e| GitHubClientError::Other(anyhow!(e)))?,
                link_urls,
            ))
        }

        fn get_page_number(url: &Url) -> GitHubClientResult<usize> {
            Ok(url
                .query_pairs()
                .find(|(n, _)| n == "page")
                .ok_or_else(|| anyhow!("page missing from query string"))?
                .1
                .parse::<usize>()
                .map_err(|e| GitHubClientError::Other(anyhow!(e)))?)
        }

        let client = Client::new();
        let mut all_items = Vec::new();

        let (items, link_urls) = get_items::<T>(&client, &self.token, &url, None).await?;
        all_items.extend(items);

        let Some(link_urls) = link_urls else {
            return Ok(all_items)
        };

        let next_page_number = get_page_number(&link_urls.next_url)?;
        let last_page_number = get_page_number(&link_urls.last_url)?;
        for (items, _) in try_join_all(
            (next_page_number..=last_page_number)
                .map(|i| get_items::<T>(&client, &self.token, &url, Some(i))),
        )
        .await?
        {
            all_items.extend(items);
        }

        Ok(all_items)
    }
}

#[derive(Debug)]
struct LinkUrls {
    next_url: Url,
    #[allow(unused)]
    last_url: Url,
}

impl LinkUrls {
    fn from_response(response: &Response) -> GitHubClientResult<Option<LinkUrls>> {
        let Some(link_header) = response.headers().get("link") else {
            return Ok(None)
        };

        let links = Self::parse_link_header(
            link_header
                .to_str()
                .map_err(|e| GitHubClientError::Other(anyhow!(e)))?,
        );

        let Some(next_url) = Self::get_link_url(&links, "next")? else {
            return Ok(None)
        };

        let Some(last_url) = Self::get_link_url(&links, "last")? else {
            return Ok(None)
        };

        Ok(Some(LinkUrls { next_url, last_url }))
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

    fn get_link_url(links: &HashMap<String, String>, k: &str) -> GitHubClientResult<Option<Url>> {
        let Some(s) = links.get(k) else {
            return Ok(None)
        };

        Ok(Some(
            s.parse::<Url>()
                .map_err(|e| GitHubClientError::Other(anyhow!(e)))?,
        ))
    }
}
