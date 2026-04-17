import React, { useState } from "react";
import ReactDOM from "react-dom/client";

// In a real Gateorix app, you would import from @gateorix/bridge:
// import { GateorixBridge } from "@gateorix/bridge";
// const bridge = new GateorixBridge();

function App() {
  const [greeting, setGreeting] = useState("");
  const [name, setName] = useState("");

  const handleGreet = async () => {
    // In production, this calls the Python backend through the bridge:
    // const result = await bridge.invoke("runtime.greet", { name });
    // setGreeting(result.message);

    // Placeholder for template demonstration:
    setGreeting(`Hello, ${name || "World"}! (from frontend)`);
  };

  return (
    <div style={{ fontFamily: "system-ui", padding: "2rem", maxWidth: "600px", margin: "0 auto" }}>
      <h1>🚀 Gateorix</h1>
      <p>Web UI. Native power.</p>

      <div style={{ marginTop: "2rem" }}>
        <input
          type="text"
          placeholder="Enter your name"
          value={name}
          onChange={(e) => setName(e.target.value)}
          style={{ padding: "0.5rem", fontSize: "1rem", marginRight: "0.5rem" }}
        />
        <button onClick={handleGreet} style={{ padding: "0.5rem 1rem", fontSize: "1rem" }}>
          Greet
        </button>
      </div>

      {greeting && (
        <p style={{ marginTop: "1rem", fontSize: "1.25rem", color: "#2563eb" }}>
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
