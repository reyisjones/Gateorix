# Gateorix Go Adapter SDK

Go runtime adapter for building Gateorix desktop app backends in Go.

## Installation

```bash
go get github.com/gateorix/gateorix/sdk/go/gateorix
```

## Quick Start

```go
package main

import "github.com/gateorix/gateorix/sdk/go/gateorix"

func main() {
    adapter := gateorix.NewAdapter()

    adapter.Command("greet", func(payload map[string]any) (any, error) {
        name, _ := payload["name"].(string)
        if name == "" {
            name = "World"
        }
        return map[string]string{"message": "Hello, " + name + "!"}, nil
    })

    // Stdio mode (default — used by Tauri sidecar)
    adapter.Run()

    // Or HTTP mode (for browser-based development)
    // adapter.RunHTTP(":3001")
}
```

## Features

- Stdio transport (newline-delimited JSON)
- HTTP transport for development
- Zero external dependencies
- Single static binary output
- Automatic `runtime.` prefix stripping
