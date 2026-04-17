# Gateorix Python Adapter SDK

Build desktop application backends in Python using the Gateorix runtime adapter protocol.

## Installation

```bash
pip install gateorix
```

## Quick Start

```python
from gateorix import GateorixAdapter

adapter = GateorixAdapter()

@adapter.command("greet")
def greet(payload):
    name = payload.get("name", "World")
    return {"message": f"Hello, {name}!"}

adapter.run()
```
