// End-to-end test against a real wss-mux instance.
//
// Skipped unless WSS_MUX_E2E=1. Run with: `npm run test:e2e`.
//
// Expects:
// - A wss-mux server running at WSS_MUX_E2E_URL (no default — set explicitly
//   to whatever path your deployment serves, e.g. ws://127.0.0.1:8080/stream).
// - A valid auth token in WSS_MUX_E2E_TOKEN.
// - A stream named WSS_MUX_E2E_STREAM (default: "test") in the server's manifest,
//   with an audience the token's principals satisfy.

import { test } from "node:test";
import assert from "node:assert/strict";
import { WebSocket as WsClient } from "ws";

import { WssMuxClient } from "../dist/index.js";

const ENABLED = process.env.WSS_MUX_E2E === "1";
const URL = process.env.WSS_MUX_E2E_URL ?? "";
const TOKEN = process.env.WSS_MUX_E2E_TOKEN ?? "";
const STREAM = process.env.WSS_MUX_E2E_STREAM ?? "test";

test("e2e: connect, subscribe, receive (skipped unless WSS_MUX_E2E=1)", { skip: !ENABLED }, async () => {
  assert.ok(URL, "WSS_MUX_E2E_URL must be set when WSS_MUX_E2E=1");
  assert.ok(TOKEN, "WSS_MUX_E2E_TOKEN must be set when WSS_MUX_E2E=1");

  const events = [];
  const client = new WssMuxClient({
    wssUrl: URL,
    getToken: async () => TOKEN,
    WebSocket: WsClient,
    reconnect: { maxAttempts: 0 },
  });

  const id = await client.subscribe(STREAM, (e) => events.push(e));
  assert.match(id, /^sub-/);

  // Caller is expected to push to the server through whatever channel
  // they configured (typically a separate process posting to the server's
  // push endpoint). This test only verifies the connect/subscribe path;
  // producers are out of scope for the SDK.

  await new Promise((r) => setTimeout(r, 200));
  await client.unsubscribe(id);
  await client.close();
});
