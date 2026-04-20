import React, { useEffect, useState } from "react";
import ReactDOM from "react-dom/client";
import "./styles.css";

const isTauri = "__TAURI_INTERNALS__" in window;

async function tauriInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<T>(cmd, args);
}

async function invokeBackend(channel: string, payload: Record<string, unknown>) {
  if (isTauri) {
    return tauriInvoke<{ ok: boolean; payload: Record<string, unknown> }>("invoke_backend", { channel, payload });
  } else {
    const res = await fetch("http://localhost:3001/invoke", {
      method: "POST", headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ id: crypto.randomUUID(), channel, payload }),
    });
    return res.json();
  }
}

type Page = "home" | "profile" | "login";

function Navbar({ page, setPage, theme, toggleTheme, user, onLogout }: {
  page: Page; setPage: (p: Page) => void; theme: string; toggleTheme: () => void;
  user: string | null; onLogout: () => void;
}) {
  return (
    <nav className="navbar">
      <div className="navbar-left">
        <span className="navbar-logo" onClick={() => setPage("home")} style={{ cursor: "pointer" }}>⚡ Gateorix</span>
      </div>
      <div className="navbar-right">
        {page !== "home" && <button className="nav-btn" onClick={() => setPage("home")}>Home</button>}
        <button className="nav-btn" onClick={toggleTheme} title="Toggle theme">{theme === "dark" ? "☀️" : "🌙"}</button>
        {user && <button className={`nav-btn ${page === "profile" ? "active" : ""}`} onClick={() => setPage("profile")}>Profile</button>}
        {user ? (
          <><span className="nav-user">Welcome, {user}</span><button className="nav-btn" onClick={onLogout}>Logout</button></>
        ) : (
          <button className={`nav-btn ${page === "login" ? "active" : ""}`} onClick={() => setPage("login")}>Login</button>
        )}
      </div>
    </nav>
  );
}

function HomePage() {
  const [greeting, setGreeting] = useState("");
  const [name, setName] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");

  const handleGreet = async () => {
    setLoading(true); setError(""); setGreeting("");
    try {
      const data = await invokeBackend("runtime.greet", { name: name || "World" });
      if (data.ok) setGreeting((data.payload as { message: string }).message);
      else setError((data.payload as { error?: string })?.error || "unknown error");
    } catch (err) {
      setError(isTauri ? `Sidecar error: ${err}` : "Cannot reach backend. Is the C++ bridge running? (./hello_gateorix_cpp --http)");
    } finally { setLoading(false); }
  };

  return (
    <div className="main-content">
      <h1 className="page-title">Hello Gateorix</h1>
      <p className="page-subtitle">A React frontend + C++ backend desktop app.</p>
      <p className="ipc-badge">IPC mode: {isTauri ? "Tauri (native)" : "HTTP dev bridge"}</p>
      <div className="input-row">
        <input className="form-input" type="text" placeholder="Your name" value={name}
          onChange={(e) => setName(e.target.value)} onKeyDown={(e) => e.key === "Enter" && handleGreet()} />
        <button className="btn btn-primary" onClick={handleGreet} disabled={loading}>{loading ? "Loading…" : "Greet"}</button>
      </div>
      {error && <p className="error-msg">{error}</p>}
      {greeting && <p className="greeting-result">{greeting}</p>}
    </div>
  );
}

function ProfilePage({ onBack }: { onBack: () => void }) {
  const [displayName, setDisplayName] = useState("");
  const [email, setEmail] = useState("");
  const [saving, setSaving] = useState(false);
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    if (!isTauri) return;
    tauriInvoke<Record<string, unknown>>("get_settings").then((s) => {
      if (s.profileName) setDisplayName(s.profileName as string);
      if (s.profileEmail) setEmail(s.profileEmail as string);
    });
  }, []);

  const handleSave = async () => {
    if (!isTauri) return;
    setSaving(true); setSaved(false);
    try { await tauriInvoke("save_settings", { data: { profileName: displayName, profileEmail: email } }); setSaved(true); }
    catch (err) { console.error("Save failed:", err); }
    finally { setSaving(false); }
  };

  return (
    <div className="main-content">
      <button className="back-btn" onClick={onBack}>← Back</button>
      <h1 className="page-title">Profile</h1>
      <p className="page-subtitle">Edit your profile. Data is saved to disk via Tauri.</p>
      <div className="avatar">👤</div>
      <div className="form-group"><label className="form-label">Display Name</label>
        <input className="form-input" type="text" placeholder="Your name" value={displayName} onChange={(e) => setDisplayName(e.target.value)} /></div>
      <div className="form-group"><label className="form-label">Email</label>
        <input className="form-input" type="email" placeholder="you@example.com" value={email} onChange={(e) => setEmail(e.target.value)} /></div>
      <button className="btn btn-primary" onClick={handleSave} disabled={saving}>{saving ? "Saving…" : "Save Profile"}</button>
      {saved && <p className="success-msg">✓ Profile saved successfully.</p>}
      {!isTauri && <p className="error-msg">Profile save requires Tauri (native mode).</p>}
    </div>
  );
}

function LoginPage({ onLogin, onBack }: { onLogin: (name: string) => void; onBack: () => void }) {
  const [username, setUsername] = useState(""); const [password, setPassword] = useState("");
  const [error, setError] = useState(""); const [loading, setLoading] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault(); setError(""); setLoading(true);
    try {
      if (isTauri) { onLogin(await tauriInvoke<string>("login", { username, password })); }
      else if (username === "admin" && password === "gateorix") onLogin("Admin");
      else if (username === "demo" && password === "demo") onLogin("Demo User");
      else throw new Error("Invalid username or password");
    } catch (err) { setError(String(err)); }
    finally { setLoading(false); }
  };

  return (
    <div className="main-content">
      <button className="back-btn" onClick={onBack}>← Back</button>
      <h1 className="page-title">Login</h1>
      <p className="page-subtitle">Demo credentials: <code>admin / gateorix</code> or <code>demo / demo</code></p>
      <form onSubmit={handleSubmit}>
        <div className="form-group"><label className="form-label">Username</label>
          <input className="form-input" type="text" placeholder="Username" value={username} onChange={(e) => setUsername(e.target.value)} autoFocus /></div>
        <div className="form-group"><label className="form-label">Password</label>
          <input className="form-input" type="password" placeholder="Password" value={password} onChange={(e) => setPassword(e.target.value)} /></div>
        <button className="btn btn-primary" type="submit" disabled={loading}>{loading ? "Logging in…" : "Login"}</button>
        {error && <p className="error-msg">{error}</p>}
      </form>
    </div>
  );
}

function App() {
  const [page, setPage] = useState<Page>("home");
  const [theme, setTheme] = useState<string>("light");
  const [user, setUser] = useState<string | null>(null);

  useEffect(() => {
    if (!isTauri) return;
    tauriInvoke<Record<string, unknown>>("get_settings").then((s) => {
      if (s.theme === "dark" || s.theme === "light") { setTheme(s.theme as string); document.documentElement.setAttribute("data-theme", s.theme as string); }
    });
  }, []);

  const toggleTheme = async () => {
    const next = theme === "dark" ? "light" : "dark";
    setTheme(next); document.documentElement.setAttribute("data-theme", next);
    if (isTauri) await tauriInvoke("save_settings", { data: { theme: next } });
  };

  return (
    <>
      <Navbar page={page} setPage={setPage} theme={theme} toggleTheme={toggleTheme} user={user}
        onLogout={async () => { if (isTauri) await tauriInvoke("logout"); setUser(null); setPage("home"); }} />
      {page === "home" && <HomePage />}
      {page === "profile" && <ProfilePage onBack={() => setPage("home")} />}
      {page === "login" && <LoginPage onLogin={(n) => { setUser(n); setPage("home"); }} onBack={() => setPage("home")} />}
    </>
  );
}

ReactDOM.createRoot(document.getElementById("root")!).render(<React.StrictMode><App /></React.StrictMode>);
