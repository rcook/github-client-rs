use crate::error::GitHubClientError;

pub type GitHubClientResult<T> = std::result::Result<T, GitHubClientError>;
