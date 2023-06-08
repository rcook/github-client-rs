use thiserror::Error;

#[derive(Debug, Error)]
pub enum GitHubClientError {
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
