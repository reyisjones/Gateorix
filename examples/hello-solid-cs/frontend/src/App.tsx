import { createSignal, Show } from "solid-js";
import { invokeBackend, isTauri } from "./lib/ipc";

export default function App() {
  const [name, setName] = createSignal("");
  const [greeting, setGreeting] = createSignal("");
  const [error, setError] = createSignal("");
  const [loading, setLoading] = createSignal(false);

  const handleGreet = async () => {
    setLoading(true);
    setError("");
    setGreeting("");
    try {
      const data = await invokeBackend("runtime.greet", { name: name() || "World" });
      if (data.ok) {
        setGreeting((data.payload as { message: string }).message);
      } else {
        setError((data.payload as { error?: string })?.error || "unknown error");
      }
    } catch (err) {
      setError(
        isTauri
          ? `Sidecar error: ${err}`
          : "Cannot reach backend. Is the dev bridge running?",
      );
    } finally {
      setLoading(false);
    }
  };

  return (
    <main class="main-content">
      <h1 class="page-title">Hello Gateorix</h1>
      <p class="page-subtitle">A SolidJS frontend talking to the Gateorix runtime.</p>
      <p class="ipc-badge">IPC mode: {isTauri ? "Tauri (native)" : "HTTP dev bridge"}</p>

      <div class="input-row">
        <input
          class="form-input"
          type="text"
          placeholder="Your name"
          value={name()}
          onInput={(e) => setName(e.currentTarget.value)}
          onKeyDown={(e) => e.key === "Enter" && handleGreet()}
        />
        <button class="btn btn-primary" disabled={loading()} onClick={handleGreet}>
          {loading() ? "Loading…" : "Greet"}
        </button>
      </div>

      <Show when={error()}>
        <p class="error-msg">{error()}</p>
      </Show>
      <Show when={greeting()}>
        <p class="greeting-result">{greeting()}</p>
      </Show>
    </main>
  );
}
