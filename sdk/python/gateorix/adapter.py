"""
Gateorix runtime adapter for Python.

Implements the adapter protocol over stdio (newline-delimited JSON).
The host core spawns this process and communicates by writing JSON
messages to stdin and reading responses from stdout.
"""

import json
import sys
from typing import Any, Callable

CommandHandler = Callable[[dict[str, Any]], Any]


class GateorixAdapter:
    """
    Python runtime adapter for Gateorix.

    Register command handlers and call `run()` to start the stdio
    message loop.

    Example:
        adapter = GateorixAdapter()

        @adapter.command("greet")
        def greet(payload):
            return {"message": f"Hello, {payload.get('name', 'World')}!"}

        adapter.run()
    """

    def __init__(self) -> None:
        self._handlers: dict[str, CommandHandler] = {}

    def command(self, name: str) -> Callable[[CommandHandler], CommandHandler]:
        """Decorator to register a command handler."""
        def decorator(func: CommandHandler) -> CommandHandler:
            self._handlers[name] = func
            return func
        return decorator

    def run(self) -> None:
        """Start the stdio message loop. Blocks until stdin is closed."""
        for line in sys.stdin:
            line = line.strip()
            if not line:
                continue

            try:
                request = json.loads(line)
            except json.JSONDecodeError:
                self._send_error("unknown", "invalid JSON")
                continue

            req_id = request.get("id", "unknown")
            channel = request.get("channel", "")
            payload = request.get("payload", {})

            # Extract action from channel (e.g. "runtime.greet" → "greet")
            action = channel.split(".")[-1] if "." in channel else channel

            handler = self._handlers.get(action)
            if handler is None:
                self._send_error(req_id, f"unknown command: {action}")
                continue

            try:
                result = handler(payload)
                self._send_ok(req_id, result)
            except Exception as exc:
                self._send_error(req_id, str(exc))

    def _send_ok(self, req_id: str, payload: Any) -> None:
        response = {"id": req_id, "ok": True, "payload": payload}
        sys.stdout.write(json.dumps(response) + "\n")
        sys.stdout.flush()

    def _send_error(self, req_id: str, message: str) -> None:
        response = {"id": req_id, "ok": False, "payload": {"error": message}}
        sys.stdout.write(json.dumps(response) + "\n")
        sys.stdout.flush()
