use futures::StreamExt;
use k8s_openapi::api::apps::v1::StatefulSet;
use kube::{
    api::PostParams,
    runtime::controller::{Action, Controller},
    Api, Client, ResourceExt,
};
use simulation::Simulation;
use std::{sync::Arc, time::Duration};

use crate::statefulset::create_owned_statefulset;

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
    println!("reconcile request: {}", sim.name_any());
    let statefulsets: Api<StatefulSet> = Api::namespaced(ctx.client.clone(), "default");
    let statefulset = create_owned_statefulset(sim);
    let pp = PostParams::default();
    match statefulsets.create(&pp, &statefulset).await {
        Ok(_) => println!("Created statefulset"),
        Err(e) => eprintln!("Failed to create statefulset: {}", e),
    }
    Ok(Action::requeue(Duration::from_secs(3600)))
}

fn error_policy(_object: Arc<Simulation>, _err: &Error, _ctx: Arc<Data>) -> Action {
    Action::requeue(Duration::from_secs(5))
}
