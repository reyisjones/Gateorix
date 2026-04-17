/**
 * IPC type definitions for the Gateorix bridge protocol.
 */

/** An IPC request sent from the frontend to the host core. */
export interface IpcRequest {
  id: string;
  channel: string;
  payload: unknown;
}

/** An IPC response returned from the host core. */
export interface IpcResponse {
  id: string;
  ok: boolean;
  payload: unknown;
}

/** An event pushed from the host core to the frontend. */
export interface IpcEvent {
  event: string;
  payload: unknown;
}

/** Options for the invoke() call. */
export interface InvokeOptions {
  /** Timeout in milliseconds. 0 means no timeout. */
  timeout?: number;
}
