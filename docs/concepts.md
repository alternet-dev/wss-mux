# Concepts

A first-principles tour of what `wss-mux` is, what problem it solves,
and the vocabulary used throughout the codebase.

## The problem

WebSocket gives you one bidirectional channel between a client and a
server. That's enough for the simplest realtime apps and not enough
for anything else.

Real applications produce many concurrent flows of updates:

- A chat client wants messages, presence, notifications, typing
  indicators — at minimum.
- A trading dashboard wants ticks, order updates, position changes,
  alerts.
- A multiplayer game wants gameplay events, voice metadata, chat,
  spectator data.
- An operations console wants logs, metrics, alerts from many sources.

Modeling each flow as a separate WebSocket is expensive: each socket
pays TCP+TLS handshake cost, occupies a slot against browser
per-origin limits, and complicates server-side connection accounting.

The natural answer is to multiplex many logical flows over one
WebSocket. That's what `wss-mux` does — and a bit more.

## Two axes of multiplexing

`wss-mux` does two related things that are worth separating in your
head:

### 1. Within a connection: many subscriptions, one socket

A single client connection holds many active subscriptions. Frames on
the socket carry a subscription identifier, so the client can route
inbound events back to the right handler.

This is the "multiplexer" part of the name. It's what lets one
WebSocket carry chat messages and presence updates and notifications
all at once.

### 2. Across connections: one event, many subscriptions

A producer publishes one event. Many clients are connected, each
holding their own subscription set. The mux routes that one event to
every client whose subscription set matches.

This is the fanout layer. It's what lets a single domain event
(somebody sent a chat message) reach every viewer of the relevant
chat room without the producer caring how many viewers there are.

Subscription multiplexing without fanout would be unusual in practice
— clients almost always want to see updates produced by something
else on the server. `wss-mux` does both.

## Vocabulary

| Term | Meaning |
|---|---|
| Connection | One WebSocket between a client and the mux. Lifetime: from `connect` to `close`. |
| Stream | A named flow of events (e.g. `chat_messages`, `presence`). Streams are declared at startup. |
| Subscription | A connection's binding to a stream. Optionally narrowed by a key. Lifetime: from `subscribe` frame to `unsubscribe` or connection close. |
| Event | A discrete message produced externally, routed to matching subscriptions. |
| Frame | A protocol message on a connection (`auth`, `subscribe`, `unsubscribe`, `event`, `error`). |
| Key | Optional secondary identifier within a stream. Lets a subscription narrow to a specific instance (e.g. `key: room-42` within `chat_messages`). |
| Audience | The set of roles permitted to subscribe to a stream. Declared per-stream in the manifest. |
| Producer | The external process that publishes events to the mux. |
| Manifest | The startup-loaded declaration of which streams exist and their audiences. |

## Delivery model

`wss-mux` is intentionally minimal on delivery guarantees:

- **At-most-once.** An event is dispatched to currently matching
  subscriptions and dropped otherwise. There's no buffering, no retry.
- **Non-replayable.** No log of past events. Reconnecting clients
  receive only events produced after their subscription is active.
- **In-memory only.** Subscriptions live in process memory. A restart
  drops them; clients reconnect and resubscribe.

If the application needs at-least-once or replay, the producer
should also write to a durable store and expose a reconciliation API
for clients to catch up after a gap. `wss-mux` is the ephemeral tier.

## Backpressure

Each connection has a bounded send queue. If a slow client falls
behind the producer's pace and the queue fills, the mux closes the
connection rather than blocking the producer. The client is expected
to reconnect and reconcile via the producer's read API.

**Backpressure never propagates to the producer.** This is a
deliberate trade-off: producer simplicity over delivery guarantees.

## Auth model

Two checks, two stages:

1. **Connection auth.** The first frame on every connection is an
   `auth` frame containing a signed token. The token's claims include
   the connection's principals (roles, user IDs, tenant IDs — whatever
   the embedder's auth model uses). The mux validates the signature.
   An invalid token closes the connection.

2. **Subscription auth.** Every `subscribe` frame is checked against
   the stream's declared audience. If the connection's principals
   intersect the stream's audience roles, the subscription is
   admitted. Otherwise the mux returns an `error` frame and the
   subscription is not active.

The token issuer is external — `wss-mux` trusts the signature and
reads claims from it. The mux is never in the business of knowing
user identities, sessions, or policy.

## What `wss-mux` is, in one sentence

A stateless dispatcher that accepts events on one end and authenticated
multiplexed WebSocket subscribers on the other, routing events to
subscribers without persistence, replay, or coordination with peer
instances.

That sentence is the whole project. Every feature should follow from
it; anything that doesn't is in the wrong project.
