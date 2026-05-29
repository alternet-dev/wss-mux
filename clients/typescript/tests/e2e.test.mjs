// End-to-end tests against a real wss-mux instance.
//
// Skipped unless WSS_MUX_E2E=1. Run via `npm run test:e2e`, or the
// repo-level harness `tests/e2e/run.sh` which boots a server and
// wires the required env vars automatically.
//
// Required when WSS_MUX_E2E=1:
// - WSS_MUX_E2E_URL          ws://host:port/stream
// - WSS_MUX_E2E_TOKEN        JWT whose principals satisfy the stream's audience
//
// Optional (additional tests if set):
// - WSS_MUX_E2E_PUSH_URL     http://host:port/events
// - WSS_MUX_E2E_PUSH_TOKEN   bearer for WSS_MUX_PUSH_AUTH_TOKEN
// - WSS_MUX_E2E_STREAM       defaults to "chat_messages" (harness default)
// - WSS_MUX_E2E_READONLY_STREAM defaults to "readonly"

import { test } from "node:test";
import assert from "node:assert/strict";
import { WebSocket as WsClient } from "ws";

import { WssMuxClient, ProtocolError } from "../dist/index.js";

const ENABLED = process.env.WSS_MUX_E2E === "1";
const URL = process.env.WSS_MUX_E2E_URL ?? "";
const TOKEN = process.env.WSS_MUX_E2E_TOKEN ?? "";
const PUSH_URL = process.env.WSS_MUX_E2E_PUSH_URL ?? "";
const PUSH_TOKEN = process.env.WSS_MUX_E2E_PUSH_TOKEN ?? "";
const STREAM = process.env.WSS_MUX_E2E_STREAM ?? "chat_messages";
const READONLY_STREAM = process.env.WSS_MUX_E2E_READONLY_STREAM ?? "readonly";

function makeClient(extra = {}) {
  return new WssMuxClient({
    wssUrl: URL,
    getToken: async () => TOKEN,
    WebSocket: WsClient,
    reconnect: { maxAttempts: 0, initialBackoffMs: 50, maxBackoffMs: 500 },
    ...extra,
  });
}

async function httpPush(stream, key, payload) {
  if (!PUSH_URL || !PUSH_TOKEN) {
    throw new Error(
      "WSS_MUX_E2E_PUSH_URL and WSS_MUX_E2E_PUSH_TOKEN required for HTTP push tests",
    );
  }
  const body = key === undefined
    ? { stream, payload }
    : { stream, key, payload };
  const res = await fetch(PUSH_URL, {
    method: "POST",
    headers: {
      "content-type": "application/json",
      authorization: `Bearer ${PUSH_TOKEN}`,
    },
    body: JSON.stringify(body),
  });
  if (!res.ok) {
    const text = await res.text().catch(() => "");
    throw new Error(`HTTP push failed: ${res.status} ${text}`);
  }
}

function waitForEvent(events, predicate, timeoutMs = 3000) {
  return new Promise((resolve, reject) => {
    const t0 = Date.now();
    const tick = () => {
      const match = events.find(predicate);
      if (match) return resolve(match);
      if (Date.now() - t0 > timeoutMs) {
        return reject(new Error(`timed out waiting; saw ${events.length} events`));
      }
      setTimeout(tick, 20);
    };
    tick();
  });
}

test(
  "e2e: connect + subscribe + WS publish own-event roundtrip",
  { skip: !ENABLED },
  async () => {
    assert.ok(URL, "WSS_MUX_E2E_URL must be set when WSS_MUX_E2E=1");
    assert.ok(TOKEN, "WSS_MUX_E2E_TOKEN must be set when WSS_MUX_E2E=1");

    const events = [];
    const client = makeClient();
    await client.subscribe(STREAM, "ts-roundtrip", (e) => events.push(e));

    // Brief settle so the server has registered the subscribe.
    await new Promise((r) => setTimeout(r, 200));

    await client.publish(STREAM, "ts-roundtrip", {
      who: "ts",
      marker: "self-roundtrip",
    });

    const ev = await waitForEvent(events, (e) => e.key === "ts-roundtrip");
    assert.equal(ev.stream, STREAM);
    assert.equal(ev.payload.who, "ts");
    assert.equal(ev.payload.marker, "self-roundtrip");

    await client.close();
  },
);

test(
  "e2e: publish to read-only stream returns unauthorized_publish",
  { skip: !ENABLED },
  async () => {
    const client = makeClient();
    await assert.rejects(
      () => client.publish(READONLY_STREAM, "x", { ignored: true }),
      (err) => {
        assert.ok(err instanceof ProtocolError, `got ${err?.name}`);
        assert.equal(err.code, "unauthorized_publish");
        return true;
      },
    );
    await client.close();
  },
);

test(
  "e2e: HTTP push is observed by WS subscriber",
  {
    skip:
      !ENABLED ||
      !process.env.WSS_MUX_E2E_PUSH_URL ||
      !process.env.WSS_MUX_E2E_PUSH_TOKEN,
  },
  async () => {
    const events = [];
    const client = makeClient();
    await client.subscribe(STREAM, "ts-http-fanout", (e) => events.push(e));
    await new Promise((r) => setTimeout(r, 200));

    await httpPush(STREAM, "ts-http-fanout", {
      who: "ts-http",
      marker: "fanout",
    });
    const ev = await waitForEvent(events, (e) => e.key === "ts-http-fanout");
    assert.equal(ev.payload.who, "ts-http");
    assert.equal(ev.payload.marker, "fanout");

    await client.close();
  },
);

test(
  "e2e: TS publish observed by another TS subscriber (cross-client)",
  { skip: !ENABLED },
  async () => {
    // Two independent SDK instances on the same server. Publisher sends
    // a `publish` frame; the subscriber observes it through the normal
    // event-frame path. Proves the dispatcher routes WS-published
    // events to *other* connections, not just the publisher's own.
    const subEvents = [];
    const subscriber = makeClient();
    await subscriber.subscribe(STREAM, "ts-cross", (e) => subEvents.push(e));
    await new Promise((r) => setTimeout(r, 200));

    const publisher = makeClient();
    await publisher.publish(STREAM, "ts-cross", {
      who: "publisher",
      marker: "cross",
    });

    const ev = await waitForEvent(subEvents, (e) => e.key === "ts-cross");
    assert.equal(ev.payload.who, "publisher");

    await publisher.close();
    await subscriber.close();
  },
);
