use crate::error::GitHubClientError;
use crate::result::GitHubClientResult;
use anyhow::anyhow;
use reqwest::{Response, Url};
use std::collections::HashMap;

#[derive(Debug)]
pub(crate) struct LinkUrls {
    pub(crate) next_url: Url,
    pub(crate) last_url: Url,
}

impl LinkUrls {
    pub(crate) fn from_response(response: &Response) -> GitHubClientResult<Option<LinkUrls>> {
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
