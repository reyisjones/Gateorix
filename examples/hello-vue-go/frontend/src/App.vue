<script setup lang="ts">
import { ref } from "vue";
import { invokeBackend, isTauri } from "./lib/ipc";

const name = ref("");
const greeting = ref("");
const error = ref("");
const loading = ref(false);

async function handleGreet() {
  loading.value = true;
  error.value = "";
  greeting.value = "";
  try {
    const data = await invokeBackend("runtime.greet", { name: name.value || "World" });
    if (data.ok) {
      greeting.value = (data.payload as { message: string }).message;
    } else {
      error.value = (data.payload as { error?: string })?.error || "unknown error";
    }
  } catch (err) {
    error.value = isTauri
      ? `Sidecar error: ${err}`
      : "Cannot reach backend. Is the dev bridge running?";
  } finally {
    loading.value = false;
  }
}
</script>

<template>
  <main class="main-content">
    <h1 class="page-title">Hello Gateorix</h1>
    <p class="page-subtitle">A Vue 3 frontend talking to the Gateorix runtime.</p>
    <p class="ipc-badge">IPC mode: {{ isTauri ? "Tauri (native)" : "HTTP dev bridge" }}</p>

    <div class="input-row">
      <input
        v-model="name"
        class="form-input"
        type="text"
        placeholder="Your name"
        @keydown.enter="handleGreet"
      />
      <button class="btn btn-primary" :disabled="loading" @click="handleGreet">
        {{ loading ? "Loading…" : "Greet" }}
      </button>
    </div>

    <p v-if="error" class="error-msg">{{ error }}</p>
    <p v-if="greeting" class="greeting-result">{{ greeting }}</p>
  </main>
</template>
