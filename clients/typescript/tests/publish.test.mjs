// Behavior tests for WssMuxClient.publish() using a mock WebSocket server.
// Runs against the compiled output in ../dist (run `npm run build` first).

import { test } from "node:test";
import assert from "node:assert/strict";
import { WebSocketServer, WebSocket as WsClient } from "ws";

import {
  WssMuxClient,
  ProtocolError,
  ConnectionClosedError,
} from "../dist/index.js";

const SUBPROTOCOL_JSON = "wss-mux";

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

// publishSettleMs kept tight so tests don't slow.
function makeClient(url, extra = {}) {
  return new WssMuxClient({
    wssUrl: url,
    getToken: async () => "test-token",
    WebSocket: WsClient,
    reconnect: { initialBackoffMs: 5, maxBackoffMs: 50 },
    publishSettleMs: 25,
    ...extra,
  });
}

const tick = (ms = 30) => new Promise((r) => setTimeout(r, ms));

// --- tests ---

test("publish sends a publish frame with id/stream/key/payload and resolves", async () => {
  const srv = await startServer();
  const client = makeClient(srv.url);

  await client.publish("chat", "room-42", { from: "alice", text: "hi" });

  // auth + publish
  assert.equal(srv.frames.length, 2);
  assert.deepEqual(srv.frames[0], { type: "auth", token: "test-token" });
  const pub = srv.frames[1];
  assert.equal(pub.type, "publish");
  assert.match(pub.id, /^pub-\d+$/);
  assert.equal(pub.stream, "chat");
  assert.equal(pub.key, "room-42");
  assert.deepEqual(pub.payload, { from: "alice", text: "hi" });

  await client.close();
  await srv.close();
});

test("publish without key omits the field", async () => {
  const srv = await startServer();
  const client = makeClient(srv.url);

  await client.publish("notifications", { online: true });
  const pub = srv.frames.find((f) => f.type === "publish");
  assert.ok(pub);
  assert.equal(pub.stream, "notifications");
  assert.ok(!("key" in pub));
  assert.deepEqual(pub.payload, { online: true });

  await client.close();
  await srv.close();
});

test("publish rejects with ProtocolError when server sends matching error frame", async () => {
  const srv = await startServer();
  const errors = [];
  // Reject the first publish on the server side.
  srv.wss.on("connection", (ws) => {
    ws.on("message", (data) => {
      const f = JSON.parse(data.toString());
      if (f.type === "publish") {
        ws.send(
          JSON.stringify({
            type: "error",
            code: "unauthorized_publish",
            message: "no producer audience match",
            id: f.id,
          }),
        );
      }
    });
  });
  const client = makeClient(srv.url, { onError: (e) => errors.push(e) });

  await assert.rejects(
    () => client.publish("chat", { text: "denied" }),
    (err) => {
      assert.ok(err instanceof ProtocolError);
      assert.equal(err.code, "unauthorized_publish");
      assert.match(err.subscriptionId, /^pub-\d+$/);
      return true;
    },
  );

  // onError also fires — same observability contract as subscribe errors.
  assert.equal(errors.length, 1);
  assert.equal(errors[0].code, "unauthorized_publish");

  await client.close();
  await srv.close();
});

test("multiple concurrent publishes settle independently", async () => {
  const srv = await startServer();
  // Reject only publishes that target stream "denied"; let others settle.
  srv.wss.on("connection", (ws) => {
    ws.on("message", (data) => {
      const f = JSON.parse(data.toString());
      if (f.type === "publish" && f.stream === "denied") {
        ws.send(
          JSON.stringify({
            type: "error",
            code: "unauthorized_publish",
            message: "no audience match",
            id: f.id,
          }),
        );
      }
    });
  });
  const client = makeClient(srv.url);

  const results = await Promise.allSettled([
    client.publish("chat", { n: 1 }),
    client.publish("denied", { n: 2 }),
    client.publish("chat", { n: 3 }),
  ]);

  assert.equal(results[0].status, "fulfilled");
  assert.equal(results[1].status, "rejected");
  assert.ok(results[1].reason instanceof ProtocolError);
  assert.equal(results[1].reason.code, "unauthorized_publish");
  assert.equal(results[2].status, "fulfilled");

  await client.close();
  await srv.close();
});

test("publish rejects with ConnectionClosedError when server drops while pending", async () => {
  const srv = await startServer();
  // On any publish, sever the connection without responding.
  srv.wss.on("connection", (ws) => {
    ws.on("message", (data) => {
      const f = JSON.parse(data.toString());
      if (f.type === "publish") {
        ws.terminate();
      }
    });
  });
  // Cap reconnect attempts so the test doesn't hang on background retries.
  const client = makeClient(srv.url, { reconnect: { maxAttempts: 0 } });

  await assert.rejects(
    () => client.publish("chat", { x: 1 }),
    (err) => err instanceof ConnectionClosedError,
  );

  await client.close();
  await srv.close();
});

test("publish rejects with ClientUsageError when client is closed", async () => {
  const srv = await startServer();
  const client = makeClient(srv.url);
  await client.close();

  await assert.rejects(
    () => client.publish("chat", { x: 1 }),
    /closed/,
  );
  await srv.close();
});

test("error frame whose id matches no pending publish or subscription still fires onError", async () => {
  // Belt-and-braces: a publish error arriving after settle (or for an
  // unknown id) should still surface to the consumer's onError hook.
  const srv = await startServer();
  const errors = [];
  const client = makeClient(srv.url, { onError: (e) => errors.push(e) });

  await client.publish("chat", { x: 1 });
  // settle window has passed; entry is no longer pending.
  await tick(40);

  srv.clients[0].send(
    JSON.stringify({
      type: "error",
      code: "rate_limited",
      message: "too many publishes",
      id: "pub-unknown",
    }),
  );
  await tick(20);

  assert.equal(errors.length, 1);
  assert.equal(errors[0].code, "rate_limited");
  assert.equal(errors[0].subscriptionId, "pub-unknown");

  await client.close();
  await srv.close();
});
