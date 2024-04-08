use futures::StreamExt;
use k8s_openapi::api::{apps::v1::StatefulSet, core::v1::Secret};
use kube::{
    api::{Patch, PatchParams},
    runtime::controller::{Action, Controller},
    Api, Client, ResourceExt,
};
use simulation::Simulation;
use std::{sync::Arc, time::Duration};

use crate::statefulset::{get_secret, get_statefulset};

mod settings;
mod simulation;
mod statefulset;

#[derive(thiserror::Error, Debug)]
pub enum Error {}
pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Clone)]
struct Data {
    client: Client,
}

const NAMESPACE: &str = "default";
const MANAGER: &str = "rumsim-operator";

#[tokio::main]
async fn main() -> Result<(), kube::Error> {
    let client = Client::try_default().await?;
    let sims = Api::<Simulation>::all(client.clone());
    let context = Arc::new(Data { client: client });

    Controller::new(sims.clone(), Default::default())
        .run(reconcile, error_policy, context)
        .for_each(|_| futures::future::ready(()))
        .await;

    Ok(())
}

async fn reconcile(sim: Arc<Simulation>, ctx: Arc<Data>) -> Result<Action> {
    println!(
        "reconcile request: {:?} {}",
        sim.metadata.namespace,
        sim.name_any()
    );
    let namespace = sim
        .metadata
        .namespace
        .clone()
        .unwrap_or(NAMESPACE.to_string());

    // Upsert secret
    let secrets = Api::<Secret>::namespaced(ctx.client.clone(), &namespace);
    let secret_data = get_secret(Arc::as_ref(&sim));
    let serverside = PatchParams::apply(MANAGER);
    match secrets
        .patch(
            &secret_data.name_any(),
            &serverside,
            &Patch::Apply(secret_data),
        )
        .await
    {
        Ok(secret) => println!("Updated secret: {:?}", secret),
        Err(e) => eprintln!("Failed to update secret: {}", e),
    }

    // Upsert statefulset
    let sfsets: Api<StatefulSet> = Api::namespaced(ctx.client.clone(), &namespace);
    let sf_data = get_statefulset(Arc::as_ref(&sim));
    match sfsets
        .patch(&sf_data.name_any(), &serverside, &Patch::Apply(sf_data))
        .await
    {
        Ok(sfset) => println!("Updated statefulset: {:?}", sfset),
        Err(e) => eprintln!("Failed to update statefulset: {}", e),
    }

    Ok(Action::requeue(Duration::from_secs(3600)))
}

fn error_policy(_object: Arc<Simulation>, _err: &Error, _ctx: Arc<Data>) -> Action {
    Action::requeue(Duration::from_secs(5))
}
