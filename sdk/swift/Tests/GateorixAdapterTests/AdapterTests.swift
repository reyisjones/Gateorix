import XCTest
@testable import GateorixAdapter

final class GateorixAdapterTests: XCTestCase {
    func testSuccessResponse() {
        let response = IpcResponse.success("req-1", ["message": AnyCodable("hello")])
        XCTAssertEqual(response.id, "req-1")
        XCTAssertTrue(response.ok)
    }

    func testErrorResponse() {
        let response = IpcResponse.error("req-2", "something failed")
        XCTAssertEqual(response.id, "req-2")
        XCTAssertFalse(response.ok)
    }

    func testCommandRegistration() {
        let adapter = GateorixAdapter()
        adapter.command("greet") { payload in
            let name = payload["name"] as? String ?? "World"
            return ["message": AnyCodable("Hello, \(name)!")]
        }
        // Registration doesn't crash — basic smoke test
    }
}
