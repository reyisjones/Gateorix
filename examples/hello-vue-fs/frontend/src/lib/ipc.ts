// IPC helpers for Gateorix: Tauri sidecar in production, HTTP bridge in dev.

export const isTauri = "__TAURI_INTERNALS__" in window;

export interface IpcResult {
  ok: boolean;
  payload: Record<string, unknown>;
}

export async function invokeBackend(
  channel: string,
  payload: Record<string, unknown>,
): Promise<IpcResult> {
  if (isTauri) {
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke<IpcResult>("invoke_backend", { channel, payload });
  }

  const res = await fetch("http://localhost:3001/invoke", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ id: crypto.randomUUID(), channel, payload }),
  });
  return res.json();
}
