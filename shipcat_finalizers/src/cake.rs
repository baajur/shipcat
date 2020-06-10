use shipcat_definitions::{ShipcatManifest};
use serde::{Deserialize, Serialize};
use reqwest::{Client, Url};

/// Representation of the important fields from StatusCake's API
///
/// Taken from https://www.statuscake.com/api/Tests/Get%20All%20Tests.md
/// Leaving it with the weird case convention because CBA.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(non_snake_case)]
pub struct Test {
    /// The statuscake uid needed to perform updates/deletes
    #[serde(skip_serializing_if = "Option::is_none")]
    TestID: Option<u64>,
    /// Our name for the test
    ///
    /// By convention "shipcatregion servicename healthcheck"
    WebsiteName: String,

    /// The url statuscake should hit
    WebsiteURL: String,

    /// The type of test (HTTP,TCP,PING)
    TestType: String,

    // optionals

    /// The configured contact group on the api
    ///
    /// Corresponds to the id on https://app.statuscake.com/CurrentGroups.php?View=41161
    /// NOTE THAT THE API EXPECTS STRING, AND RETURNS VECTOR OF STRING
    #[serde(default)]
    ContactGroup: Vec<String>,

    /// Period between pings in seconds
    CheckRate: u32,

    /// How many minutes to wait before sending an alert
    TriggerRate: Option<u32>,

    /// Fake boolean for whether or not it's paused
    Paused: Option<bool>,

    /// Fake boolean for whether or not to use public reporting
    Public: Option<u32>,

    /// Timeout in seconds
    Timeout: Option<u32>,

    StatusCodes: Option<String>,


}

impl Test {
    // TODO: allow customizing this?
    fn new(name: &str, url: &Url) -> Self {
        Self {
            WebsiteName: name.into(),
            WebsiteURL: url.to_string(),
            TestType: "HTTP".into(),
            ContactGroup: vec!["31035".into()], // hello internal id
            CheckRate: 30,

            TestID: None,
            // kind of reasonable defaults
            Paused: Some(false),
            Timeout: Some(40),
            TriggerRate: Some(3),
            Public: Some(0),
            // TODO: better way of enumerating this...
            StatusCodes: Some("204,205,206,303,400,401,403,404,405,406,408,410,413,444,429,494,495,496,499,500,501,502,503,504,505,506,507,508,509,510,511,521,522,523,524,520,598,599".into())
        }
    }
}

/// StatusCake api client
pub struct Cake {
    client: Client,
    addr: Url,
    token: String,
    user: String,
    region: String,
}
impl Cake {
    pub fn new() -> anyhow::Result<Self> {
        use std::env;
        Ok(Self {
            client: Client::new(),
            addr: Url::parse(&env::var("STATUSCAKE_BASE_URL")
                .unwrap_or("https://app.statuscake.com/API/Tests/".into()))?,
            token: env::var("STATUSCAKE_API_KEY").expect("need STATUSCAKE_API_KEY"),
            user: env::var("STATUSCAKE_USER").expect("need STATUSCAKE_USER"),
            region: env::var("SHIPCAT_REGION").expect("need SHIPCAT_REGION"),
        })
    }

    async fn get_tests(&self) -> anyhow::Result<Vec<Test>> {
        // curl -H "API: [APIKey]" -H "Username: [Username]" -X GET https://app.statuscake.com/API/Tests/
        let res = self.client
            .get(self.addr.clone())
            .header("Username", self.user.clone())
            .header("API", self.token.clone())
            .send()
            .await?;
        if !res.status().is_success() {
            let status = res.status().to_owned();
            return Err(anyhow::anyhow!("StatusCake GET failed: {}", status));
        }
        let text = res.text().await?;
        debug!("Got response: {}", text);
        let data = serde_json::from_str::<Vec<Test>>(&text)?;
        Ok(data)
    }

    pub async fn cleanup(&self, name: &str) -> anyhow::Result<Option<Test>> {
        // 1. get all tests
        let tests = self.get_tests().await?;
        // 2. find the id of the one with a matching name and region
        let orphan = tests.iter().find(|t| {
            t.WebsiteName == format!("{} {} healthcheck", self.region, name)
        });
        // 3. delete the test
        // curl -H "API: [API Key]" -H "Username: [Username]"  -d "TestID=26309&WebsiteName=MySite" -X PUT https://app.statuscake.com/API/Tests/Update
        // presumably, using -X DELETE
        if let Some(o) = orphan {
            info!("found orphan {:?}", o);
            return Ok(Some(o.clone()));
        }
        unimplemented!()

    }
}

#[cfg(test)]
mod tests {
    use super::Cake;

    #[tokio::test]
    async fn get_tests() {
        std::env::set_var("RUST_LOG", "trace");
        env_logger::init();
        let cake = Cake::new().unwrap();
        //let tests = cake.get_tests().await.unwrap();
        let o = cake.cleanup("ir-frontend").await.unwrap();
        println!("tests {:?}", o);
    }
}
