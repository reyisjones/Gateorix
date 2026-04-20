# Gateorix .NET Adapter SDK

Runtime adapter for building Gateorix desktop app backends in C# or F#.

## Installation

```bash
dotnet add package Gateorix.Adapter
```

## Quick Start (C#)

```csharp
using Gateorix;

var adapter = new GateorixAdapter();

adapter.Command("greet", payload =>
{
    var name = payload.TryGetProperty("name", out var n) ? n.GetString() : "World";
    return new { message = $"Hello, {name}!" };
});

// Stdio mode (default — used by Tauri sidecar)
adapter.Run();

// Or HTTP mode (for browser-based development)
// adapter.RunHttp("http://localhost:3001");
```

## Features

- Stdio transport (newline-delimited JSON)
- HTTP transport via ASP.NET Minimal API
- System.Text.Json serialization
- AOT compilation support (`dotnet publish -r win-x64 --self-contained`)
- Works with C# and F#
