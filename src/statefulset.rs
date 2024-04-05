use crate::{settings, simulation::SimulationSpec, Simulation};
use base64::prelude::*;
use k8s_openapi::{
    api::{
        apps::v1::StatefulSet,
        core::v1::{EnvVar, EnvVarSource, SecretKeySelector},
    },
    ByteString,
};
use kube::{api::ObjectMeta, Resource};
use std::{collections::BTreeMap, sync::Arc};

const NAME: &str = "rumsim";
const SECRET: &str = "rumsim-credentials";
const USER_PROPERTY: &str = "user";
const PASS_PROPERTY: &str = "pass";
const IMAGE: &str = "eickler/rumsim:latest";
const POD_CAPACITY: u64 = 100000;

fn get_name() -> Option<BTreeMap<String, String>> {
    Some(
        [("app".to_string(), NAME.to_string())]
            .iter()
            .cloned()
            .collect::<BTreeMap<_, _>>(),
    )
}

fn get_var_u64(name: &str, value: u64) -> EnvVar {
    EnvVar {
        name: name.to_string(),
        value: Some(value.to_string()),
        ..Default::default()
    }
}

fn get_var_str(name: &str, value: &str) -> EnvVar {
    EnvVar {
        name: name.to_string(),
        value: Some(value.to_string()),
        ..Default::default()
    }
}

fn get_secret_ref(name: &str, secret: &str, key: &str) -> EnvVar {
    EnvVar {
        name: name.to_string(),
        value_from: Some(EnvVarSource {
            secret_key_ref: Some(SecretKeySelector {
                name: Some(secret.to_string()),
                key: key.to_string(),
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn get_field_ref(name: &str, field: &str) -> EnvVar {
    EnvVar {
        name: name.to_string(),
        value_from: Some(EnvVarSource {
            field_ref: Some(k8s_openapi::api::core::v1::ObjectFieldSelector {
                field_path: field.to_string(),
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn get_variables(sim: &SimulationSpec, devices: u64) -> Vec<EnvVar> {
    let settings = settings::Settings::new();

    vec![
        // Simulation-related variables
        get_var_u64("SIM_DEVICES", devices),
        get_var_u64("SIM_DATA_POINTS", sim.data_points),
        get_var_u64("SIM_FREQUENCY_SECS", sim.frequency_secs),
        get_var_u64("SIM_WAIT_TIME_SECS", sim.wait_time_secs.unwrap_or(0)),
        get_var_u64("SIM_RUNS", sim.runs.unwrap_or(0)),
        // MQTT-related variables
        get_var_str("BROKER_URL", &settings.broker_url),
        get_secret_ref("BROKER_USER", SECRET, USER_PROPERTY),
        get_secret_ref("BROKER_PASS", SECRET, PASS_PROPERTY),
        get_field_ref("BROKER_CLIENT_ID", "metadata.name"),
        get_var_u64("BROKER_QOS", sim.qos.unwrap_or(1) as u64),
        // OTLP-related variables
        get_var_str(
            "OTLP_ENDPOINT",
            &settings.otlp_collector.unwrap_or("".to_string()),
        ),
        get_var_str("OTLP_AUTH", &settings.otlp_auth.unwrap_or("".to_string())),
    ]
}

fn to_byte_string(s: &str) -> ByteString {
    ByteString {
        0: BASE64_STANDARD.encode(s).as_bytes().to_vec(),
    }
}

pub fn get_secret() -> k8s_openapi::api::core::v1::Secret {
    let settings = settings::Settings::new();

    k8s_openapi::api::core::v1::Secret {
        metadata: ObjectMeta {
            name: Some(SECRET.to_string()),
            ..Default::default()
        },
        data: Some(
            [
                (USER_PROPERTY, settings.broker_user),
                (PASS_PROPERTY, settings.broker_pass),
            ]
            .iter()
            .map(|(k, v)| (k.to_string(), to_byte_string(&v)))
            .collect::<BTreeMap<_, ByteString>>(),
        ),
        ..Default::default()
    }
}

pub fn get_owned_statefulset(sim: Arc<Simulation>) -> StatefulSet {
    let oref = sim.controller_owner_ref(&()).unwrap();
    let replicas = sim.spec.devices * sim.spec.data_points / sim.spec.frequency_secs / POD_CAPACITY;

    StatefulSet {
        metadata: ObjectMeta {
            name: Some("rumsim-statefulset".to_string()),
            owner_references: Some(vec![oref]),
            ..Default::default()
        },
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
                    containers: vec![k8s_openapi::api::core::v1::Container {
                        name: NAME.to_string(),
                        image: Some(IMAGE.to_string()),
                        env: Some(get_variables(&sim.spec, sim.spec.devices / replicas as u64)),
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
