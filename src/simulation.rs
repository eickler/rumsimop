use k8s_openapi::serde::{Deserialize, Serialize};
use kube::CustomResource;
use schemars::JsonSchema;

#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[kube(kind = "Simulation", group = "rumsim.io", version = "v1", namespaced)]
pub struct SimulationSpec {
    pub devices: u64,
    pub data_points: u64,
    pub frequency_secs: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub qos: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seed: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wait_time_secs: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runs: Option<u64>,
}
