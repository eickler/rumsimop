# rumsimop: A Kubernetes operator for creating simulated MQTT workloads

Install the CRD for defining simulation workloads into your cluster.

```
cargo run --bin crdgen | kubectl apply -f -
```

Install the operator (TBD):

```
helm install rumsimop charts \
  --set BROKER_URL=mqtt://mybroker.com:1883
  --set BROKER_USER=<user>
  --set BROKER_PASS=<pass>
```

Optionally: Monitor through OTLP: OTLP_COLLECTOR, OTLP_AUTH.

Run a workload:

```
kubectl apply -f example_simulation.yaml
```

Kubernetes operator for running the Rust MQTT simulator rumsim

Install with

```
cargo run --bin crdgen | kubectl apply -f -
```

Ideas/notes:

- Namespace handling! (simulation per namespace?)
