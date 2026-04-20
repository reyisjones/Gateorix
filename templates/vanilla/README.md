# Vanilla Template

Minimal Gateorix starter — plain HTML/CSS/JS, no framework.

## Quick Start

```bash
gateorix init my-app     # select "vanilla" as UI framework
cd my-app
gateorix dev             # opens in browser at http://localhost:5173
```

## Structure

```
frontend/
├── index.html           ← entry point
└── package.json         ← Vite dev server
gateorix.config.json     ← project manifest
```

## Adding a Backend

```bash
gateorix add runtime python   # or go, dotnet, cpp
gateorix dev
```

## Building for Desktop

```bash
cd frontend
npx tauri init            # adds src-tauri/
gateorix build --release
```
