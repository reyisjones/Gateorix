// Hello Gateorix — Go backend example.
//
// Run modes:
//
//	HTTP server (dev bridge):
//	  go run main.go --http
//	  → starts on http://localhost:3001
//
//	Stdio sidecar (production):
//	  echo '{"id":"1","channel":"runtime.greet","payload":{"name":"Alice"}}' | go run main.go
package main

import (
	"bufio"
	"encoding/json"
	"fmt"
	"log"
	"net/http"
	"os"
	"strings"
)

// --- Request / Response types ---

type Request struct {
	ID      string                 `json:"id"`
	Channel string                 `json:"channel"`
	Payload map[string]interface{} `json:"payload"`
}

type Response struct {
	ID      string      `json:"id"`
	OK      bool        `json:"ok"`
	Payload interface{} `json:"payload"`
}

// --- Command handlers ---

func handleGreet(payload map[string]interface{}) (interface{}, error) {
	name, _ := payload["name"].(string)
	if name == "" {
		name = "World"
	}
	return map[string]string{"message": fmt.Sprintf("Hello from Go, %s! Welcome to Gateorix.", name)}, nil
}

func handleEcho(payload map[string]interface{}) (interface{}, error) {
	return payload, nil
}

var handlers = map[string]func(map[string]interface{}) (interface{}, error){
	"greet": handleGreet,
	"echo":  handleEcho,
}

func dispatch(req Request) Response {
	channel := req.Channel
	if idx := strings.LastIndex(channel, "."); idx >= 0 {
		channel = channel[idx+1:]
	}

	handler, ok := handlers[channel]
	if !ok {
		return Response{
			ID: req.ID, OK: false,
			Payload: map[string]string{"error": fmt.Sprintf("unknown command: %s", req.Channel)},
		}
	}

	result, err := handler(req.Payload)
	if err != nil {
		return Response{ID: req.ID, OK: false, Payload: map[string]string{"error": err.Error()}}
	}
	return Response{ID: req.ID, OK: true, Payload: result}
}

// --- HTTP dev bridge ---

func runHTTP(port int) {
	mux := http.NewServeMux()

	mux.HandleFunc("/invoke", func(w http.ResponseWriter, r *http.Request) {
		// CORS headers
		w.Header().Set("Access-Control-Allow-Origin", "http://localhost:5173")
		w.Header().Set("Access-Control-Allow-Methods", "POST, OPTIONS")
		w.Header().Set("Access-Control-Allow-Headers", "Content-Type")

		if r.Method == http.MethodOptions {
			w.WriteHeader(http.StatusNoContent)
			return
		}

		if r.Method == http.MethodGet {
			w.Header().Set("Content-Type", "application/json")
			json.NewEncoder(w).Encode(map[string]interface{}{
				"status": "running",
				"usage":  "POST /invoke with JSON body: {id, channel, payload}",
			})
			return
		}

		if r.Method != http.MethodPost {
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
			return
		}

		var req Request
		if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
			w.Header().Set("Content-Type", "application/json")
			w.WriteHeader(http.StatusBadRequest)
			json.NewEncoder(w).Encode(Response{OK: false, Payload: map[string]string{"error": "invalid JSON"}})
			return
		}

		resp := dispatch(req)
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(resp)
	})

	addr := fmt.Sprintf(":%d", port)
	fmt.Printf("\n  Gateorix HTTP dev bridge (Go)\n")
	fmt.Printf("  → listening on http://localhost%s/invoke\n", addr)
	fmt.Printf("  → CORS allowed: http://localhost:5173\n\n")
	log.Fatal(http.ListenAndServe(addr, mux))
}

// --- Stdio sidecar mode ---

func runStdio() {
	scanner := bufio.NewScanner(os.Stdin)
	scanner.Buffer(make([]byte, 0, 64*1024), 10*1024*1024)

	for scanner.Scan() {
		line := scanner.Text()
		if line == "" {
			continue
		}

		var req Request
		if err := json.Unmarshal([]byte(line), &req); err != nil {
			resp := Response{ID: "unknown", OK: false, Payload: map[string]string{"error": "invalid JSON"}}
			out, _ := json.Marshal(resp)
			fmt.Fprintln(os.Stdout, string(out))
			continue
		}

		resp := dispatch(req)
		out, _ := json.Marshal(resp)
		fmt.Fprintln(os.Stdout, string(out))
	}
}

func main() {
	for _, arg := range os.Args[1:] {
		if arg == "--http" {
			runHTTP(3001)
			return
		}
	}
	runStdio()
}
