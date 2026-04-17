"""
Hello Gateorix — Python backend.

This is the runtime adapter entry point. The Gateorix host core spawns
this process and communicates over stdin/stdout using newline-delimited JSON.
"""

from gateorix import GateorixAdapter

adapter = GateorixAdapter()


@adapter.command("greet")
def greet(payload: dict) -> dict:
    """Return a greeting message."""
    name = payload.get("name", "World")
    return {"message": f"Hello, {name}! Welcome to Gateorix."}


@adapter.command("echo")
def echo(payload: dict) -> dict:
    """Echo the payload back."""
    return payload


if __name__ == "__main__":
    adapter.run()
