# rumsimop: A K8S operator for simulating MQTT workloads

## Quickstart

Install the CRD for simulation workloads into your cluster.

```
kubectl apply -f simulations.crd
```

Run the operator (TBD: create a chart).

```
export BROKER_URL=<mqtt://localhost:1883>
export BROKER_USER=<mqtt>
export BROKER_PASS=<pass>
cargo run --bin rumsimop
```

Run a workload:

```
kubectl create namespace mysimulation
kubectl apply -n mysimulation -f example_simulation.yaml
```

Check the workload:

```
kubectl get simulation -n mysimulation example-simulation
kubectl get statefulset -n mysimulation example-simulation
kubectl get secret -n mysimulation example-simulation
kubectl get pods -n mysimulation
```

Stop the workload:

```
kubectl delete simulation -n mysimulation example-simulation
```

## Functionality

When you create a simulation, the operator

- Installs a secret with the broker credentials using the namespace and name of the simulation.
- Install a statefulset with the broker URL and all further parameters using the namespace and name of the simulation.
- The number of replicas in the statefulset will correspond to the requested workload. Currently, we assume that each pod can emit 100.000 MQTT messages per second (hardcoded).
- You can enable OTLP tracing and metrics on the operator and the pods by setting
  - OTLP_COLLECTOR: URL of the OTLP collector.
  - OTLP_AUTH: Authentication string for the OTLP collector.
- You can choose a version of the simulator to be used by setting RUMSIM_VERSION (default "latest").

## Next steps

In the operator:

- Test helm charts for the operator and add them to a public registry -- gh pages?
- Test if multiple simulations can run concurrently (i.e. client ID, device IDs, does this work?)
- Add test cases
- Add observability? Anything specific required here? The results are pretty much observable.
- Check/fix running multiple operators in the same cluster (i.e. multiple MQTT destinations).
- Improve docs.
