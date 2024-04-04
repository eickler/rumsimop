use crate::simulation::Simulation;
use kube::CustomResourceExt;
mod simulation;

fn main() {
    print!("{}", serde_yaml::to_string(&Simulation::crd()).unwrap())
}
