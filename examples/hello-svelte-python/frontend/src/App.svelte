<script lang="ts">
  import { invokeBackend, isTauri } from "./lib/ipc";

  let name = "";
  let greeting = "";
  let error = "";
  let loading = false;

  async function handleGreet() {
    loading = true;
    error = "";
    greeting = "";
    try {
      const data = await invokeBackend("runtime.greet", { name: name || "World" });
      if (data.ok) {
        greeting = (data.payload as { message: string }).message;
      } else {
        error = (data.payload as { error?: string })?.error || "unknown error";
      }
    } catch (err) {
      error = isTauri
        ? `Sidecar error: ${err}`
        : "Cannot reach backend. Is the dev bridge running?";
    } finally {
      loading = false;
    }
  }
</script>

<main class="main-content">
  <h1 class="page-title">Hello Gateorix</h1>
  <p class="page-subtitle">A Svelte frontend talking to the Gateorix runtime.</p>
  <p class="ipc-badge">IPC mode: {isTauri ? "Tauri (native)" : "HTTP dev bridge"}</p>

  <div class="input-row">
    <input
      bind:value={name}
      class="form-input"
      type="text"
      placeholder="Your name"
      on:keydown={(e) => e.key === "Enter" && handleGreet()}
    />
    <button class="btn btn-primary" disabled={loading} on:click={handleGreet}>
      {loading ? "Loading…" : "Greet"}
    </button>
  </div>

  {#if error}
    <p class="error-msg">{error}</p>
  {/if}
  {#if greeting}
    <p class="greeting-result">{greeting}</p>
  {/if}
</main>
