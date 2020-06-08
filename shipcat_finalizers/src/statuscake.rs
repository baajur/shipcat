#[macro_use] extern crate log;
use shipcat_definitions::{ShipcatManifest};
use futures::{StreamExt, TryStreamExt};

use kube::{
    api::{Api, ListParams, WatchEvent},
    runtime::Informer,
    Client,
};


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    std::env::set_var("RUST_LOG", "info,kube=debug");
    env_logger::init();
    let client = Client::try_default().await?;
    let namespace = std::env::var("NAMESPACE").unwrap_or("default".into());

    // This example requires `kubectl apply -f examples/foo.yaml` run first
    let mfs: Api<ShipcatManifest> = Api::namespaced(client, &namespace);
    let lp = ListParams::default().timeout(20); // low timeout in this example
    let mfinf = Informer::new(mfs).params(lp);

    loop {
        let mut mfevs = mfinf.poll().await?.boxed();

        while let Some(ev) = mfevs.try_next().await? {
            if let Action::Cleanup(o) = check_events_for_gc(ev).await? {
                info!("need to gc: {}", o.spec.name);
            }
        }
    }
}

fn reconcile(sm: ShipcatManifest) -> anyhow::Result<()> {

    Ok(())
}

enum Action {
    /// Nothing worth doing
    Nothing,
    /// We need to cleanup this manifest
    Cleanup(ShipcatManifest),
}

async fn check_events_for_gc(ev: WatchEvent<ShipcatManifest>) -> anyhow::Result<Action> {
    match ev {
        WatchEvent::Added(o) => {
            info!("Added: {}", o.spec.name);
            if let Some(_dt) = &o.metadata.deletion_timestamp {
                return Ok(Action::Cleanup(o));
            }
        }
        WatchEvent::Modified(o) => {
            info!("Modified: {} ({:?})", o.spec.name, o);
            if let Some(_dt) = &o.metadata.deletion_timestamp {
                return Ok(Action::Cleanup(o));
            }
        }
        WatchEvent::Deleted(o) => {
            info!("Deleted: {}", o.spec.name);
        }
        WatchEvent::Error(e) => {
            warn!("Error event: {:?}", e);
        }
        _ => {}
    }
    Ok(Action::Nothing)
}
