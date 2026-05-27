import type { ErrorCode, SubscriptionId } from "./types.js";

/** Base class for all wss-mux client errors. */
export class WssMuxError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "WssMuxError";
  }
}

/**
 * Server returned an `error` frame.
 *
 * For keep-open codes (`rate_limited`, plus per-subscription codes like
 * `unknown_stream`, `unauthorized_subscribe`, `overflow`,
 * `duplicate_subscription_id`), the connection stays open. For fatal codes
 * (`bad_frame`, `unauthenticated`, `expired_token`, `unknown_frame_type`)
 * the server closes the connection.
 */
export class ProtocolError extends WssMuxError {
  constructor(
    public readonly code: ErrorCode,
    message: string,
    public readonly subscriptionId?: SubscriptionId,
  ) {
    super(message);
    this.name = "ProtocolError";
  }
}

/** The WebSocket connection closed unexpectedly. */
export class ConnectionClosedError extends WssMuxError {
  constructor(
    public readonly code: number,
    public readonly reason: string,
  ) {
    super(`WebSocket closed: code=${code} reason=${reason || "(none)"}`);
    this.name = "ConnectionClosedError";
  }
}

/** Invalid use of the client API. */
export class ClientUsageError extends WssMuxError {
  constructor(message: string) {
    super(message);
    this.name = "ClientUsageError";
  }
}
