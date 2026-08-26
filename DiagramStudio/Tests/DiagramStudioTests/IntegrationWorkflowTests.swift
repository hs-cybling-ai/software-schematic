import Foundation
import XCTest
@testable import DiagramStudio

final class IntegrationWorkflowTests: XCTestCase {
    func testDiscoverEditSaveAndReopenAllFormats() throws {
        let root = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString, isDirectory: true)
        try FileManager.default.createDirectory(at: root, withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: root) }
        let repository = URL(fileURLWithPath: #filePath).deletingLastPathComponent().deletingLastPathComponent().deletingLastPathComponent().deletingLastPathComponent()
        let fixtures = [("fixtures/bpmn/valid.bpmn", "flow.bpmn"), ("fixtures/data-graph/valid.dgraph", "schema.dgraph")]
        for (source, destination) in fixtures {
            try FileManager.default.copyItem(at: repository.appendingPathComponent(source), to: root.appendingPathComponent(destination))
        }

        let service = WorkspaceFileService()
        let nodes = try service.discover(root: root)
        XCTAssertEqual(Set(nodes.map(\.format)), Set([.bpmn, .dataGraph]))
        for node in nodes {
            let (xml, metadata) = try service.read(node.url, root: root)
            let edited = xml.replacingOccurrences(of: "Example", with: "Persisted Example") + "\n"
            _ = try service.atomicWrite(edited, to: node.url, root: root, expected: metadata)
            XCTAssertEqual(try service.read(node.url, root: root).0, edited)
        }
    }

    func testMissingAndMalformedFilesDoNotOverwriteSource() throws {
        let root = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString, isDirectory: true)
        try FileManager.default.createDirectory(at: root, withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: root) }
        let malformed = root.appendingPathComponent("broken.bpmn")
        let bytes = Data("<broken>".utf8)
        try bytes.write(to: malformed)
        let service = WorkspaceFileService()
        XCTAssertEqual(try service.read(malformed, root: root).0, "<broken>")
        XCTAssertEqual(try Data(contentsOf: malformed), bytes)
        XCTAssertThrowsError(try service.read(root.appendingPathComponent("gone.bpmn"), root: root))
    }

    func testCreateDiscoverAndReadStarterDocumentsInRootAndNestedFolder() throws {
        let root = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString, isDirectory: true)
        let nested = root.appendingPathComponent("Architecture", isDirectory: true)
        try FileManager.default.createDirectory(at: nested, withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: root) }
        let service = WorkspaceFileService()

        let flow = try service.createDiagram(name: "Flow", format: .bpmn, destination: root, root: root)
        let schema = try service.createDiagram(name: "Schema", format: .dataGraph, destination: nested, root: root)
        let nodes = try service.discover(root: root)
        XCTAssertEqual(nodes.map(\.name), ["Architecture", "Flow.bpmn"])
        XCTAssertEqual(nodes.first?.children?.map(\.name), ["Schema.dgraph"])
        XCTAssertEqual(try service.read(flow, root: root).0, DiagramFormat.bpmn.starterXML)
        let source = try service.read(schema, root: root).0
        let value = try JSONSerialization.jsonObject(with: Data(source.utf8)) as? [String: Any]
        XCTAssertEqual(value?["version"] as? Int, 2)
        XCTAssertEqual((value?["contextStore"] as? [String: Any])?["uri"] as? String, "Schema.context.sqlite")
    }
}
