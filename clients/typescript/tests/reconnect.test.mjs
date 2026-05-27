// Reconnect + token-refresh behaviour.

import { test } from "node:test";
import assert from "node:assert/strict";
import { WebSocketServer, WebSocket as WsClient } from "ws";

import {
  WssMuxClient,
  CLOSE_UNAUTHENTICATED,
} from "../dist/index.js";

// The wire subprotocol the SDK negotiates. Not part of the public API
// (intentionally not re-exported from index); hardcoded here to match
// what the SDK actually sends on upgrade.
const SUBPROTOCOL_JSON = "wss-mux";

function startServer(onConnection) {
  const wss = new WebSocketServer({
    port: 0,
    handleProtocols: (protocols) =>
      protocols.has(SUBPROTOCOL_JSON) ? SUBPROTOCOL_JSON : false,
  });
  wss.on("connection", (ws) => onConnection(ws));
  return new Promise((resolve) => {
    wss.on("listening", () => {
      resolve({
        wss,
        url: `ws://127.0.0.1:${wss.address().port}`,
        close: () =>
          new Promise((res) => {
            wss.clients.forEach((c) => c.terminate());
            wss.close(() => res());
          }),
      });
    });
  });
}

test("replays subscriptions on reconnect", async () => {
  const connections = [];
  const srv = await startServer((ws) => {
    const conn = { authFrame: null, subFrames: [] };
    connections.push(conn);
    ws.on("message", (data) => {
      const frame = JSON.parse(data.toString());
      if (frame.type === "auth") conn.authFrame = frame;
      if (frame.type === "subscribe") conn.subFrames.push(frame);
    });
  });

  const client = new WssMuxClient({
    wssUrl: srv.url,
    getToken: async () => "tok",
    WebSocket: WsClient,
    reconnect: { initialBackoffMs: 5, maxBackoffMs: 20 },
  });

  await client.subscribe("alpha", "k1", () => {});
  await client.subscribe("beta", () => {});
  await new Promise((r) => setTimeout(r, 30));
  assert.equal(connections.length, 1);
  assert.equal(connections[0].subFrames.length, 2);

  // Force-close server-side: client sees a close, should reconnect.
  for (const c of srv.wss.clients) c.terminate();
  await new Promise((r) => setTimeout(r, 60));

  assert.equal(connections.length, 2, "client reconnected");
  assert.deepEqual(
    connections[1].subFrames.map((f) => f.stream).sort(),
    ["alpha", "beta"],
  );

  await client.close();
  await srv.close();
});

test("4401 close triggers fresh getToken() call", async () => {
  let connectionCount = 0;
  const srv = await startServer((ws) => {
    connectionCount += 1;
    if (connectionCount === 1) {
      // First connection: read the auth, then close with 4401.
      ws.once("message", () => ws.close(CLOSE_UNAUTHENTICATED, "expired_token"));
    }
    // Subsequent connections: stay open.
  });

  const tokens = [];
  const client = new WssMuxClient({
    wssUrl: srv.url,
    getToken: async () => {
      const t = `tok-${tokens.length + 1}`;
      tokens.push(t);
      return t;
    },
    WebSocket: WsClient,
    reconnect: { initialBackoffMs: 5, maxBackoffMs: 20 },
  });

  await client.subscribe("chat", () => {});
  await new Promise((r) => setTimeout(r, 80));

  assert.equal(connectionCount, 2, "reconnected after 4401");
  assert.equal(tokens.length, 2, "getToken called twice (initial + 4401 refresh)");
  assert.notEqual(tokens[0], tokens[1]);

  await client.close();
  await srv.close();
});

test("non-4401 reconnect does NOT call getToken again", async () => {
  let connectionCount = 0;
  const srv = await startServer((ws) => {
    connectionCount += 1;
    if (connectionCount === 1) {
      // Drop with a generic 1006-ish close (use 1011 server error).
      ws.once("message", () => ws.close(1011, "server error"));
    }
  });

  let getTokenCalls = 0;
  const client = new WssMuxClient({
    wssUrl: srv.url,
    getToken: async () => {
      getTokenCalls += 1;
      return "tok";
    },
    WebSocket: WsClient,
    reconnect: { initialBackoffMs: 5, maxBackoffMs: 20 },
  });

  await client.subscribe("chat", () => {});
  await new Promise((r) => setTimeout(r, 80));

  assert.equal(connectionCount, 2, "reconnected after server-side drop");
  assert.equal(
    getTokenCalls,
    1,
    "getToken called only on initial connect (cached on non-4401)",
  );

  await client.close();
  await srv.close();
});

test("maxAttempts: gives up after the configured cap", async () => {
  // No server: connect fails immediately every time.
  const url = "ws://127.0.0.1:1"; // port 1 is reliably refused
  const states = [];
  const client = new WssMuxClient({
    wssUrl: url,
    getToken: async () => "tok",
    WebSocket: WsClient,
    reconnect: { initialBackoffMs: 5, maxBackoffMs: 20, maxAttempts: 2 },
    onStateChange: (s) => states.push(s),
  });

  await assert.rejects(() => client.subscribe("chat", () => {}));
  assert.ok(states.includes("closed"));
});
