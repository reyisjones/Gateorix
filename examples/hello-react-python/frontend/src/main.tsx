import React, { useState } from "react";
import ReactDOM from "react-dom/client";

function App() {
  const [greeting, setGreeting] = useState("");
  const [name, setName] = useState("");

  const handleGreet = async () => {
    // Once the bridge is wired up:
    // import { GateorixBridge } from '@gateorix/bridge';
    // const bridge = new GateorixBridge();
    // const result = await bridge.invoke<{ message: string }>('runtime.greet', { name });
    // setGreeting(result.message);

    setGreeting(`Hello, ${name || "World"}! Welcome to Gateorix.`);
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
          style={{ padding: "0.5rem 1rem", fontSize: "1rem", borderRadius: "4px", background: "#2563eb", color: "#fff", border: "none", cursor: "pointer" }}
        >
          Greet
        </button>
      </div>

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
