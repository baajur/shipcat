use shipcat_definitions::{ShipcatManifest};
use reqwest::{Client, Url};


/// StatusCake api client
pub struct Cake {
    client: Client,
    addr: Url,
    token: String,
    user: String,
}
impl Cake {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            client: Client::new(),
            addr: Url::parse(&std::env::var("STATUSCAKE_BASE_URL")
                .unwrap_or("https://app.statuscake.com/API/Tests/".into()))?,
            token: std::env::var("STATUSCAKE_API_KEY").expect("need STATUSCAKE_API_KEY"),
            user: std::env::var("STATUSCAKE_USER").expect("need STATUSCAKE_USER"),
        })
    }
}
