use kubernetes::client::APIClient;
use std::collections::BTreeMap;

use shipcat_definitions::{Crd, CrdList, Manifest, Config};

use super::{Result, Error};

static GROUPNAME: &str = "babylontech.co.uk";
static SHIPCATMANIFESTS: &str = "shipcatmanifests";
static SHIPCATCONFIGS: &str = "shipcatconfigs";

// Request builders
fn make_all_crd_entry_req(resource: &str, group: &str) -> Result<http::Request<Vec<u8>>> {
    let ns = std::env::var("ENV_NAME").expect("Must have an env name evar");
    let urlstr = format!("/apis/{group}/v1/namespaces/{ns}/{resource}?",
        group = group, resource = resource, ns = ns);
    let urlstr = url::form_urlencoded::Serializer::new(urlstr).finish();
    let mut req = http::Request::get(urlstr);
    req.body(vec![]).map_err(Error::from)
}
fn make_crd_entry_req(resource: &str, group: &str, name: &str) -> Result<http::Request<Vec<u8>>> {
    let ns = std::env::var("ENV_NAME").expect("Must have an env name evar");
    let urlstr = format!("/apis/{group}/v1/namespaces/{ns}/{resource}/{name}?",
        group = group, resource = resource, name = name, ns = ns);
    let urlstr = url::form_urlencoded::Serializer::new(urlstr).finish();
    let mut req = http::Request::get(urlstr);
    req.body(vec![]).map_err(Error::from)
}
/*fn watch_crd_entry_after(resource: &str, group: &str, name: &str, rver: u32) -> Result<http::Request<Vec<u8>>> {
    let urlstr = format!("/apis/{group}/v1/namespaces/dev/{resource}/{name}?",
        group = group, resource = resource, name = name);
    let mut qp = url::form_urlencoded::Serializer::new(urlstr);

    qp.append_pair("timeoutSeconds", "30");
    qp.append_pair("watch", "true");

    // last version to watch after
    //qp.append_pair("resourceVersion", &rver.to_string());

    let urlstr = qp.finish();
    let mut req = http::Request::get(urlstr);
    req.body(vec![]).map_err(Error::from)
}*/


// program interface - request consumers
pub type ManifestMap = BTreeMap<String, Manifest>;

pub fn get_shipcat_manifests(client: &APIClient) -> Result<ManifestMap> {
    let req = make_all_crd_entry_req(SHIPCATMANIFESTS, GROUPNAME)?;
    let res = client.request::<CrdList<Manifest>>(req)?;
    let mut data = BTreeMap::new();
    for i in res.items {
        data.insert(i.spec.name.clone(), i.spec);
    }
    let keys = data.keys().cloned().into_iter().collect::<Vec<_>>().join(", ");
    debug!("Initialized with: {}", keys);
    Ok(data)
}

pub fn get_shipcat_config(client: &APIClient, name: &str) -> Result<Crd<Config>> {
    let req = make_crd_entry_req(SHIPCATCONFIGS, GROUPNAME, name)?;
    let res = client.request::<Crd<Config>>(req)?;
    debug!("got config with version {}", &res.spec.version);
    // TODO: merge with version found in rolling env?
    Ok(res)
}

/*this doesn't actually work...
pub fn watch_shipcat_manifest(client: &APIClient, name: &str, rver: u32) -> Result<Crd<Manifest>> {
    let req = watch_crd_entry_after(SHIPCATMANIFESTS, GROUPNAME, name, rver)
        .expect("failed to define crd watch request");
    let res = client.request::<Crd<_>>(req)?;
    debug!("{}", &res.spec.name);
    Ok(res)
}*/

// actually unused now because everything returns from cache
/*pub fn get_shipcat_manifest(client: &APIClient, name: &str) -> Result<Crd<Manifest>> {
    let req = make_crd_entry_req(SHIPCATMANIFESTS, GROUPNAME, name)?;
    let res = client.request::<Crd<Manifest>>(req)?;
    debug!("got {}", &res.spec.name);
    // TODO: merge with version found in rolling env?
    Ok(res)
}
*/
