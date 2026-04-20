import Foundation

/// Command handler function type.
public typealias CommandHandler = ([String: Any]) -> [String: AnyCodable]

/// Gateorix runtime adapter for Swift backends.
///
/// Register command handlers and call `run()` to start the stdio message loop.
///
/// ```swift
/// let adapter = GateorixAdapter()
///
/// adapter.command("greet") { payload in
///     let name = payload["name"] as? String ?? "World"
///     return ["message": AnyCodable("Hello from Swift, \(name)!")]
/// }
///
/// adapter.run()
/// ```
public class GateorixAdapter {
    private var handlers: [String: CommandHandler] = [:]
    private let decoder = JSONDecoder()
    private let encoder = JSONEncoder()

    public init() {
        encoder.outputFormatting = []
    }

    /// Register a command handler for the given action name.
    public func command(_ action: String, handler: @escaping CommandHandler) {
        handlers[action] = handler
    }

    /// Start the stdio message loop — reads JSON lines from stdin,
    /// dispatches to registered handlers, writes JSON responses to stdout.
    public func run() {
        while let line = readLine() {
            guard let data = line.data(using: .utf8) else { continue }
            guard let request = try? decoder.decode(IpcRequest.self, from: data) else {
                continue
            }

            let action = request.channel
                .split(separator: ".")
                .last
                .map(String.init) ?? request.channel

            let response: IpcResponse
            if let handler = handlers[action] {
                let payloadDict = request.payload.mapValues { $0.value }
                let result = handler(payloadDict)
                response = .success(request.id, result)
            } else {
                response = .error(request.id, "unknown command: \(request.channel)")
            }

            if let responseData = try? encoder.encode(response),
               let responseStr = String(data: responseData, encoding: .utf8) {
                print(responseStr)
                fflush(stdout)
            }
        }
    }
}
