import Foundation

/// JSON-based IPC request from the Gateorix host core.
public struct IpcRequest: Codable {
    public let id: String
    public let channel: String
    public let payload: [String: AnyCodable]
}

/// JSON-based IPC response sent back to the host core.
public struct IpcResponse: Codable {
    public let id: String
    public let ok: Bool
    public let payload: [String: AnyCodable]

    public static func success(_ id: String, _ data: [String: AnyCodable]) -> IpcResponse {
        IpcResponse(id: id, ok: true, payload: data)
    }

    public static func error(_ id: String, _ message: String) -> IpcResponse {
        IpcResponse(id: id, ok: false, payload: ["error": AnyCodable(message)])
    }
}

/// Type-erased Codable wrapper for JSON values.
public struct AnyCodable: Codable {
    public let value: Any

    public init(_ value: Any) {
        self.value = value
    }

    public init(from decoder: Decoder) throws {
        let container = try decoder.singleValueContainer()
        if let str = try? container.decode(String.self) {
            value = str
        } else if let int = try? container.decode(Int.self) {
            value = int
        } else if let double = try? container.decode(Double.self) {
            value = double
        } else if let bool = try? container.decode(Bool.self) {
            value = bool
        } else if let dict = try? container.decode([String: AnyCodable].self) {
            value = dict.mapValues { $0.value }
        } else if let arr = try? container.decode([AnyCodable].self) {
            value = arr.map { $0.value }
        } else if container.decodeNil() {
            value = NSNull()
        } else {
            throw DecodingError.typeMismatch(Any.self, .init(codingPath: decoder.codingPath, debugDescription: "Unsupported type"))
        }
    }

    public func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer()
        switch value {
        case let str as String: try container.encode(str)
        case let int as Int: try container.encode(int)
        case let double as Double: try container.encode(double)
        case let bool as Bool: try container.encode(bool)
        case is NSNull: try container.encodeNil()
        default: try container.encodeNil()
        }
    }
}
