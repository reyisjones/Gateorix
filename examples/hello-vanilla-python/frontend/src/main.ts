import "./styles.css";
import { invokeBackend, isTauri } from "./lib/ipc";

const badge = document.getElementById("ipc-badge")!;
badge.textContent = `IPC mode: ${isTauri ? "Tauri (native)" : "HTTP dev bridge"}`;

const input = document.getElementById("name") as HTMLInputElement;
const button = document.getElementById("greet-btn") as HTMLButtonElement;
const error = document.getElementById("error") as HTMLParagraphElement;
const result = document.getElementById("result") as HTMLParagraphElement;

async function handleGreet() {
  button.disabled = true;
  button.textContent = "Loading…";
  error.hidden = true;
  result.hidden = true;

  try {
    const data = await invokeBackend("runtime.greet", { name: input.value || "World" });
    if (data.ok) {
      result.textContent = (data.payload as { message: string }).message;
      result.hidden = false;
    } else {
      error.textContent = (data.payload as { error?: string })?.error || "unknown error";
      error.hidden = false;
    }
  } catch (err) {
    error.textContent = isTauri
      ? `Sidecar error: ${err}`
      : "Cannot reach backend. Is the dev bridge running?";
    error.hidden = false;
  } finally {
    button.disabled = false;
    button.textContent = "Greet";
  }
}

button.addEventListener("click", handleGreet);
input.addEventListener("keydown", (e) => {
  if (e.key === "Enter") handleGreet();
});
