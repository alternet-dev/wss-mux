# Deployment patterns

Choose a deployment topology from where subscribers for an event are
connected, not from the number of pods alone. In every multi-instance
deployment today, the instance that accepts a producer push dispatches locally
and relays the event once to every peer. Peer-relay preserves delivery across
the fleet, but it is an unconditional broadcast: it does not yet know which
peers have interested subscribers.

This guide chooses how client and producer traffic reaches the fleet. For the
peer-discovery setup, TLS, and failure runbook, see
[Operations](operations.md).

## Decision tree

```text
One instance is enough?
  yes -> Run one instance. No peer topology is needed.
  no  -> Are most events for an unkeyed audience spread across the fleet?
           yes -> Broadcast
           no  -> Are subscribers unintentionally clustered on one instance?
                    yes -> Concentrated
                    no  -> Keyed
```

| Pattern | Subscriber placement | First choice | What to watch |
|---|---|---|---|
| [Broadcast](#broadcast) | A broad, unkeyed audience is balanced among instances. | Regular `ClusterIP` Service; default kube-proxy iptables round-robin is sufficient. | Normal peer count and relay health. Selective relay has little to save. |
| [Concentrated](#concentrated) | One unkeyed audience is mostly on one instance. | Keep the regular `ClusterIP` Service, then fix the client-connection imbalance. | High `wss_mux_relay_events_unwanted_total` relative to relayed events. |
| [Keyed](#keyed) | Many small keyed audiences are scattered among instances. | Keep the regular `ClusterIP` Service; evaluate key-aware placement only when its complexity is justified. | Relay waste that grows with fleet size. |

The built-in load harness uses these same names. Measure a representative
fleet before changing routing:

```bash
cargo run --bin wss-mux-loadgen -- --peers 3 --topology broadcast throughput
cargo run --bin wss-mux-loadgen -- --peers 3 --topology concentrated throughput
cargo run --bin wss-mux-loadgen -- --peers 3 --topology keyed throughput
```

Compare the throughput result and the relay-waste ratio
`wss_mux_relay_events_unwanted_total /
wss_mux_relay_events_relayed_total`. A high ratio identifies relay work with
no local subscriber; it does not by itself indicate lost delivery.

## Service roles: regular `ClusterIP` and headless

Use two Service roles in a Kubernetes fleet:

| Service | Consumers | Purpose |
|---|---|---|
| Regular `ClusterIP` (`wss-mux` in the example manifest) | Clients and producers | Presents one load-balanced endpoint for WebSocket connections and `POST /events`. |
| Headless (`wss-mux-headless`, `clusterIP: None`) | `wss-mux` instances only | Returns the ready pod addresses that peer-relay needs. |

The headless Service is not a client-sharding endpoint. It exists so each
instance can enumerate peers; pointing clients or producers at it moves
balancing responsibility into every caller and makes failure handling harder.
Conversely, a regular `ClusterIP` cannot replace it for discovery because it
resolves to one virtual IP rather than the ready pod addresses. The complete
two-Service manifest is in [Operations](operations.md#zero-config-kubernetes-deployment).

## Broadcast

Use this pattern for a popular, unkeyed stream whose subscribers are broadly
spread across healthy instances: a dashboard feed is a typical example. Each
peer normally has a matching subscriber, so relaying an event to every peer
is useful work.

Route client and producer traffic through the regular `ClusterIP` Service.
Default kube-proxy iptables round-robin is enough to distribute new TCP
connections in this case. Do not add an L7 load balancer or a key-aware
routing layer just to improve this topology: peer-relay already supplies the
needed fanout, and planned selective relay has little to save for a balanced
broadcast audience ([#51](https://github.com/alternet-dev/wss-mux/issues/51)).

WebSocket connections are long-lived, so connection balancing is about the
arrival of new connections rather than redistribution of existing ones. Check
connection counts and `wss_mux_peers_known` after rollouts and scale events.

## Concentrated

This pattern means an unkeyed audience has accumulated on one instance while
the fleet has other instances. It is usually an anti-pattern: an event
accepted elsewhere still relays to every peer, but most peers have no local
subscriber and the concentrated instance bears the client work.

Start with the regular `ClusterIP` Service and find the source of the
clustering, such as client affinity, a misconfigured external balancer, or a
small audience that has not had a chance to spread. If the cluster runs
kube-proxy in IPVS mode, consider its `lc` (least-connection) scheduler for
new client connections. It can improve placement for this pattern, but it
does not move established WebSockets and it is not a substitute for correcting
explicit stickiness.

Interest-based selective relay is likely valuable when this pattern is common,
because the waste grows with fleet size. It is designed but not implemented;
follow [#51](https://github.com/alternet-dev/wss-mux/issues/51) rather than
expecting a current configuration switch to change relay behavior.

## Keyed

Use this pattern for many small audiences identified by a key, such as chat
rooms or per-user channels. A key's subscribers can be distributed across a
fleet even when overall connection balancing is healthy. An event for one key
therefore interests only a subset of peers, so unconditional relay naturally
does more unwanted work as the fleet grows.

First, keep the regular `ClusterIP` Service and measure the waste ratio. If it
is material, consider client-side key-aware sharding: clients for the same key
choose the same deliberately exposed shard or routing key. That design must
preserve reconnect behavior and avoid using the internal headless peer Service
as a public endpoint. It is an application-routing decision, not a required
`wss-mux` setting.

Selective relay would offer an additional, often larger reduction for this
pattern, but is not present today. Its coordination cost is why it remains an
opt-in future design ([#51](https://github.com/alternet-dev/wss-mux/issues/51)).

## When an L7 load balancer earns its cost

A regular `ClusterIP` is an L4 service: it selects a backend when a TCP
connection is established. Add an L7 proxy such as Envoy or NGINX when a
specific L7 property pays for its operational cost, for example:

- TLS termination, edge policy, or HTTP-level observability already requires
  it.
- Producer requests must be balanced per request rather than per TCP
  connection.
- The application can expose a stable routing key that the proxy can safely
  use for consistent placement.
- Connection draining and routing controls must be coordinated with a broader
  ingress policy.

It is not necessary merely because `wss-mux` has multiple replicas, and it
cannot infer a subscription key from frames after a WebSocket upgrade. For a
keyed design, make the routing input explicit at the application boundary
rather than assuming the proxy can inspect WebSocket traffic.

Keep peer-relay on the direct peer path described in [Operations](operations.md);
an edge proxy is for client and producer traffic, not peer discovery. Do not
configure automatic retries for `POST /events`: `wss-mux` does not deduplicate
pushes, so a retry after an uncertain response can duplicate an event.

## HTTP/1.1 producer keep-alive pins a backend

With a `ClusterIP`, kube-proxy chooses a backend pod when it sees the TCP SYN.
An HTTP/1.1 producer that reuses one keep-alive connection sends every later
`POST /events` on that same TCP connection, so all of those pushes reach the
same pod. This is normal connection behavior, not peer-relay failure: events
still relay to subscribers on other pods. It can nevertheless concentrate
HTTP parsing, local dispatch, and outbound relay work on one ingress pod.

Mitigate the imbalance at the producer or ingress boundary:

1. Use a bounded pool of multiple producer connections rather than one
   permanent connection.
2. Periodically recycle idle producer connections if a pool is not available.
3. Use an L7 proxy when per-request producer balancing is worth its cost.
4. Scale the fleet only after confirming that ingress concentration, rather
   than fleet-wide capacity, is the limiting factor.

Watch per-pod CPU and connection/relay metrics together; a single hot ingress
pod with otherwise healthy peers is the characteristic signal. There is no
per-source rate limit for HTTP `POST /events` today. The existing
`WSS_MUX_WS_PUBLISH_RATE` setting applies only to WebSocket publish frames.
HTTP push fairness needs a producer identity model
([#56](https://github.com/alternet-dev/wss-mux/issues/56))
and is tracked as [#60](https://github.com/alternet-dev/wss-mux/issues/60).
The separate opt-in overload-signal discussion is
[#52](https://github.com/alternet-dev/wss-mux/issues/52).
