# Operations

Running `wss-mux` in production: when you need more than one instance,
how cross-instance fanout works, how to deploy it with zero
configuration on Kubernetes, and how to diagnose it when something
looks wrong.

## Single vs. multi-instance

One instance is the simple case and the right default. Each instance
is single-instance-coherent: it only knows its own connected clients.
You need more than one instance when:

| Driver | Detail |
|---|---|
| Connection count | A single process saturates (~100k+ concurrent WebSockets, or you hit fd/CPU limits). |
| High availability | A node/pod failure must not take down all clients. |
| Autoscaling | An HPA scales replicas with load. |

Going multi-instance introduces one problem: a producer pushes an
event to *one* instance, but subscribers for that event are spread
across *all* instances. `wss-mux` solves this with **peer-relay** — no
external infrastructure, no producer changes.

If you run a single instance, none of the rest of this document
applies: there are no peers, discovery resolves nothing, and behaviour
is byte-identical to having no peer-relay code at all.

## How peer-relay works

```
producer --push--> instance A --relay(once)--> instance B, C, ...
                       |                            |
                       +--> A's own clients         +--> their own clients
```

1. A producer `POST`s to any instance (`POST /v1/events`), unchanged.
2. That instance dispatches to its own subscribers **and** relays the
   event once to every discovered peer (`POST /internal/v1/relay`,
   CBOR, authenticated with the same `WSS_MUX_PUSH_AUTH_TOKEN`).
3. Each peer dispatches to *its* own subscribers and does **not**
   re-relay. The distinct receipt path is a structural one-hop guard —
   there is no gossip, no multi-hop, no loop.

Properties:

- Producer write-count is O(1) in the instance count (one push, as
  before). The producer contract never changes.
- Relay is **best-effort and fire-and-forget**. A slow or dead peer
  never reaches the producer (the `204` returns as soon as the event
  is accepted locally). A failed relay is dropped and metered, never
  retried. This is consistent with the at-most-once, ephemeral
  contract.
- Resolving no peers ⇒ no relay ⇒ behaviour byte-identical to a single
  instance.

## Zero-config Kubernetes deployment

Deploy the manifest below **unchanged** and peer-relay works with no
environment configuration, no downward API, and no sidecars. The
instance auto-detects its namespace from the mounted ServiceAccount,
composes the conventional FQDN
`wss-mux-headless.<namespace>.svc.cluster.local`, resolves it on a
short interval, and excludes its own address automatically.

```yaml
# Client/producer traffic (load-balanced, normal ClusterIP).
apiVersion: v1
kind: Service
metadata:
  name: wss-mux
spec:
  selector: { app: wss-mux }
  ports:
    - { name: http, port: 8080, targetPort: 8080 }
---
# Peer discovery ONLY. Headless (clusterIP: None) so DNS returns one
# A record per ready pod. The name MUST be `wss-mux-headless` for the
# zero-config path; rename it and you must set WSS_MUX_PEER_SERVICE.
apiVersion: v1
kind: Service
metadata:
  name: wss-mux-headless
spec:
  clusterIP: None
  selector: { app: wss-mux }
  ports:
    - { name: http, port: 8080, targetPort: 8080 }
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: wss-mux
spec:
  replicas: 3
  selector: { matchLabels: { app: wss-mux } }
  template:
    metadata:
      labels: { app: wss-mux }
    spec:
      containers:
        - name: wss-mux
          image: ghcr.io/alternet-dev/wss-mux:latest
          ports: [{ containerPort: 8080 }]
          env:
            - { name: WSS_MUX_PUSH_AUTH_TOKEN, valueFrom: { secretKeyRef: { name: wss-mux, key: push-token } } }
            - { name: WSS_MUX_HANDSHAKE_SIGNING_KEY, valueFrom: { secretKeyRef: { name: wss-mux, key: signing-key } } }
            - { name: WSS_MUX_STREAMS_MANIFEST_PATH, value: /etc/wss-mux/streams.yaml }
          volumeMounts:
            - { name: manifest, mountPath: /etc/wss-mux, readOnly: true }
          readinessProbe: { httpGet: { path: /readyz, port: 8080 } }
          livenessProbe:  { httpGet: { path: /healthz, port: 8080 } }
      volumes:
        - name: manifest
          configMap: { name: wss-mux-streams }
```

- A plain `Deployment` is sufficient. No `StatefulSet`, no sticky
  sessions, no pod identity: any instance can receive any producer
  push and relay it.
- Clients and producers use the regular `wss-mux` Service. The
  `wss-mux-headless` Service is *only* for instance-to-instance
  discovery — do not point clients at it.
- `clusterIP: None` is required: a headless Service is what makes DNS
  return per-pod A records. A normal ClusterIP Service resolves to a
  single virtual IP, not the individual pods, so discovery cannot
  enumerate the fleet ⇒ no fanout.

### Discovery knobs

All optional. The defaults are what make the manifest above
zero-config.

| Env var | Default | Meaning |
|---|---|---|
| `WSS_MUX_PEER_RELAY` | (on) | `off` disables relay entirely and silences the discovery query. |
| `WSS_MUX_PEER_SERVICE` | `wss-mux-headless` | Headless Service name for the composed FQDN. |
| `WSS_MUX_PEER_NAMESPACE` | (auto) | Overrides the namespace auto-read from the ServiceAccount file. |
| `WSS_MUX_CLUSTER_DOMAIN` | `cluster.local` | Cluster DNS domain. |
| `WSS_MUX_PEER_DNS` | — | Full FQDN override; skips composition (non-conventional Service names). |
| `WSS_MUX_PEERS` | — | Static CSV (`http://h1:8080,https://h2:8080`); a fixed fleet, no DNS polling. Non-k8s. |
| `WSS_MUX_PEER_DNS_REFRESH_SECS` | `3` | Re-resolution interval. |
| `WSS_MUX_PEER_RELAY_TIMEOUT_MS` | `500` | Per-relay request timeout. |
| `WSS_MUX_PEER_CA_FILE` | — | Trust a private CA for `https://` peers. |
| `WSS_MUX_PEER_CLIENT_CERT` / `_KEY` | — | Client cert + key for mutual TLS to peers. |

### Peer TLS / mTLS

In-cluster pod-to-pod traffic is plaintext `http://` by default — the
norm, and what the zero-config path uses. For a private-CA or
zero-trust network, peers can speak TLS without any code change:

```yaml
env:
  # `https://` peers signed by a private cluster CA:
  - { name: WSS_MUX_PEER_DNS, value: "https://wss-mux-headless.prod.svc.cluster.local:8080" }
  - { name: WSS_MUX_PEER_CA_FILE, value: /etc/wss-mux-tls/ca.pem }
  # ...and mutual TLS (both must be set, or startup fails):
  - { name: WSS_MUX_PEER_CLIENT_CERT, value: /etc/wss-mux-tls/client.pem }
  - { name: WSS_MUX_PEER_CLIENT_KEY,  value: /etc/wss-mux-tls/client-key.pem }
```

A bad CA/cert path fails startup loudly rather than silently
degrading. Client-edge TLS termination is unchanged and remains the
reverse proxy's job — this is a separate axis.

## Autoscaling (HPA)

With the conventional `wss-mux-headless` Service, autoscaling works
out of the box: a new pod becomes a DNS A record once Ready, every
instance picks it up within `WSS_MUX_PEER_DNS_REFRESH_SECS`, and it
both relays to and receives relays from the rest of the fleet.

> **If you rename the headless Service and set neither
> `WSS_MUX_PEER_SERVICE` nor `WSS_MUX_PEER_DNS`, discovery resolves
> nothing.** Every instance then behaves single-instance: a producer
> push reaches only the one instance it landed on, so roughly
> **(N−1)/N of clients miss every event** on an N-replica deployment.
> There is no error — delivery just silently degrades. Always confirm
> `wss_mux_peers_known` ≈ N−1 on each pod after a deploy.

### Scale-down and reconnection

When a pod terminates, its WebSocket connections drop. Subscriptions
are **ephemeral** — there is no server-side persistence (durable
subscriptions are a v0.5 roadmap item). Clients must reconnect
(ideally with jittered backoff) and re-send their `auth` +
`subscribe` frames. Plan for reconnect storms on rolling restarts;
make sure your client library re-subscribes automatically.

A terminating pod may linger in peers' sets for up to one refresh
interval. Relays to it fail fast (connection refused / timeout within
`WSS_MUX_PEER_RELAY_TIMEOUT_MS`), are metered as
`wss_mux_relay_failed_total`, and dropped. This is expected and
harmless during scale-down — it does not affect the producer or
delivery to live instances.

### DNS-staleness window

Membership is *polled*, not pushed. Both blind spots are bounded by
`WSS_MUX_PEER_DNS_REFRESH_SECS` (default 3s):

- **Scale-up:** a new pod is invisible to peers until the next
  refresh — its clients miss events published in that ≤3s window.
- **Scale-down:** a dead pod lingers in the set until the next
  refresh; relays to it fail and are absorbed best-effort.

Lower the interval for a tighter window at the cost of more (cached,
cheap) DNS queries; raise it to reduce queries at the cost of a wider
window. Per-relay delivery failure is *not* polled — it is handled
inline per request.

## Non-Kubernetes / fixed fleet

No DNS, fixed hosts (bare metal, Compose, a static VM set): list peers
explicitly. Static peers are never polled.

```
WSS_MUX_PEERS=http://10.0.0.11:8080,http://10.0.0.12:8080
```

Each instance lists the *others* (its own address, if included, is
auto-excluded). The same `WSS_MUX_PUSH_AUTH_TOKEN` must be shared
across the fleet.

## Why peer-relay and not Redis

The roadmap originally slated a Redis pub/sub adapter for
cross-instance fanout. It was rejected:

- **Infrastructure-agnostic pitch.** Redis imposes external infra to
  run, secure, monitor, and scale — exactly what `wss-mux` exists to
  avoid. Peer-relay rides the HTTP stack and cluster DNS you already
  have: zero new infrastructure, zero new dependency class.
- **Contract fit.** `wss-mux` is at-most-once and ephemeral.
  Best-effort, drop-on-failure relay matches that exactly; a durable
  broker would be contract-mismatched weight.
- **Operational surface.** Peer-relay has no broker to fail
  independently of the mux, no connection pool to a stateful service,
  no failover semantics to reason about.

State replication beyond this optional event relay remains an explicit
non-goal. Durable subscriptions (surviving restarts) are a separate,
opt-in v0.5 item.

## Observability

Scrape `GET /metrics` (Prometheus/OpenMetrics). Peer-relay signals:

| Metric | Healthy | Investigate when |
|---|---|---|
| `wss_mux_peers_known` (gauge) | ≈ replicas − 1 on every pod | `0` (or far below N−1) in a multi-instance deploy |
| `wss_mux_relay_sent_total` | rising with producer traffic | flat while events flow and peers exist |
| `wss_mux_relay_failed_total{reason}` | ~0 steady state; brief blips on scale-down | sustained/climbing outside a scale event |
| `wss_mux_events_rejected_total{reason="payload_too_large"}` | 0 | rising — producers exceeding a stream's `max_payload_bytes` |

`reason` on relay failure is `timeout`, `connect`, `status`, or
`other`. Sustained `connect` usually means a NetworkPolicy or a wrong
peer port; `status` means peers are up but rejecting (often auth/token
skew); `timeout` means peers are overloaded or
`WSS_MUX_PEER_RELAY_TIMEOUT_MS` is too tight.

## Symptoms → cause → fix

| Symptom | Likely cause | Fix |
|---|---|---|
| Clients on some pods miss events after scale-up; `wss_mux_peers_known` low/0 | Headless Service renamed without `WSS_MUX_PEER_SERVICE`/`_DNS`; or not headless (`clusterIP` not `None`); or `WSS_MUX_PEER_RELAY=off`; or wrong namespace | Restore the conventional Service or set the knob; ensure `clusterIP: None`; confirm relay is on |
| `wss_mux_peers_known` = 0 single-instance | Expected — no peers | Nothing; behaviour is correct |
| `wss_mux_relay_failed_total{reason="connect"}` sustained | NetworkPolicy blocking pod-to-pod `:8080`; wrong port; peers not Ready | Allow intra-deployment `:8080`; verify port; check readiness |
| `…{reason="status"}` sustained | `WSS_MUX_PUSH_AUTH_TOKEN` differs across pods | Use one shared secret fleet-wide |
| `…{reason="timeout"}` sustained | Peers overloaded; or timeout too tight | Scale out; raise `WSS_MUX_PEER_RELAY_TIMEOUT_MS` |
| All relays fail right after enabling TLS | Bad CA / client cert path or PEM | Check `WSS_MUX_PEER_CA_FILE` / `_CLIENT_CERT` / `_KEY`; startup also logs this |
| Brief relay-failure spike during a rollout | Terminating pods still in the peer set | Expected; bounded by the refresh interval |
| Producer sees latency or timeouts | **Not** peer-relay — it is fire-and-forget and never blocks the producer | Look at the producer path, manifest load (`/readyz`), or the client side |
| `413` from `POST /v1/events` | Payload exceeds the stream's `max_payload_bytes` | Raise the cap in the manifest or shrink the payload |
| Duplicate events at a client | Should not happen (one-hop guard). Almost always an external loop | Ensure nothing re-`POST`s to `/internal/v1/relay`; it is internal-only |
| Reconnect storm on every deploy | Expected with ephemeral connections on rolling restart | Ensure clients reconnect with jittered backoff and auto-resubscribe |
