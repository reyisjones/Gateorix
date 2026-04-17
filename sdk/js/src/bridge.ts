/**
 * GateorixBridge — the main API for frontend ↔ host communication.
 *
 * Usage:
 *   import { GateorixBridge } from '@gateorix/bridge';
 *   const bridge = new GateorixBridge();
 *   const result = await bridge.invoke('filesystem.readText', { path: './data/notes.txt' });
 */

import type { IpcRequest, IpcResponse, IpcEvent, InvokeOptions } from "./types.js";

type EventCallback = (event: IpcEvent) => void;

let requestCounter = 0;

function generateId(): string {
  return `req-${++requestCounter}-${Date.now()}`;
}

/**
 * The bridge client used in the frontend to invoke host commands,
 * listen for events, and communicate with runtime adapters.
 */
export class GateorixBridge {
  private eventListeners: Map<string, Set<EventCallback>> = new Map();
  private pendingRequests: Map<string, {
    resolve: (res: IpcResponse) => void;
    reject: (err: Error) => void;
  }> = new Map();

  constructor() {
    // In a real Tauri-backed implementation, this would register
    // a global message handler from the webview IPC channel.
    this.setupMessageHandler();
  }

  /**
   * Invoke a host command and wait for the response.
   *
   * @param channel  The target channel (e.g. "filesystem.readText")
   * @param payload  The request payload
   * @param options  Optional invoke settings
   */
  async invoke<T = unknown>(
    channel: string,
    payload: unknown = {},
    options: InvokeOptions = {}
  ): Promise<T> {
    const id = generateId();
    const request: IpcRequest = { id, channel, payload };

    return new Promise<T>((resolve, reject) => {
      // Set up timeout
      let timer: ReturnType<typeof setTimeout> | undefined;
      if (options.timeout && options.timeout > 0) {
        timer = setTimeout(() => {
          this.pendingRequests.delete(id);
          reject(new Error(`invoke "${channel}" timed out after ${options.timeout}ms`));
        }, options.timeout);
      }

      this.pendingRequests.set(id, {
        resolve: (res: IpcResponse) => {
          if (timer) clearTimeout(timer);
          if (res.ok) {
            resolve(res.payload as T);
          } else {
            reject(new Error(JSON.stringify(res.payload)));
          }
        },
        reject: (err: Error) => {
          if (timer) clearTimeout(timer);
          reject(err);
        },
      });

      // Send the request to the host core via the webview bridge.
      // In a Tauri environment this would call window.__TAURI_INTERNALS__.invoke()
      this.postMessage(request);
    });
  }

  /**
   * Listen for events from the host core.
   */
  on(event: string, callback: EventCallback): () => void {
    if (!this.eventListeners.has(event)) {
      this.eventListeners.set(event, new Set());
    }
    this.eventListeners.get(event)!.add(callback);

    // Return an unsubscribe function
    return () => {
      this.eventListeners.get(event)?.delete(callback);
    };
  }

  /**
   * Post a message to the host core.
   * Placeholder — actual implementation depends on the webview runtime.
   */
  private postMessage(request: IpcRequest): void {
    // In production, this sends through Tauri's IPC channel:
    // window.__TAURI_INTERNALS__.invoke('gateorix_ipc', { request })
    if (typeof window !== "undefined" && (window as any).__GATEORIX_IPC__) {
      (window as any).__GATEORIX_IPC__.postMessage(JSON.stringify(request));
    }
  }

  /**
   * Set up the handler for messages coming back from the host core.
   */
  private setupMessageHandler(): void {
    if (typeof window === "undefined") return;

    (window as any).__GATEORIX_RECEIVE__ = (raw: string) => {
      try {
        const message = JSON.parse(raw);

        // Check if it's a response to a pending request
        if (message.id && this.pendingRequests.has(message.id)) {
          const pending = this.pendingRequests.get(message.id)!;
          this.pendingRequests.delete(message.id);
          pending.resolve(message as IpcResponse);
          return;
        }

        // Otherwise treat as an event
        if (message.event) {
          const listeners = this.eventListeners.get(message.event);
          if (listeners) {
            for (const cb of listeners) {
              cb(message as IpcEvent);
            }
          }
        }
      } catch {
        // Ignore malformed messages
      }
    };
  }
}
