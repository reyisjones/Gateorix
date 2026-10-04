# JavaScript Bridge SDK

The local `@gateorix/bridge` package exposes `GateorixBridge` and TypeScript types
for requests, responses, events, and invocation options. Package ownership and
registry publication are separate maintainer decisions; these instructions use
the source checkout and do not assume a published package is available.

## Module Contract

This package is ESM-only. Use its public root export:

```typescript
import { GateorixBridge, type InvokeOptions } from "@gateorix/bridge";
```

The runtime entry is `dist/index.js`; declarations are `dist/index.d.ts`.
CommonJS `require()` and deep imports are not supported public entry points.
The package test verifies ESM loading and declaration resolution in both
TypeScript NodeNext and Bundler modes.

## Build and Test

From the repository root:

```bash
npm --prefix sdk/js ci
npm --prefix sdk/js test
```

The test packs the SDK, installs the tarball in a temporary consumer with
installation scripts disabled, checks its file inventory, imports it, and
type-checks consumer code. It removes the temporary consumer afterward.

`npm --prefix sdk/js run build` clears this package's generated `dist` directory,
compiles the SDK, and copies the repository license to `dist/LICENSE`. Packing
from the source checkout runs that build automatically. The tarball contains
JavaScript, declarations, the license, README and package metadata, not source,
tests, build scripts or source maps. Consumers do not need TypeScript to import
the package. Rebuilding or repacking from an installed tarball is not supported.

## Transport Status

Packaging checks do not certify host communication. The existing bridge expects
a host-provided `window.__GATEORIX_IPC__.postMessage` hook and installs the global
`window.__GATEORIX_RECEIVE__` callback. It is not automatically integrated with
the examples' Tauri bridges. Missing-transport errors, default deadlines,
independent instance routing and disposal remain follow-up work. Do not use
successful package import as evidence of production-ready IPC.

See the [adapter protocol](https://github.com/reyisjones/Gateorix/blob/main/docs/adapter-protocol.md)
and [security model](https://github.com/reyisjones/Gateorix/blob/main/docs/security.md).