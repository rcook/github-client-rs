use serde::Deserialize;

#[derive(Deserialize)]
pub struct Owner {
    #[serde(rename = "login")]
    pub login: String,
}
