# Hello React + Python

A minimal Gateorix example application with a React frontend and Python backend.

## Structure

```
hello-react-python/
├── frontend/          # React + Vite frontend
│   ├── src/
│   │   └── main.tsx
│   ├── index.html
│   └── package.json
├── backend/
│   └── main.py        # Python adapter backend
└── gateorix.config.json
```

## Running

```bash
# Start the frontend dev server
cd frontend && npm install && npm run dev

# In another terminal, test the Python backend
echo '{"id":"1","channel":"runtime.greet","payload":{"name":"Alice"}}' | python3 backend/main.py
```
