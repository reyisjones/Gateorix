// Package gateorix provides a runtime adapter SDK for building Gateorix
// desktop app backends in Go. It implements the adapter protocol over
// stdio (newline-delimited JSON).
//
// Example:
//
//	package main
//
//	import "github.com/gateorix/gateorix/sdk/go/gateorix"
//
//	func main() {
//	    adapter := gateorix.NewAdapter()
//	    adapter.Command("greet", func(payload map[string]any) (any, error) {
//	        name, _ := payload["name"].(string)
//	        if name == "" { name = "World" }
//	        return map[string]string{"message": "Hello, " + name + "!"}, nil
//	    })
//	    adapter.Run()
//	}
package gateorix

import (
	"bufio"
	"encoding/json"
	"fmt"
	"log"
	"net/http"
	"os"
)

// CommandHandler is a function that handles a Gateorix adapter command.
// It receives the payload and returns a result or an error.
type CommandHandler func(payload map[string]any) (any, error)

// Request represents a message from the host core to the sidecar.
type Request struct {
	ID      string         `json:"id"`
	Channel string         `json:"channel"`
	Payload map[string]any `json:"payload"`
}

// Response represents a message from the sidecar back to the host core.
type Response struct {
	ID      string `json:"id"`
	OK      bool   `json:"ok"`
	Payload any    `json:"payload"`
}

// Adapter is the main entry point for a Gateorix Go backend.
type Adapter struct {
	handlers map[string]CommandHandler
}

// NewAdapter creates a new adapter instance.
func NewAdapter() *Adapter {
	return &Adapter{
		handlers: make(map[string]CommandHandler),
	}
}

// Command registers a handler for the given channel name.
// The channel is matched against the "channel" field in incoming requests
// (e.g. "runtime.greet" matches if you register "greet" — the "runtime."
// prefix is stripped automatically).
func (a *Adapter) Command(name string, handler CommandHandler) {
	a.handlers[name] = handler
}

// dispatch routes a request to the appropriate handler.
func (a *Adapter) dispatch(req Request) Response {
	// Strip "runtime." prefix if present
	channel := req.Channel
	if len(channel) > 8 && channel[:8] == "runtime." {
		channel = channel[8:]
	}

	handler, ok := a.handlers[channel]
	if !ok {
		return Response{
			ID: req.ID,
			OK: false,
			Payload: map[string]string{
				"error": fmt.Sprintf("unknown command: %s", req.Channel),
			},
		}
	}

	result, err := handler(req.Payload)
	if err != nil {
		return Response{
			ID: req.ID,
			OK: false,
			Payload: map[string]string{
				"error": err.Error(),
			},
		}
	}

	return Response{
		ID:      req.ID,
		OK:      true,
		Payload: result,
	}
}

// Run starts the stdio message loop, reading JSON requests from stdin
// and writing JSON responses to stdout. Blocks until stdin is closed.
func (a *Adapter) Run() {
	scanner := bufio.NewScanner(os.Stdin)
	// Allow large messages (up to 10 MB)
	scanner.Buffer(make([]byte, 0, 64*1024), 10*1024*1024)

	for scanner.Scan() {
		line := scanner.Text()
		if line == "" {
			continue
		}

		var req Request
		if err := json.Unmarshal([]byte(line), &req); err != nil {
			resp := Response{
				ID: "",
				OK: false,
				Payload: map[string]string{
					"error": fmt.Sprintf("invalid JSON: %v", err),
				},
			}
			out, _ := json.Marshal(resp)
			fmt.Fprintln(os.Stdout, string(out))
			continue
		}

		resp := a.dispatch(req)
		out, err := json.Marshal(resp)
		if err != nil {
			resp = Response{
				ID: req.ID,
				OK: false,
				Payload: map[string]string{
					"error": fmt.Sprintf("marshal error: %v", err),
				},
			}
			out, _ = json.Marshal(resp)
		}
		fmt.Fprintln(os.Stdout, string(out))
	}
}

// RunHTTP starts an HTTP server on the given address (e.g. ":3001").
// The host core sends POST requests with JSON bodies to any path.
func (a *Adapter) RunHTTP(addr string) error {
	mux := http.NewServeMux()

	mux.HandleFunc("/invoke", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
			return
		}

		var req Request
		if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
			http.Error(w, "invalid JSON", http.StatusBadRequest)
			return
		}

		resp := a.dispatch(req)
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(resp)
	})

	// Health check
	mux.HandleFunc("/health", func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		fmt.Fprintln(w, `{"status":"ok"}`)
	})

	log.Printf("Gateorix Go adapter listening on %s", addr)
	return http.ListenAndServe(addr, mux)
}
