import React, { useState } from "react";
import ReactDOM from "react-dom/client";

// Detect if we're running inside a Tauri webview
const isTauri = "__TAURI_INTERNALS__" in window;

// Dynamically import Tauri invoke only when running in Tauri
async function invokeBackend(channel: string, payload: Record<string, unknown>) {
  if (isTauri) {
    // Tauri IPC — calls the Rust invoke_backend command which relays to Python sidecar
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke<{ ok: boolean; payload: Record<string, unknown> }>("invoke_backend", {
      channel,
      payload,
    });
  } else {
    // HTTP dev bridge fallback — for browser development without Tauri
    const res = await fetch("http://localhost:3001/invoke", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        id: crypto.randomUUID(),
        channel,
        payload,
      }),
    });
    return res.json();
  }
}

function App() {
  const [greeting, setGreeting] = useState("");
  const [name, setName] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");

  const handleGreet = async () => {
    setLoading(true);
    setError("");
    setGreeting("");

    try {
      const data = await invokeBackend("runtime.greet", { name: name || "World" });

      if (data.ok) {
        setGreeting((data.payload as { message: string }).message);
      } else {
        setError((data.payload as { error?: string })?.error || "unknown error");
      }
    } catch (err) {
      setError(
        isTauri
          ? `Sidecar error: ${err}`
          : "Cannot reach backend. Is the bridge running? (python main.py --http)"
      );
    } finally {
      setLoading(false);
    }
  };

  return (
    <div style={{ fontFamily: "system-ui", padding: "2rem", maxWidth: "600px", margin: "0 auto" }}>
      <h1>Hello Gateorix</h1>
      <p>A React frontend + Python backend desktop app.</p>
      <p style={{ fontSize: "0.8rem", color: "#6b7280" }}>
        IPC mode: {isTauri ? "Tauri (native)" : "HTTP dev bridge"}
      </p>

      <div style={{ marginTop: "2rem" }}>
        <input
          type="text"
          placeholder="Your name"
          value={name}
          onChange={(e) => setName(e.target.value)}
          style={{ padding: "0.5rem", fontSize: "1rem", marginRight: "0.5rem", borderRadius: "4px", border: "1px solid #ccc" }}
        />
        <button
          onClick={handleGreet}
          disabled={loading}
          style={{ padding: "0.5rem 1rem", fontSize: "1rem", borderRadius: "4px", background: loading ? "#93c5fd" : "#2563eb", color: "#fff", border: "none", cursor: loading ? "wait" : "pointer" }}
        >
          {loading ? "Loading…" : "Greet"}
        </button>
      </div>

      {error && (
        <p style={{ marginTop: "1.5rem", fontSize: "1rem", color: "#dc2626", background: "#fef2f2", padding: "0.75rem 1rem", borderRadius: "4px", border: "1px solid #fecaca" }}>
          {error}
        </p>
      )}

      {greeting && (
        <p style={{ marginTop: "1.5rem", fontSize: "1.25rem", color: "#2563eb" }}>
          {greeting}
        </p>
      )}
    </div>
  );
}

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
