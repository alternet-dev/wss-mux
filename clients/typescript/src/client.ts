// WssMuxClient: a minimal, dependency-free browser WebSocket client for
// the wss-mux protocol (v0.5 wire). See docs/protocol.md in the wss-mux
// repo for the authoritative wire spec.
//
// Design notes:
// - No runtime dependencies. Uses globalThis.WebSocket by default; the
//   constructor accepts a custom WebSocket ctor for tests / non-browser use.
// - Reconnects automatically with exponential backoff. Token refresh is
//   triggered on the initial connect and on close code 4401, per the
//   issue scope (#54).
// - Subscriptions are tracked locally and replayed on reconnect. The wire
//   protocol acks subscribe by absence: we add to the local map and send;
//   any error frame for that subscription id removes it from the map and
//   is reported via opts.onError.

import {
  CLOSE_BAD_FRAME,
  CLOSE_UNAUTHENTICATED,
  SUBPROTOCOL_JSON,
} from "./types.js";
import type {
  AuthFrame,
  EventFrame,
  ServerFrame,
  SubscribeFrame,
  SubscriptionId,
  UnsubscribeFrame,
} from "./types.js";
import {
  ClientUsageError,
  ConnectionClosedError,
  ProtocolError,
  WssMuxError,
} from "./errors.js";

export type SubscriptionCallback = (event: EventFrame) => void;
export type ErrorCallback = (error: ProtocolError) => void;
export type StateCallback = (state: ConnectionState) => void;

export type ConnectionState =
  | "idle"
  | "connecting"
  | "authenticating"
  | "ready"
  | "reconnecting"
  | "closing"
  | "closed";

export interface ReconnectOptions {
  /** Cap on reconnect attempts. Default: Infinity. */
  maxAttempts?: number;
  /** Initial backoff in ms. Default: 1000. */
  initialBackoffMs?: number;
  /** Maximum backoff in ms. Default: 30000. */
  maxBackoffMs?: number;
  /** Backoff multiplier per attempt. Default: 2. */
  backoffMultiplier?: number;
}

export interface ClientOptions {
  /** Full WebSocket URL: `wss://host[:port]/path`. */
  wssUrl: string;
  /**
   * Returns the current auth token. Invoked on initial connect and on
   * reconnect after close code 4401 (expired_token). Cached otherwise.
   */
  getToken: () => string | Promise<string>;
  /** WebSocket constructor; defaults to `globalThis.WebSocket`. */
  WebSocket?: typeof WebSocket;
  /** Reconnect behaviour. Defaults below. */
  reconnect?: ReconnectOptions;
  /** Called when the server sends an `error` frame. */
  onError?: ErrorCallback;
  /** Called on connection state transitions. */
  onStateChange?: StateCallback;
}

const DEFAULT_RECONNECT: Required<ReconnectOptions> = {
  maxAttempts: Number.POSITIVE_INFINITY,
  initialBackoffMs: 1000,
  maxBackoffMs: 30000,
  backoffMultiplier: 2,
};

interface ActiveSubscription {
  stream: string;
  key?: string;
  callback: SubscriptionCallback;
}

/** Per-subscription fatal codes that remove the sub from the server side. */
const SUB_FATAL_CODES = new Set([
  "unknown_stream",
  "unauthorized_subscribe",
  "duplicate_subscription_id",
  "overflow",
]);

export class WssMuxClient {
  private readonly opts: ClientOptions;
  private readonly reconnectOpts: Required<ReconnectOptions>;
  private readonly WebSocketCtor: typeof WebSocket;

  private ws: WebSocket | null = null;
  private state: ConnectionState = "idle";

  private readonly subscriptions: Map<SubscriptionId, ActiveSubscription> =
    new Map();
  private nextSubCounter = 1;

  private reconnectAttempt = 0;
  private reconnectTimer: ReturnType<typeof setTimeout> | null = null;

  /** Resolves on the next "ready"; rejects if reconnects are exhausted. */
  private readyDeferred: Deferred<void> | null = null;

  /** Cached token; reused on non-4401 reconnects, refreshed on 4401. */
  private cachedToken: string | null = null;

  constructor(opts: ClientOptions) {
    if (!opts.wssUrl) {
      throw new ClientUsageError("wssUrl is required");
    }
    if (typeof opts.getToken !== "function") {
      throw new ClientUsageError("getToken is required");
    }

    this.opts = opts;
    this.reconnectOpts = {
      ...DEFAULT_RECONNECT,
      ...(opts.reconnect ?? {}),
    };

    const ctor =
      opts.WebSocket ??
      (globalThis as unknown as { WebSocket?: typeof WebSocket }).WebSocket;
    if (!ctor) {
      throw new ClientUsageError(
        "No WebSocket constructor available; provide opts.WebSocket on non-browser hosts",
      );
    }
    this.WebSocketCtor = ctor;
  }

  /** Current connection state. */
  get connectionState(): ConnectionState {
    return this.state;
  }

  /** Number of currently-tracked subscriptions (local view). */
  get subscriptionCount(): number {
    return this.subscriptions.size;
  }

  subscribe(
    stream: string,
    callback: SubscriptionCallback,
  ): Promise<SubscriptionId>;
  subscribe(
    stream: string,
    key: string | undefined,
    callback: SubscriptionCallback,
  ): Promise<SubscriptionId>;
  async subscribe(
    stream: string,
    keyOrCallback: string | undefined | SubscriptionCallback,
    maybeCallback?: SubscriptionCallback,
  ): Promise<SubscriptionId> {
    if (this.state === "closing" || this.state === "closed") {
      throw new ClientUsageError("Client is closed");
    }

    let key: string | undefined;
    let callback: SubscriptionCallback;
    if (typeof keyOrCallback === "function") {
      key = undefined;
      callback = keyOrCallback;
    } else {
      key = keyOrCallback;
      if (typeof maybeCallback !== "function") {
        throw new ClientUsageError("callback is required");
      }
      callback = maybeCallback;
    }

    const id = this.nextId();
    this.subscriptions.set(id, { stream, key, callback });

    if (this.state === "ready") {
      this.send(this.subscribeFrame(id, stream, key));
    } else {
      // ensureReady triggers replay on connect; nothing to send here.
      await this.ensureReady();
    }
    return id;
  }

  async unsubscribe(id: SubscriptionId): Promise<void> {
    if (!this.subscriptions.has(id)) return; // idempotent
    this.subscriptions.delete(id);
    if (this.state === "ready") {
      this.send({ type: "unsubscribe", id });
    }
    // If not ready, the next reconnect's replay simply omits this id. The
    // server's previous connection has been torn down, so no stale sub.
  }

  /** Graceful close. After this call, no more subscribes accepted. */
  async close(): Promise<void> {
    if (this.state === "closed") return;
    this.state = "closing";
    this.emitState();

    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }
    if (this.ws) {
      try {
        this.ws.close(1000, "client close");
      } catch {
        // ignore
      }
      this.ws = null;
    }
    this.subscriptions.clear();

    if (this.readyDeferred) {
      this.readyDeferred.reject(new WssMuxError("client closed"));
      this.readyDeferred = null;
    }

    this.state = "closed";
    this.emitState();
  }

  // --- internals ---

  private nextId(): SubscriptionId {
    return `sub-${this.nextSubCounter++}`;
  }

  private subscribeFrame(
    id: SubscriptionId,
    stream: string,
    key: string | undefined,
  ): SubscribeFrame {
    const frame: SubscribeFrame = { type: "subscribe", id, stream };
    if (key !== undefined) frame.key = key;
    return frame;
  }

  private async ensureReady(): Promise<void> {
    if (this.state === "ready") return;
    if (this.state === "idle") {
      return this.connect(/* refreshToken */ true);
    }
    // connecting / authenticating / reconnecting — wait on the deferred.
    if (!this.readyDeferred) {
      this.readyDeferred = deferred<void>();
    }
    return this.readyDeferred.promise;
  }

  private async connect(refreshToken: boolean): Promise<void> {
    if (refreshToken || this.cachedToken === null) {
      this.cachedToken = await this.opts.getToken();
    }

    this.state = "connecting";
    this.emitState();

    if (!this.readyDeferred) {
      this.readyDeferred = deferred<void>();
    }
    const deferredAtConnect = this.readyDeferred;

    let ws: WebSocket;
    try {
      ws = new this.WebSocketCtor(this.opts.wssUrl, [SUBPROTOCOL_JSON]);
    } catch {
      // Synchronous ctor failure — schedule reconnect.
      this.scheduleReconnect(false);
      return deferredAtConnect.promise;
    }

    this.ws = ws;

    ws.addEventListener("open", () => this.onOpen());
    ws.addEventListener("message", (e) =>
      this.onMessage(e as MessageEvent<string>),
    );
    ws.addEventListener("close", (e) => this.onClose(e as CloseEvent));
    ws.addEventListener("error", () => {
      // Error events precede a close event; close handler does the work.
    });

    return deferredAtConnect.promise;
  }

  private onOpen(): void {
    this.state = "authenticating";
    this.emitState();

    this.send({ type: "auth", token: this.cachedToken ?? "" });

    // Ack-by-absence: assume auth succeeded. A 4401 close arrives if not.
    this.state = "ready";
    this.emitState();
    this.reconnectAttempt = 0;

    // Replay subscriptions over the new connection.
    for (const [id, sub] of this.subscriptions) {
      this.send(this.subscribeFrame(id, sub.stream, sub.key));
    }

    if (this.readyDeferred) {
      this.readyDeferred.resolve();
      this.readyDeferred = null;
    }
  }

  private onMessage(event: MessageEvent<string>): void {
    let frame: ServerFrame;
    try {
      frame = JSON.parse(event.data) as ServerFrame;
    } catch {
      // Server is well-behaved; ignore malformed frames defensively.
      return;
    }

    if (frame.type === "event") {
      const sub = this.subscriptions.get(frame.id);
      if (sub) sub.callback(frame);
      return;
    }

    if (frame.type === "error") {
      if (frame.id && SUB_FATAL_CODES.has(frame.code)) {
        this.subscriptions.delete(frame.id);
      }
      const err = new ProtocolError(frame.code, frame.message, frame.id);
      this.opts.onError?.(err);
    }
  }

  private onClose(event: CloseEvent): void {
    this.ws = null;

    if (this.state === "closing" || this.state === "closed") return;

    // 4400 is a protocol bug on our side; don't loop reconnects.
    if (event.code === CLOSE_BAD_FRAME) {
      this.state = "closed";
      this.emitState();
      if (this.readyDeferred) {
        this.readyDeferred.reject(
          new ConnectionClosedError(event.code, event.reason),
        );
        this.readyDeferred = null;
      }
      return;
    }

    const refreshToken = event.code === CLOSE_UNAUTHENTICATED;
    this.scheduleReconnect(refreshToken);
  }

  private scheduleReconnect(refreshToken: boolean): void {
    if (this.reconnectAttempt >= this.reconnectOpts.maxAttempts) {
      this.state = "closed";
      this.emitState();
      if (this.readyDeferred) {
        this.readyDeferred.reject(
          new WssMuxError("reconnect attempts exhausted"),
        );
        this.readyDeferred = null;
      }
      return;
    }

    this.state = "reconnecting";
    this.emitState();

    const delayMs = Math.min(
      this.reconnectOpts.initialBackoffMs *
        Math.pow(
          this.reconnectOpts.backoffMultiplier,
          this.reconnectAttempt,
        ),
      this.reconnectOpts.maxBackoffMs,
    );
    this.reconnectAttempt += 1;

    if (!this.readyDeferred) {
      this.readyDeferred = deferred<void>();
    }

    this.reconnectTimer = setTimeout(() => {
      this.reconnectTimer = null;
      void this.connect(refreshToken).catch(() => {
        // Errors surface via onClose → scheduleReconnect.
      });
    }, delayMs);
  }

  private send(frame: AuthFrame | SubscribeFrame | UnsubscribeFrame): void {
    const ws = this.ws;
    if (!ws || ws.readyState !== ws.OPEN) return;
    ws.send(JSON.stringify(frame));
  }

  private emitState(): void {
    this.opts.onStateChange?.(this.state);
  }
}

interface Deferred<T> {
  promise: Promise<T>;
  resolve: (v: T) => void;
  reject: (e: unknown) => void;
}

function deferred<T>(): Deferred<T> {
  let resolve!: (v: T) => void;
  let reject!: (e: unknown) => void;
  const promise = new Promise<T>((res, rej) => {
    resolve = res;
    reject = rej;
  });
  return { promise, resolve, reject };
}
