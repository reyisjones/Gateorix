import React, { useState } from "react";
import ReactDOM from "react-dom/client";

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
      // Call the Python backend via the HTTP dev bridge
      const res = await fetch("http://localhost:3001/invoke", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          id: crypto.randomUUID(),
          channel: "runtime.greet",
          payload: { name: name || "World" },
        }),
      });
      const data = await res.json();

      if (data.ok) {
        setGreeting(data.payload.message);
      } else {
        setError(data.payload?.error || "unknown error");
      }
    } catch (err) {
      // Network error — bridge is probably not running
      setError("Cannot reach backend. Is the bridge running? (python main.py --http)");
    } finally {
      setLoading(false);
    }
  };

  return (
    <div style={{ fontFamily: "system-ui", padding: "2rem", maxWidth: "600px", margin: "0 auto" }}>
      <h1>Hello Gateorix</h1>
      <p>A React frontend + Python backend desktop app.</p>

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
