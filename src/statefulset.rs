use crate::{envbuilder::EnvVarBuilder, settings::SETTINGS, Simulation};
use k8s_openapi::{
    api::{
        apps::v1::StatefulSet,
        core::v1::{EnvVar, EnvVarSource, SecretKeySelector},
    },
    chrono::{self, Utc},
    ByteString,
};
use kube::{api::ObjectMeta, Resource, ResourceExt};
use std::collections::BTreeMap;

const NAME: &str = "rumsim";
const USER_PROPERTY: &str = "user";
const PASS_PROPERTY: &str = "pass";
pub const PULL_SECRET: &str = "regcred";
const POD_CAPACITY_SECS: u64 = 100000;

fn get_name() -> Option<BTreeMap<String, String>> {
    Some(
        [("app".to_string(), NAME.to_string())]
            .iter()
            .cloned()
            .collect::<BTreeMap<_, _>>(),
    )
}

fn get_variables(simobj: &Simulation, devices: u64) -> Vec<EnvVar> {
    let sim = &simobj.spec;
    let start_time = Utc::now() + chrono::Duration::seconds(sim.wait_time_secs.unwrap_or(0) as i64);

    EnvVarBuilder::new()
        .add_u64("SIM_DEVICES", devices)
        .add_u64("SIM_DATA_POINTS", sim.data_points)
        .add_u64("SIM_FREQUENCY_SECS", sim.frequency_secs)
        .add_str("SIM_START_TIME", &start_time.to_rfc3339())
        .add_u64_opt("SIM_RUNS", sim.runs)
        .add_u64_opt("SIM_SEED", sim.seed)
        .add_str("BROKER_URL", &SETTINGS.broker_url)
        .add_secret_ref("BROKER_USER", &simobj.name_any(), USER_PROPERTY)
        .add_secret_ref("BROKER_PASS", &simobj.name_any(), PASS_PROPERTY)
        .add_field_ref("BROKER_CLIENT_ID", "metadata.name")
        .add_u64_opt("BROKER_QOS", sim.qos)
        .add_str_opt("OTLP_ENDPOINT", SETTINGS.otlp_collector.clone())
        .add_str_opt("OTLP_AUTH", SETTINGS.otlp_auth.clone())
        .add_str_opt("RUST_LOG", SETTINGS.rust_log.clone())
        .build()
}

fn to_byte_string(s: &str) -> ByteString {
    ByteString {
        0: s.as_bytes().to_vec(),
    }
}

pub fn get_metadata(sim: &Simulation) -> ObjectMeta {
    let oref = sim.controller_owner_ref(&()).unwrap();

    ObjectMeta {
        name: Some(sim.name_any()),
        owner_references: Some(vec![oref]),
        ..Default::default()
    }
}

pub fn get_secret(sim: &Simulation) -> k8s_openapi::api::core::v1::Secret {
    // kube.rs kindly does the base64 encoding for us.
    k8s_openapi::api::core::v1::Secret {
        metadata: get_metadata(sim),
        data: Some(
            [
                (USER_PROPERTY, SETTINGS.broker_user.clone()),
                (PASS_PROPERTY, SETTINGS.broker_pass.clone()),
            ]
            .iter()
            .map(|(k, v)| (k.to_string(), to_byte_string(&v)))
            .collect::<BTreeMap<_, ByteString>>(),
        ),
        ..Default::default()
    }
}

pub fn get_statefulset(sim: &Simulation) -> StatefulSet {
    let image = format!("ghcr.io/eickler/rumsim:{}", SETTINGS.image_version);

    // Calculate the number of replicas needed.
    let replicas = (sim.spec.devices * sim.spec.data_points) as f64
        / (sim.spec.frequency_secs * POD_CAPACITY_SECS) as f64;
    let replicas = replicas.ceil() as u64;

    StatefulSet {
        metadata: get_metadata(sim),
        spec: Some(k8s_openapi::api::apps::v1::StatefulSetSpec {
            service_name: NAME.to_string(),
            replicas: Some(replicas as i32),
            selector: k8s_openapi::apimachinery::pkg::apis::meta::v1::LabelSelector {
                match_labels: get_name(),
                ..Default::default()
            },
            template: k8s_openapi::api::core::v1::PodTemplateSpec {
                metadata: Some(ObjectMeta {
                    labels: get_name(),
                    ..Default::default()
                }),
                spec: Some(k8s_openapi::api::core::v1::PodSpec {
                    image_pull_secrets: Some(vec![
                        k8s_openapi::api::core::v1::LocalObjectReference {
                            name: Some(PULL_SECRET.to_string()),
                        },
                    ]),
                    containers: vec![k8s_openapi::api::core::v1::Container {
                        name: NAME.to_string(),
                        image: Some(image),
                        env: Some(get_variables(&sim, sim.spec.devices / replicas as u64)),
                        ..Default::default()
                    }],
                    ..Default::default()
                }),
            },
            ..Default::default()
        }),
        ..Default::default()
    }
}
