#[macro_use]
extern crate lazy_static;

use futures::StreamExt;
use k8s_openapi::{
    api::{apps::v1::StatefulSet, core::v1::Secret},
    apimachinery::pkg::apis::meta::v1::OwnerReference,
};
use kube::{
    api::{ObjectMeta, Patch, PatchParams},
    runtime::controller::{Action, Controller},
    Api, Client, Resource, ResourceExt,
};
use simulation::Simulation;
use std::{sync::Arc, time::Duration};

use crate::statefulset::{get_secret, get_statefulset, PULL_SECRET};

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

    // Clone image pull secret
    let oref = Arc::as_ref(&sim).controller_owner_ref(&()).unwrap();
    clone_pull_secret(&namespace, oref, Arc::as_ref(&ctx)).await;

    // Upsert secret for MQTT access
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

async fn clone_pull_secret(namespace: &String, oref: OwnerReference, ctx: &Data) {
    let source_secret = get_source_secret(ctx).await.unwrap();
    let mut metadata = ObjectMeta {
        name: Some(PULL_SECRET.to_string()),
        ..Default::default()
    };
    // Delete the secret along with the simulation only when it's not my secret!
    if namespace != NAMESPACE {
        metadata.owner_references = Some(vec![oref]);
    }

    let target_secrets = Api::<Secret>::namespaced(ctx.client.clone(), &namespace);
    let target_secret = Secret {
        metadata,
        type_: Some("kubernetes.io/dockerconfigjson".to_string()),
        data: source_secret.data.clone(),
        ..Default::default()
    };

    let serverside = PatchParams::apply(MANAGER);
    match target_secrets
        .patch(
            &target_secret.name_any(),
            &serverside,
            &Patch::Apply(target_secret),
        )
        .await
    {
        Ok(secret) => println!("Updated image pull secret: {:?}", secret),
        Err(e) => eprintln!("Failed to update image pull secret: {}", e),
    }
}

async fn get_source_secret(ctx: &Data) -> Result<Secret, kube::Error> {
    let secrets = Api::<Secret>::namespaced(ctx.client.clone(), NAMESPACE);
    secrets.get(PULL_SECRET).await
}

fn error_policy(_object: Arc<Simulation>, _err: &Error, _ctx: Arc<Data>) -> Action {
    Action::requeue(Duration::from_secs(5))
}
