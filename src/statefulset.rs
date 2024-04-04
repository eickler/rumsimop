use crate::Simulation;
use k8s_openapi::api::apps::v1::StatefulSet;
use kube::{api::ObjectMeta, Resource};
use std::sync::Arc;

const POD_CAPACITY: u64 = 100000;

pub fn create_owned_statefulset(sim: Arc<Simulation>) -> StatefulSet {
    let oref = sim.controller_owner_ref(&()).unwrap();
    let replicas = sim.spec.devices * sim.spec.data_points / sim.spec.frequency_secs / POD_CAPACITY;

    StatefulSet {
        metadata: ObjectMeta {
            name: Some("rumsim-statefulset".to_string()),
            owner_references: Some(vec![oref]),
            ..Default::default()
        },
        spec: Some(k8s_openapi::api::apps::v1::StatefulSetSpec {
            service_name: "rumsim".to_string(),
            replicas: Some(replicas as i32),
            selector: k8s_openapi::apimachinery::pkg::apis::meta::v1::LabelSelector {
                match_labels: Some(
                    [("app".to_string(), "rumsim".to_string())]
                        .iter()
                        .cloned()
                        .collect::<std::collections::BTreeMap<_, _>>(),
                ),
                ..Default::default()
            },
            template: k8s_openapi::api::core::v1::PodTemplateSpec {
                metadata: Some(ObjectMeta {
                    labels: Some(
                        [("app".to_string(), "rumsim".to_string())]
                            .iter()
                            .cloned()
                            .collect::<std::collections::BTreeMap<_, _>>(),
                    ),
                    ..Default::default()
                }),
                spec: Some(k8s_openapi::api::core::v1::PodSpec {
                    containers: vec![k8s_openapi::api::core::v1::Container {
                        name: "rumsim".to_string(),
                        image: Some("eickler/rumsim:latest".to_string()),
                        env: Some(vec![
                            k8s_openapi::api::core::v1::EnvVar {
                                name: "RUMSIM_FREQUENCY".to_string(),
                                value: Some("1".to_string()),
                                ..Default::default()
                            },
                            k8s_openapi::api::core::v1::EnvVar {
                                name: "RUMSIM_DATA_POINTS".to_string(),
                                value: Some("100000".to_string()),
                                ..Default::default()
                            },
                            k8s_openapi::api::core::v1::EnvVar {
                                name: "CLIENT_ID".to_string(),
                                value_from: Some(k8s_openapi::api::core::v1::EnvVarSource {
                                    field_ref: Some(
                                        k8s_openapi::api::core::v1::ObjectFieldSelector {
                                            field_path: "metadata.name".to_string(),
                                            ..Default::default()
                                        },
                                    ),
                                    ..Default::default()
                                }),
                                ..Default::default()
                            },
                        ]),
                        ..Default::default()
                    }],
                    ..Default::default()
                }),
            },
            ..Default::default()
        }),
        // Fill in the spec here...
        ..Default::default()
    }

    /*         - name: rumsim
             image: "{{ .Values.image.repository }}:{{ .Values.image.tag | default .Chart.AppVersion }}"
             env:
               - name: CLIENT_ID
                 valueFrom:
                   fieldRef:
                     fieldPath: metadata.name
               - name: URL
                 valueFrom:
                   configMapKeyRef:
                     name: rumsim-config
                     key: url
               - name: USER
                 valueFrom:
                   secretKeyRef:
                     name: rumsim-credentials
                     key: user
               - name: PASS
                 valueFrom:
                   secretKeyRef:
                     name: rumsim-credentials
                     key: pass
               - name: RUST_LOG
                 value: info
    */
}
