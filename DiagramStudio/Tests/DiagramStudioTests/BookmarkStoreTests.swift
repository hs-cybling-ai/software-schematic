import Foundation
import XCTest
@testable import DiagramStudio

@MainActor
final class BookmarkStoreTests: XCTestCase {
    func testNoBookmarkRestoresAsNil() throws {
        let suite = "DiagramStudioTests-\(UUID().uuidString)"
        let defaults = try XCTUnwrap(UserDefaults(suiteName: suite))
        defer { defaults.removePersistentDomain(forName: suite) }
        let store = SecurityScopedBookmarkStore(defaults: defaults)
        XCTAssertNil(try store.restore())
    }

    func testBookmarkRoundTripRestoresSelectedFolder() throws {
        let suite = "DiagramStudioTests-\(UUID().uuidString)"
        let defaults = try XCTUnwrap(UserDefaults(suiteName: suite))
        defer { defaults.removePersistentDomain(forName: suite) }
        let folder = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString, isDirectory: true)
        try FileManager.default.createDirectory(at: folder, withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: folder) }
        let store = SecurityScopedBookmarkStore(defaults: defaults)
        try store.persist(folder)
        let restored = try XCTUnwrap(store.restore())
        XCTAssertEqual(restored.url.standardizedFileURL, folder.standardizedFileURL)
        XCTAssertFalse(restored.stale)
    }
}
