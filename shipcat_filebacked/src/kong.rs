use merge::Merge;
use std::collections::BTreeMap;

use shipcat_definitions::structs::{Authentication, Authorization, BabylonAuthHeader, Cors, Kong};
use shipcat_definitions::{Region, Result};

use super::authorization::AuthorizationSource;
use super::util::{Build, Enabled};

#[derive(Deserialize, Default, Merge, Clone)]
#[serde(default, deny_unknown_fields)]
pub struct KongSource {
    pub upstream_url: Option<String>,
    pub uris: Option<String>,
    pub hosts: Option<Vec<String>>,
    pub strip_uri: Option<bool>,
    pub preserve_host: Option<bool>,
    pub cors: Option<Cors>,
    pub additional_internal_ips: Option<Vec<String>>,

    pub internal: Option<bool>,
    #[serde(rename = "camelCase")]
    pub publicly_accessible: Option<bool>,
    pub auth: Option<Authentication>,
    pub babylon_auth_header: Option<BabylonAuthHeader>,
    pub authorization: Enabled<AuthorizationSource>,

    pub upstream_connect_timeout: Option<u32>,
    pub upstream_send_timeout: Option<u32>,
    pub upstream_read_timeout: Option<u32>,
    pub add_headers: BTreeMap<String, String>,
}

pub struct KongBuildParams {
    pub service: String,
    pub region: Region,
}

impl Build<Option<Kong>, KongBuildParams> for KongSource {
    /// Build a Kong from a KongSource, validating and mutating properties.
    fn build(self, params: &KongBuildParams) -> Result<Option<Kong>> {
        let KongBuildParams { region, service } = params;
        if let Some(k) = &region.kong {
            let hosts = self.build_hosts(&k.base_url)?;

            if hosts.is_empty() && self.uris.is_none() {
                return Ok(None);
            }

            let upstream_url = self.build_upstream_url(&service, &region.namespace);
            let (auth, authorization) = KongSource::build_auth(self.auth, self.authorization)?;

            let preserve_host = self.preserve_host.unwrap_or(true);

            Ok(Some(Kong {
                name: service.to_string(),
                upstream_url: upstream_url,
                upstream_service: if preserve_host {
                    Some(service.to_string())
                } else {
                    None
                },
                internal: self.internal.unwrap_or_default(),
                publiclyAccessible: self.publicly_accessible.unwrap_or_default(),
                uris: self.uris,
                hosts,
                authorization,
                strip_uri: self.strip_uri.unwrap_or_default(),
                preserve_host,
                cors: self.cors,
                additional_internal_ips: self.additional_internal_ips.unwrap_or_default(),
                babylon_auth_header: self.babylon_auth_header,
                upstream_connect_timeout: self.upstream_connect_timeout,
                upstream_send_timeout: self.upstream_send_timeout,
                upstream_read_timeout: self.upstream_read_timeout,
                add_headers: self.add_headers,
                // Legacy authorization
                auth,
            }))
        } else {
            Ok(None)
        }
    }
}

impl KongSource {
    fn build_upstream_url(&self, service: &str, namespace: &str) -> String {
        if let Some(upstream_url) = &self.upstream_url {
            upstream_url.to_string()
        } else {
            format!("http://{}.{}.svc.cluster.local", service, namespace)
        }
    }

    fn build_auth(auth: Option<Authentication>, authz: Enabled<AuthorizationSource>) -> Result<(Authentication, Option<Authorization>)> {
        match (
            auth,
            authz.build(&())?,
        ) {
            // authorization is enabled
            (None, Some(a)) | (Some(Authentication::Jwt), Some(a)) => {
                Ok((Authentication::Jwt, Some(a)))
            }
            (Some(_), Some(_)) => bail!("auth must be unset or JWT if authorization is enabled"),
            // otherwise
            (Some(x), _) => Ok((x.clone(), None)),
            (None, _) => Ok((Authentication::default(), None)),
        }
    }

    fn build_hosts(&self, base_url: &str) -> Result<Vec<String>> {
        Ok(self.hosts.clone().unwrap_or_default().into_iter()
            .map(|h| {
                let fully_qualified = h.contains('.');
                if fully_qualified {
                    h
                } else {
                    format!("{}{}", h, base_url)
                }
            }).collect())
    }
}
