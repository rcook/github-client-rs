mod github_client;
mod object_model;

pub use self::github_client::{GitHubClient, GitHubClientError};
pub use self::object_model::{Owner, Repo};
