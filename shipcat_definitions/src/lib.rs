#![recursion_limit = "1024"]
#![allow(renamed_and_removed_lints)]
#![allow(non_snake_case)]
#![warn(rust_2018_idioms)]

#[macro_use] extern crate serde_derive;
#[macro_use] extern crate log;
#[macro_use] extern crate maplit;

#[macro_use] extern crate error_chain; // bail and error_chain macro
error_chain! {
    types {
        Error, ErrorKind, ResultExt, Result;
    }
    links {}
    foreign_links {
        Fmt(::std::fmt::Error);
        Io(::std::io::Error) #[cfg(unix)];
        Float(::std::num::ParseFloatError);
        Int(::std::num::ParseIntError);
        Tmpl(tera::Error);
        SerdeY(serde_yaml::Error);
        SerdeJ(serde_json::Error);
        UrlP(url::ParseError);
        Reqe(reqwest::Error);
        Time(::std::time::SystemTimeError);
        Chrono(chrono::format::ParseError);
    }
    errors {
        MissingVaultAddr {
            description("VAULT_ADDR not specified")
            display("VAULT_ADDR not specified")
        }
        MissingVaultToken {
            description("VAULT_TOKEN not specified")
            display("VAULT_TOKEN not specified")
        }
        UnexpectedHttpStatus(status: reqwest::StatusCode) {
            description("unexpected HTTP status")
            display("unexpected HTTP status: {}", &status)
        }
        NoHomeDirectory {
            description("can't find home directory")
            display("can't find home directory")
        }
        // TODO: rename
        Url(url: reqwest::Url) {
            description("could not access URL")
            display("could not access URL '{}'", &url)
        }
        InvalidTemplate(svc: String) {
            description("invalid template")
            display("service '{}' has invalid templates", svc)
        }
        InvalidOneOffTemplate(tpl: String) {
            description("invalid template")
            display("template '{}' is invalid", tpl)
        }
        InvalidManifest(svc: String) {
            description("manifest does not validate")
            display("manifest for {} does not validate", &svc)
        }
        InvalidSecretForm(key: String) {
            description("secret is of incorrect form")
            display("secret '{}' not have the 'value' key", &key)
        }
        SecretNotAccessible(key: String) {
            description("secret could not be reached or accessed")
            display("secret '{}'", &key)
        }
        FailedToBuildManifest(service_name: String, region_name: String) {
            description("failed to build manifest")
            display("failed to build manifest for {} in {}", &service_name, &region_name)
        }
    }
}

/// Config with regional data
pub mod region;
pub use crate::region::{Environment, KongConfig, ReconciliationMode, Region, VaultConfig, VersionScheme};
/// Master config with cross-region data
pub mod config;
pub use crate::config::{Cluster, Config, ConfigFallback, ManifestDefaults, ShipcatConfig};


/// Structs for the manifest
pub mod structs;

pub mod manifest;
pub use crate::manifest::{Manifest, ShipcatManifest};

pub mod base;
pub use crate::base::BaseManifest;

/// Definitions of teams/squads/tribes (via ewok or otherwise)
pub mod teams;

/// Crd wrappers
mod crds;
pub use crate::crds::gen_all_crds;

/// Status objects
pub mod status;
pub use status::ManifestStatus;

/// Internal classifications and states
mod states;
pub use crate::states::{ConfigState, PrimaryWorkload};

/// Computational helpers
pub mod math;

/// A renderer of `tera` templates (jinja style)
///
/// Used for small app configs that are inlined in the completed manifests.
pub mod template;

/// A Hashicorp Vault HTTP client using `reqwest`
pub mod vault;
pub use crate::vault::Vault;

pub mod deserializers;
