#[macro_use] extern crate log;
use shipcat_definitions::{ShipcatManifest};
use futures::{StreamExt, TryStreamExt};

use kube::{
    api::{Api, ListParams, WatchEvent},
    runtime::Informer,
    Client,
};

mod cake;
use cake::Cake;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    std::env::set_var("RUST_LOG", "info,kube=debug");
    env_logger::init();
    let client = Client::try_default().await?;
    let namespace = std::env::var("NAMESPACE").unwrap_or("default".into());

    // Watch shipcatmanifests
    let mfs: Api<ShipcatManifest> = Api::namespaced(client, &namespace);
    let lp = ListParams::default().timeout(20); // low timeout in this example
    let mfinf = Informer::new(mfs).params(lp);

    // Prepare to make statuscake api calls based on them
    let cake = Cake::new()?;
    loop {
        let mut mfevs = mfinf.poll().await?.boxed();

        while let Some(ev) = mfevs.try_next().await? {
            if let Some(o) = check_events(ev) {
                reconcile(o, &cake).await?;
            }
        }
    }
}

async fn reconcile(sm: ShipcatManifest, cake: &Cake) -> anyhow::Result<()> {
    if let Some(dt) = &sm.metadata.deletion_timestamp {
        // TODO: check if finalizer is there
        // TODO: figure out how to let shipcatmanifest deletion background
        // i.e. exist but not block kubectl
        info!("need to gc: {} (deleted at {})", sm.spec.name, dt.0);
        cake.cleanup(&sm.spec.name).await?;
        // TODO: remove finalizer
    }
    Ok(())
}

fn check_events(ev: WatchEvent<ShipcatManifest>) -> Option<ShipcatManifest> {
    match ev {
        WatchEvent::Added(o)
        | WatchEvent::Modified(o)
        | WatchEvent::Deleted(o) => return Some(o),
        WatchEvent::Error(e) => {
            warn!("Error event: {:?}", e);
        }
        _ => {}
    }
    None
}
