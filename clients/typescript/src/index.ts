export { WssMuxClient } from "./client.js";
export type {
  ClientOptions,
  ConnectionState,
  ErrorCallback,
  ReconnectOptions,
  StateCallback,
  SubscriptionCallback,
} from "./client.js";

export {
  WssMuxError,
  ProtocolError,
  ConnectionClosedError,
  ClientUsageError,
} from "./errors.js";

export type {
  AuthFrame,
  ClientFrame,
  ErrorCode,
  ErrorFrame,
  EventFrame,
  ServerFrame,
  StreamName,
  SubscribeFrame,
  SubscriptionId,
  UnsubscribeFrame,
} from "./types.js";

export {
  CLOSE_BAD_FRAME,
  CLOSE_NORMAL,
  CLOSE_UNAUTHENTICATED,
} from "./types.js";

// SUBPROTOCOL_JSON is intentionally not re-exported; the SDK negotiates
// the wire subprotocol internally and consumers should not need to
// reference it directly.
