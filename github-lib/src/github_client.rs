use crate::error::GitHubClientError;
use crate::link_urls::LinkUrls;
use crate::object_model::Repo;
use crate::result::GitHubClientResult;
use anyhow::anyhow;
use futures_util::future::try_join_all;
use log::debug;
use reqwest::header::{ACCEPT, USER_AGENT};
use reqwest::{Client, IntoUrl, Url};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use serde::de::DeserializeOwned;

pub struct GitHubClient {
    client: ClientWithMiddleware,
    url: Url,
    token: String,
}

impl GitHubClient {
    pub fn new<U>(url: U, token: &str) -> GitHubClientResult<Self>
    where
        U: IntoUrl,
    {
        let client = ClientBuilder::new(Client::new()).build();
        Ok(Self {
            client,
            url: url
                .into_url()
                .map_err(|e| GitHubClientError::Other(anyhow!(e)))?,
            token: String::from(token),
        })
    }

    pub async fn get_user_repos(&self) -> GitHubClientResult<Vec<Repo>> {
        debug!("get_user_repos");
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
            client: &ClientWithMiddleware,
            token: &str,
            url: &Url,
            page_number: Option<usize>,
        ) -> GitHubClientResult<(Vec<T>, Option<LinkUrls>)>
        where
            T: DeserializeOwned,
        {
            debug!("get_items(page_number={:?})", page_number);
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

            debug!("get_items(page_number={:?}) returned", page_number);

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
            url.query_pairs()
                .find(|(n, _)| n == "page")
                .ok_or_else(|| anyhow!("page missing from query string"))?
                .1
                .parse::<usize>()
                .map_err(|e| GitHubClientError::Other(anyhow!(e)))
        }

        let mut all_items = Vec::new();

        let (items, link_urls) = get_items::<T>(&self.client, &self.token, &url, None).await?;
        all_items.extend(items);

        let Some(link_urls) = link_urls else {
            return Ok(all_items)
        };

        let next_page_number = get_page_number(&link_urls.next_url)?;
        let last_page_number = get_page_number(&link_urls.last_url)?;
        for (items, _) in try_join_all(
            (next_page_number..=last_page_number)
                .map(|i| get_items::<T>(&self.client, &self.token, &url, Some(i))),
        )
        .await?
        {
            all_items.extend(items);
        }

        Ok(all_items)
    }
}
