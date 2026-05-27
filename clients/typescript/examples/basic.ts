// Minimal browser usage example for @wss-mux/client.
//
// Run with a bundler (vite, webpack, esbuild) or directly in a browser
// environment that supports ES modules.

import { WssMuxClient, ProtocolError } from "@wss-mux/client";

async function main() {
  // The token-source the SDK calls. Provide your OIDC access-token
  // getter here; the SDK invokes it on initial connect and on 4401.
  const getToken = async (): Promise<string> => {
    // Example: read from your auth provider.
    // const token = await myAuthProvider.getAccessToken();
    // return token;
    return "your-jwt-here";
  };

  const client = new WssMuxClient({
    // Whatever path your wss-mux deployment serves the WebSocket on; the
    // handshake response from your auth endpoint is the canonical source.
    wssUrl: "wss://realtime.example.com/stream",
    getToken,
    onError: (err: ProtocolError) => {
      console.warn("server error:", err.code, err.message, err.subscriptionId);
    },
    onStateChange: (state) => {
      console.log("connection state:", state);
    },
  });

  // Subscribe to a stream with an optional key narrow.
  const sid = await client.subscribe("notification_banner", "room-42", (event) => {
    console.log("event:", event.payload);
  });

  // Later: unsubscribe.
  // await client.unsubscribe(sid);

  // On teardown: graceful close.
  // await client.close();

  return sid;
}

void main();
