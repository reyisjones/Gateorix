"""
Hello Gateorix — Python backend example.

Run modes:
  HTTP server (dev bridge):
    python main.py --http
    → starts on http://localhost:3001

  Stdio sidecar (production):
    echo '{"id":"1","channel":"runtime.greet","payload":{"name":"Alice"}}' | python main.py
"""

import json
import sys
from http.server import HTTPServer, BaseHTTPRequestHandler


# --- Command handlers ---

def handle_greet(payload: dict) -> dict:
    name = payload.get("name", "World")
    return {"message": f"Hello, {name}! Welcome to Gateorix."}


def handle_echo(payload: dict) -> dict:
    return payload


HANDLERS = {
    "greet": handle_greet,
    "echo": handle_echo,
}


def dispatch(request: dict) -> dict:
    """Route a request to the appropriate handler and return a response."""
    req_id = request.get("id", "unknown")
    channel = request.get("channel", "")
    payload = request.get("payload", {})

    # Extract action from channel (e.g. "runtime.greet" → "greet")
    action = channel.split(".")[-1] if "." in channel else channel
    handler = HANDLERS.get(action)

    if handler is None:
        return {"id": req_id, "ok": False, "payload": {"error": f"unknown command: {action}"}}

    try:
        result = handler(payload)
        return {"id": req_id, "ok": True, "payload": result}
    except Exception as exc:
        return {"id": req_id, "ok": False, "payload": {"error": str(exc)}}


# --- HTTP dev bridge server ---

class BridgeHandler(BaseHTTPRequestHandler):
    """Handles POST /invoke requests from the frontend dev bridge."""

    def do_GET(self):
        # Friendly response when someone opens the URL in a browser
        self._send_json(200, {
            "status": "running",
            "usage": "POST /invoke with JSON body: {id, channel, payload}",
            "example": {"id": "1", "channel": "runtime.greet", "payload": {"name": "World"}},
        })

    def do_POST(self):
        if self.path != "/invoke":
            self.send_response(404)
            self.end_headers()
            return

        content_length = int(self.headers.get("Content-Length", 0))
        body = self.rfile.read(content_length)

        try:
            request = json.loads(body)
        except json.JSONDecodeError:
            self._send_json(400, {"ok": False, "payload": {"error": "invalid JSON"}})
            return

        response = dispatch(request)
        self._send_json(200, response)

    def do_OPTIONS(self):
        # CORS preflight for frontend dev server on a different port
        self.send_response(204)
        self._cors_headers()
        self.end_headers()

    def _send_json(self, status: int, data: dict):
        body = json.dumps(data).encode()
        self.send_response(status)
        self._cors_headers()
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def _cors_headers(self):
        self.send_header("Access-Control-Allow-Origin", "http://localhost:5173")
        self.send_header("Access-Control-Allow-Methods", "POST, OPTIONS")
        self.send_header("Access-Control-Allow-Headers", "Content-Type")

    def log_message(self, format, *args):
        # Prefix logs so they're easy to spot
        print(f"  [bridge] {args[0]}")


def run_http(port: int = 3001):
    server = HTTPServer(("127.0.0.1", port), BridgeHandler)
    print(f"\n  Gateorix HTTP dev bridge")
    print(f"  → listening on http://localhost:{port}/invoke")
    print(f"  → CORS allowed: http://localhost:5173\n")
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print("\n  Bridge stopped.")
        server.server_close()


# --- Stdio sidecar mode (production) ---

def run_stdio():
    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue

        try:
            request = json.loads(line)
        except json.JSONDecodeError:
            response = {"id": "unknown", "ok": False, "payload": {"error": "invalid JSON"}}
            sys.stdout.write(json.dumps(response) + "\n")
            sys.stdout.flush()
            continue

        response = dispatch(request)
        sys.stdout.write(json.dumps(response) + "\n")
        sys.stdout.flush()


if __name__ == "__main__":
    if "--http" in sys.argv:
        run_http()
    else:
        run_stdio()
