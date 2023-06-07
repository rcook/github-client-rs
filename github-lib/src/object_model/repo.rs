use super::owner::Owner;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Repo {
    #[serde(rename = "id")]
    pub id: i32,

    #[serde(rename = "name")]
    pub name: String,

    #[serde(rename = "full_name")]
    pub full_name: String,

    #[serde(rename = "private")]
    pub private: bool,

    #[serde(rename = "html_url")]
    pub html_url: String,

    #[serde(rename = "owner")]
    pub owner: Owner,
}
