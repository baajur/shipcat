use reqwest;
use reqwest::header::Connection;
use serde_json;
use std::collections::BTreeMap;
use std::env;
use std::fs::File;
use std::io::Read;

use super::{Result, ErrorKind, ResultExt, Error};

// hyper/reqwest header for Vault GET requests
header! { (XVaultToken, "X-Vault-Token") => [String] }

fn default_addr() -> Result<String> {
    env::var("VAULT_ADDR").map_err(|_| ErrorKind::MissingVaultAddr.into())
}
fn default_token() -> Result<String> {
    env::var("VAULT_TOKEN")
        .or_else(|_: env::VarError| -> Result<String> {
            // Build a path to ~/.vault-token.
            let path = env::home_dir()
                .ok_or_else(|| { ErrorKind::NoHomeDirectory })?
                .join(".vault-token");

            // Read the file.
            let mut f = File::open(path)?;
            let mut token = String::new();
            f.read_to_string(&mut token)?;
            Ok(token)
        })
        .chain_err(|| ErrorKind::MissingVaultToken)
}

/// Secret data retrieved from Vault using only standard fields
#[derive(Debug, Deserialize)]
struct Secret {
    /// The key-value pairs associated with this secret.
    data: BTreeMap<String, String>,
    // How long this secret will remain valid for, in seconds.
    lease_duration: u64,
}

/// Vault client with cached data
pub struct Vault {
    /// Our HTTP client.  This can be configured to mock out the network.
    client: reqwest::Client,
    /// The address of our Vault server.
    addr: reqwest::Url,
    /// The token which we'll use to access Vault.
    token: String,
    /// Local cache of secrets.
    secrets: BTreeMap<String, Secret>,
    /// Whether to return a fake value for all secrets
    mock: bool,
}


impl Vault {
    /// Initialize using the same evars or token files that the `vault` CLI uses
    pub fn default() -> Result<Vault> {
        Vault::new(reqwest::Client::new(), &default_addr()?, default_token()?)
    }

    fn new<U, S>(client: reqwest::Client, addr: U, token: S) -> Result<Vault>
        where U: reqwest::IntoUrl,
              S: Into<String>
    {
        let addr = addr.into_url()?;
        Ok(Vault {
            client: client,
            addr: addr,
            token: token.into(),
            secrets: BTreeMap::new(),
            mock: false,
        })
    }

    /// Mock all `read` calls to the http client
    pub fn mock_secrets(&mut self) {
        self.mock = true;
    }

    // The actual HTTP GET logic
    fn get_secret(&self, path: &str) -> Result<Secret> {
        let url = self.addr.join(&format!("v1/{}", path))?;
        debug!("GET {}", url);

        let mkerr = || ErrorKind::Url(url.clone());
        let mut res = self.client.get(url.clone())
            // Leaving the connection open will cause errors on reconnect
            // after inactivity.
            .header(Connection::close())
            .header(XVaultToken(self.token.clone()))
            .send()
            .chain_err(&mkerr)?;

        // Generate informative errors for HTTP failures, because these can
        // be caused by everything from bad URLs to overly restrictive vault policies
        if !res.status().is_success() {
            let status = res.status().to_owned();
            let err: Error = ErrorKind::UnexpectedHttpStatus(status).into();
            return Err(err).chain_err(&mkerr);
        }

        let mut body = String::new();
        res.read_to_string(&mut body)?;
        Ok(serde_json::from_str(&body)?)
    }

    /// Read secret from a Vault via an authenticated HTTP GET (or memory cache)
    pub fn read(&mut self, key: &str) -> Result<String> {
        let pth = format!("secret/{}", key);

        // Check cache for secret first
        if !self.secrets.contains_key(&pth) {
            // Nope. Do the request, then cache the result.
            let secret = self.get_secret(&pth)?;
            self.secrets.insert(pth.to_owned(), secret);
        }

        // Retrieve secret from cache (now that it exists)
        let secret = &self.secrets[&pth];

        // NB: Currently assume each path in vault has a single `value`
        // Read the value key (which should exist)
        secret.data
            .get("value")
            .ok_or_else(|| { ErrorKind::MissingSecret(pth).into() })
            .map(|v| {
                if self.mock {
                    "VAULT_VALIDATED".into()
                } else {
                    v.clone()
                }
            })
    }
}


#[cfg(test)]
mod tests {
    use super::Vault;

    #[test]
    fn get_dev_secret() {
        let mut client = Vault::default().unwrap();
        let secret = client.read("dev-uk/amphora-svc/ASK_SECRET").unwrap();
        assert_eq!(secret, "catdog");
    }
}
