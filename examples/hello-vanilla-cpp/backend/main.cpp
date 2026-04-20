// Hello Gateorix — C++ backend example.
//
// Build:
//   mkdir build && cd build && cmake .. && cmake --build .
//
// Run modes:
//   HTTP server (dev bridge):
//     ./hello_gateorix_cpp --http
//     → starts on http://localhost:3001
//
//   Stdio sidecar (production):
//     echo '{"id":"1","channel":"runtime.greet","payload":{"name":"Alice"}}' | ./hello_gateorix_cpp

#include <iostream>
#include <sstream>
#include <string>
#include <map>
#include <functional>
#include <cstring>

// --- Minimal JSON helpers (no external deps) ---

static std::string json_escape(const std::string &s) {
    std::string out;
    for (char c : s) {
        switch (c) {
            case '"':  out += "\\\""; break;
            case '\\': out += "\\\\"; break;
            case '\n': out += "\\n"; break;
            case '\r': out += "\\r"; break;
            case '\t': out += "\\t"; break;
            default:   out += c;
        }
    }
    return out;
}

static std::string json_get_string(const std::string &json, const std::string &key) {
    std::string needle = "\"" + key + "\"";
    auto pos = json.find(needle);
    if (pos == std::string::npos) return "";
    pos = json.find(':', pos + needle.size());
    if (pos == std::string::npos) return "";
    pos = json.find('"', pos + 1);
    if (pos == std::string::npos) return "";
    auto end = json.find('"', pos + 1);
    while (end != std::string::npos && json[end - 1] == '\\') end = json.find('"', end + 1);
    if (end == std::string::npos) return "";
    return json.substr(pos + 1, end - pos - 1);
}

static std::string json_get_object(const std::string &json, const std::string &key) {
    std::string needle = "\"" + key + "\"";
    auto pos = json.find(needle);
    if (pos == std::string::npos) return "{}";
    pos = json.find('{', pos + needle.size());
    if (pos == std::string::npos) return "{}";
    int depth = 0;
    size_t start = pos;
    for (size_t i = pos; i < json.size(); ++i) {
        if (json[i] == '{') depth++;
        else if (json[i] == '}') { depth--; if (depth == 0) return json.substr(start, i - start + 1); }
    }
    return "{}";
}

// --- Handlers ---

static std::string handle_greet(const std::string &payload) {
    std::string name = json_get_string(payload, "name");
    if (name.empty()) name = "World";
    return "{\"message\":\"Hello from C++, " + json_escape(name) + "! Welcome to Gateorix.\"}";
}

static std::string handle_echo(const std::string &payload) {
    return payload;
}

static std::map<std::string, std::function<std::string(const std::string &)>> handlers = {
    {"greet", handle_greet},
    {"echo", handle_echo},
};

static std::string dispatch(const std::string &id, const std::string &channel, const std::string &payload) {
    std::string action = channel;
    auto dot = channel.rfind('.');
    if (dot != std::string::npos) action = channel.substr(dot + 1);

    auto it = handlers.find(action);
    if (it == handlers.end()) {
        return "{\"id\":\"" + json_escape(id) + "\",\"ok\":false,\"payload\":{\"error\":\"unknown command: " + json_escape(action) + "\"}}";
    }
    try {
        std::string result = it->second(payload);
        return "{\"id\":\"" + json_escape(id) + "\",\"ok\":true,\"payload\":" + result + "}";
    } catch (const std::exception &e) {
        return "{\"id\":\"" + json_escape(id) + "\",\"ok\":false,\"payload\":{\"error\":\"" + json_escape(e.what()) + "\"}}";
    }
}

// --- Stdio mode ---

static void run_stdio() {
    std::string line;
    while (std::getline(std::cin, line)) {
        if (line.empty()) continue;
        std::string id = json_get_string(line, "id");
        std::string channel = json_get_string(line, "channel");
        std::string payload = json_get_object(line, "payload");
        std::cout << dispatch(id, channel, payload) << "\n" << std::flush;
    }
}

// --- HTTP mode (minimal, single-threaded, no external deps) ---

#ifdef _WIN32
  #include <winsock2.h>
  #include <ws2tcpip.h>
  #pragma comment(lib, "ws2_32.lib")
  typedef SOCKET sock_t;
  #define CLOSESOCK closesocket
  static void init_sockets() { WSADATA d; WSAStartup(MAKEWORD(2,2), &d); }
#else
  #include <sys/socket.h>
  #include <netinet/in.h>
  #include <unistd.h>
  typedef int sock_t;
  #define CLOSESOCK close
  #define INVALID_SOCKET -1
  static void init_sockets() {}
#endif

static std::string read_http_request(sock_t client) {
    std::string buf;
    char tmp[4096];
    while (true) {
        int n;
#ifdef _WIN32
        n = recv(client, tmp, sizeof(tmp), 0);
#else
        n = read(client, tmp, sizeof(tmp));
#endif
        if (n <= 0) break;
        buf.append(tmp, n);
        // Check if we have complete headers + body
        auto hdr_end = buf.find("\r\n\r\n");
        if (hdr_end != std::string::npos) {
            auto cl_pos = buf.find("Content-Length:");
            if (cl_pos == std::string::npos) cl_pos = buf.find("content-length:");
            if (cl_pos != std::string::npos) {
                int cl = std::stoi(buf.substr(cl_pos + 15));
                if (buf.size() >= hdr_end + 4 + (size_t)cl) break;
            } else {
                break;
            }
        }
    }
    return buf;
}

static void send_response(sock_t client, int status, const std::string &status_text,
                           const std::string &body, const std::string &origin) {
    std::ostringstream resp;
    resp << "HTTP/1.1 " << status << " " << status_text << "\r\n"
         << "Content-Type: application/json\r\n"
         << "Content-Length: " << body.size() << "\r\n"
         << "Access-Control-Allow-Origin: " << origin << "\r\n"
         << "Access-Control-Allow-Methods: POST, GET, OPTIONS\r\n"
         << "Access-Control-Allow-Headers: Content-Type\r\n"
         << "Connection: close\r\n"
         << "\r\n"
         << body;
    std::string r = resp.str();
#ifdef _WIN32
    send(client, r.c_str(), (int)r.size(), 0);
#else
    write(client, r.c_str(), r.size());
#endif
}

static void run_http() {
    init_sockets();
    sock_t server = socket(AF_INET, SOCK_STREAM, 0);
    if (server == INVALID_SOCKET) { std::cerr << "socket() failed\n"; return; }

    int opt = 1;
    setsockopt(server, SOL_SOCKET, SO_REUSEADDR, (const char *)&opt, sizeof(opt));

    struct sockaddr_in addr{};
    addr.sin_family = AF_INET;
    addr.sin_addr.s_addr = INADDR_ANY;
    addr.sin_port = htons(3001);

    if (bind(server, (struct sockaddr *)&addr, sizeof(addr)) < 0) {
        std::cerr << "bind() failed — is port 3001 already in use?\n";
        CLOSESOCK(server);
        return;
    }
    listen(server, 8);

    std::cout << "\n  Gateorix HTTP dev bridge (C++)\n"
              << "  → listening on http://localhost:3001/invoke\n"
              << "  → CORS allowed: http://localhost:5173\n\n" << std::flush;

    const std::string cors_origin = "http://localhost:5173";

    while (true) {
        sock_t client = accept(server, nullptr, nullptr);
        if (client == INVALID_SOCKET) continue;

        std::string raw = read_http_request(client);
        // Parse first line
        auto first_line_end = raw.find("\r\n");
        std::string first_line = raw.substr(0, first_line_end);

        if (first_line.find("OPTIONS") == 0) {
            send_response(client, 204, "No Content", "", cors_origin);
        } else if (first_line.find("GET /health") != std::string::npos) {
            send_response(client, 200, "OK", "{\"status\":\"ok\"}", cors_origin);
        } else if (first_line.find("GET /invoke") != std::string::npos) {
            send_response(client, 200, "OK",
                "{\"status\":\"running\",\"usage\":\"POST /invoke with JSON body: {id, channel, payload}\"}",
                cors_origin);
        } else if (first_line.find("POST /invoke") != std::string::npos) {
            auto body_start = raw.find("\r\n\r\n");
            std::string body = (body_start != std::string::npos) ? raw.substr(body_start + 4) : "";
            std::string id = json_get_string(body, "id");
            std::string channel = json_get_string(body, "channel");
            std::string payload = json_get_object(body, "payload");
            send_response(client, 200, "OK", dispatch(id, channel, payload), cors_origin);
        } else {
            send_response(client, 404, "Not Found", "{\"error\":\"not found\"}", cors_origin);
        }
        CLOSESOCK(client);
    }
}

int main(int argc, char *argv[]) {
    for (int i = 1; i < argc; ++i) {
        if (std::strcmp(argv[i], "--http") == 0) {
            run_http();
            return 0;
        }
    }
    run_stdio();
    return 0;
}
