// Behavior tests for WssMuxClient using a mock WebSocket server.
// Runs against the compiled output in ../dist (run `npm run build` first).

import { test } from "node:test";
import assert from "node:assert/strict";
import { WebSocketServer, WebSocket as WsClient } from "ws";

import { WssMuxClient, ProtocolError } from "../dist/index.js";

// The wire subprotocol the SDK negotiates. Not part of the public API
// (intentionally not re-exported from index); hardcoded here to match
// what the SDK actually sends on upgrade.
const SUBPROTOCOL_JSON = "wss-mux";

/**
 * Start a mock wss-mux server on an ephemeral port. Returns a handle with:
 * - url:        ws:// URL to connect to
 * - frames:     all frames received from any client connection, in order
 * - clients:    server-side WebSocket instances, in connection order
 * - close():    terminate all clients and shut down
 */
function startServer() {
  const wss = new WebSocketServer({
    port: 0,
    handleProtocols: (protocols) =>
      protocols.has(SUBPROTOCOL_JSON) ? SUBPROTOCOL_JSON : false,
  });
  const frames = [];
  const clients = [];
  wss.on("connection", (ws) => {
    clients.push(ws);
    ws.on("message", (data) => {
      frames.push(JSON.parse(data.toString()));
    });
  });
  return new Promise((resolve) => {
    wss.on("listening", () => {
      resolve({
        wss,
        frames,
        clients,
        url: `ws://127.0.0.1:${wss.address().port}`,
        close: () =>
          new Promise((res) => {
            for (const c of wss.clients) c.terminate();
            wss.close(() => res());
          }),
      });
    });
  });
}

function makeClient(url, extra = {}) {
  return new WssMuxClient({
    wssUrl: url,
    getToken: async () => "test-token",
    WebSocket: WsClient,
    reconnect: { initialBackoffMs: 5, maxBackoffMs: 50 },
    ...extra,
  });
}

const tick = (ms = 30) => new Promise((r) => setTimeout(r, ms));

// --- tests ---

test("connects, sends auth frame with token, then subscribe", async () => {
  const srv = await startServer();
  const client = makeClient(srv.url);

  const subId = await client.subscribe("chat", "room-42", () => {});
  await tick();

  assert.equal(srv.frames.length, 2, "auth + subscribe");
  assert.deepEqual(srv.frames[0], { type: "auth", token: "test-token" });
  assert.equal(srv.frames[1].type, "subscribe");
  assert.equal(srv.frames[1].stream, "chat");
  assert.equal(srv.frames[1].key, "room-42");
  assert.equal(srv.frames[1].id, subId);
  assert.match(subId, /^sub-\d+$/);

  await client.close();
  await srv.close();
});

test("subscribe without key omits the field", async () => {
  const srv = await startServer();
  const client = makeClient(srv.url);

  await client.subscribe("notifications", () => {});
  await tick();

  const subFrame = srv.frames.find((f) => f.type === "subscribe");
  assert.ok(subFrame);
  assert.equal(subFrame.stream, "notifications");
  assert.ok(!("key" in subFrame));

  await client.close();
  await srv.close();
});

test("delivers event frames to the matching subscription callback", async () => {
  const srv = await startServer();
  const client = makeClient(srv.url);

  const received = [];
  const id = await client.subscribe("chat", "room-1", (e) => received.push(e));
  await tick();

  assert.equal(srv.clients.length, 1);
  srv.clients[0].send(
    JSON.stringify({
      type: "event",
      id,
      stream: "chat",
      key: "room-1",
      payload: { from: "alice", text: "hi" },
    }),
  );
  await tick(20);

  assert.equal(received.length, 1);
  assert.deepEqual(received[0].payload, { from: "alice", text: "hi" });

  await client.close();
  await srv.close();
});

test("error frame invokes onError; fatal-per-sub codes drop the subscription", async () => {
  const srv = await startServer();
  const errors = [];
  const client = makeClient(srv.url, { onError: (e) => errors.push(e) });

  const id = await client.subscribe("chat", () => {});
  await tick();

  srv.clients[0].send(
    JSON.stringify({
      type: "error",
      code: "unauthorized_subscribe",
      message: "no audience match",
      id,
    }),
  );
  await tick(20);

  assert.equal(errors.length, 1);
  assert.ok(errors[0] instanceof ProtocolError);
  assert.equal(errors[0].code, "unauthorized_subscribe");
  assert.equal(errors[0].subscriptionId, id);
  assert.equal(client.subscriptionCount, 0);

  await client.close();
  await srv.close();
});

test("keep-open error (rate_limited) leaves the subscription intact", async () => {
  const srv = await startServer();
  const errors = [];
  const client = makeClient(srv.url, { onError: (e) => errors.push(e) });

  const id = await client.subscribe("chat", () => {});
  await tick();

  srv.clients[0].send(
    JSON.stringify({
      type: "error",
      code: "rate_limited",
      message: "too many frames",
      id,
    }),
  );
  await tick(20);

  assert.equal(errors.length, 1);
  assert.equal(errors[0].code, "rate_limited");
  assert.equal(client.subscriptionCount, 1, "rate_limited keeps the sub");

  await client.close();
  await srv.close();
});

test("unsubscribe sends frame and is idempotent", async () => {
  const srv = await startServer();
  const client = makeClient(srv.url);

  const id = await client.subscribe("chat", () => {});
  await tick();

  await client.unsubscribe(id);
  await client.unsubscribe(id); // second call: no-op, no extra frame
  await tick(20);

  const unsubFrames = srv.frames.filter((f) => f.type === "unsubscribe");
  assert.equal(unsubFrames.length, 1);
  assert.equal(unsubFrames[0].id, id);
  assert.equal(client.subscriptionCount, 0);

  await client.close();
  await srv.close();
});

test("subscribe rejects when client is closed", async () => {
  const srv = await startServer();
  const client = makeClient(srv.url);
  await client.close();

  await assert.rejects(() => client.subscribe("chat", () => {}), /closed/);
  await srv.close();
});

test("transitions through expected connection states", async () => {
  const srv = await startServer();
  const states = [];
  const client = makeClient(srv.url, {
    onStateChange: (s) => states.push(s),
  });

  await client.subscribe("chat", () => {});
  await tick();

  assert.ok(states.includes("connecting"));
  assert.ok(states.includes("authenticating"));
  assert.ok(states.includes("ready"));

  await client.close();
  assert.ok(states.includes("closed"));

  await srv.close();
});
