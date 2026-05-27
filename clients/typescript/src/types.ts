// Frame types and error codes defined by the wss-mux protocol.
// Authoritative source: docs/protocol.md in the wss-mux repo.

export type StreamName = string;
export type SubscriptionId = string;

/** Client → server: first frame on every connection. */
export interface AuthFrame {
  type: "auth";
  token: string;
}

/** Client → server: bind a subscription within the connection. */
export interface SubscribeFrame {
  type: "subscribe";
  id: SubscriptionId;
  stream: StreamName;
  /** Omit to subscribe to all events on the stream. */
  key?: string;
}

/** Client → server: remove a subscription. Idempotent. */
export interface UnsubscribeFrame {
  type: "unsubscribe";
  id: SubscriptionId;
}

/** Server → client: an event matched a subscription. */
export interface EventFrame {
  type: "event";
  id: SubscriptionId;
  stream: StreamName;
  key?: string;
  /** Forwarded verbatim from the producer. */
  payload: unknown;
}

/** Server → client: reports a problem. */
export interface ErrorFrame {
  type: "error";
  code: ErrorCode;
  message: string;
  /** Present when the error pertains to a specific frame. */
  id?: SubscriptionId;
}

export type ClientFrame = AuthFrame | SubscribeFrame | UnsubscribeFrame;
export type ServerFrame = EventFrame | ErrorFrame;

/** Per docs/protocol.md §"Error codes". */
export type ErrorCode =
  | "unknown_frame_type"
  | "bad_frame"
  | "unauthenticated"
  | "expired_token"
  | "unknown_stream"
  | "unauthorized_subscribe"
  | "duplicate_subscription_id"
  | "rate_limited"
  | "overflow";

/** WebSocket close codes used by wss-mux. */
export const CLOSE_NORMAL = 1000;
export const CLOSE_BAD_FRAME = 4400;
export const CLOSE_UNAUTHENTICATED = 4401;

/** The JSON subprotocol identifier negotiated on upgrade. */
export const SUBPROTOCOL_JSON = "wss-mux";
