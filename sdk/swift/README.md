# Gateorix Swift Adapter SDK

> **Status:** Phase 3 — Initial Implementation

Swift adapter SDK for writing Gateorix desktop app backends in Swift.
Communicates with the host core over the adapter protocol (stdio JSON).

## Installation

Add to your `Package.swift`:

```swift
.package(path: "../sdk/swift")
```

## Usage

```swift
import GateorixAdapter

let adapter = GateorixAdapter()

adapter.command("greet") { payload in
    let name = payload["name"] as? String ?? "World"
    return ["message": AnyCodable("Hello from Swift, \(name)!")]
}

adapter.run()
```

## Features

- `GateorixAdapter` class with command registration
- Stdio transport (newline-delimited JSON)
- `IpcRequest` / `IpcResponse` protocol types
- `AnyCodable` wrapper for type-erased JSON values

## Requirements

- Swift 5.9+
- macOS 13+
