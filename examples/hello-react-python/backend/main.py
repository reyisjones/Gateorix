"""
Hello Gateorix — Python backend example.

Run standalone for testing:
  echo '{"id":"1","channel":"runtime.greet","payload":{"name":"Alice"}}' | python3 main.py
"""

import json
import sys


def handle_greet(payload: dict) -> dict:
    name = payload.get("name", "World")
    return {"message": f"Hello, {name}! Welcome to Gateorix."}


def handle_echo(payload: dict) -> dict:
    return payload


HANDLERS = {
    "greet": handle_greet,
    "echo": handle_echo,
}


def main() -> None:
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

        req_id = request.get("id", "unknown")
        channel = request.get("channel", "")
        payload = request.get("payload", {})

        action = channel.split(".")[-1] if "." in channel else channel
        handler = HANDLERS.get(action)

        if handler is None:
            response = {"id": req_id, "ok": False, "payload": {"error": f"unknown command: {action}"}}
        else:
            try:
                result = handler(payload)
                response = {"id": req_id, "ok": True, "payload": result}
            except Exception as exc:
                response = {"id": req_id, "ok": False, "payload": {"error": str(exc)}}

        sys.stdout.write(json.dumps(response) + "\n")
        sys.stdout.flush()


if __name__ == "__main__":
    main()
