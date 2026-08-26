import Foundation
import XCTest
@testable import DiagramStudio

final class WorkspaceFileServiceTests: XCTestCase {
    private var root: URL!
    private let service = WorkspaceFileService()

    override func setUpWithError() throws {
        root = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString, isDirectory: true)
        try FileManager.default.createDirectory(at: root, withIntermediateDirectories: true)
    }

    override func tearDownWithError() throws { try? FileManager.default.removeItem(at: root) }

    func testDiscoveryFiltersAndSortsFoldersBeforeFiles() throws {
        let nested = root.appendingPathComponent("Zoo", isDirectory: true)
        try FileManager.default.createDirectory(at: nested, withIntermediateDirectories: true)
        try "<bpmn/>".write(to: root.appendingPathComponent("alpha.bpmn"), atomically: true, encoding: .utf8)
        try DiagramFormat.dataGraph.starterXML.write(to: root.appendingPathComponent("schema.dgraph"), atomically: true, encoding: .utf8)
        try "ignore".write(to: root.appendingPathComponent("notes.txt"), atomically: true, encoding: .utf8)
        let nodes = try service.discover(root: root)
        XCTAssertEqual(nodes.map(\.name), ["alpha.bpmn", "schema.dgraph"])
    }

    func testCanonicalBoundaryRejectsSiblingAndEscapingSymlink() throws {
        let sibling = root.deletingLastPathComponent().appendingPathComponent(root.lastPathComponent + "-sibling")
        try FileManager.default.createDirectory(at: sibling, withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: sibling) }
        let outside = sibling.appendingPathComponent("outside.bpmn")
        try "outside".write(to: outside, atomically: true, encoding: .utf8)
        let link = root.appendingPathComponent("linked.bpmn")
        try FileManager.default.createSymbolicLink(at: link, withDestinationURL: outside)
        XCTAssertFalse(service.contains(sibling, root: root))
        XCTAssertFalse(try service.discover(root: root).contains { $0.name == "linked.bpmn" })
        XCTAssertThrowsError(try service.read(link, root: root)) { XCTAssertEqual($0 as? WorkspaceError, .outsideWorkspace) }
    }

    func testAtomicWriteDetectsConflictAndPreservesExternalBytes() throws {
        let file = root.appendingPathComponent("diagram.bpmn")
        try "original".write(to: file, atomically: true, encoding: .utf8)
        let (_, metadata) = try service.read(file, root: root)
        try "external change with different size".write(to: file, atomically: true, encoding: .utf8)
        XCTAssertThrowsError(try service.atomicWrite("editor", to: file, root: root, expected: metadata)) { XCTAssertEqual($0 as? WorkspaceError, .sourceChanged) }
        XCTAssertEqual(try String(contentsOf: file, encoding: .utf8), "external change with different size")
    }

    func testOversizedFileIsRejectedWithoutModification() throws {
        let file = root.appendingPathComponent("large.bpmn")
        try Data(repeating: 0x20, count: DiagramFormat.maximumFileSize + 1).write(to: file)
        XCTAssertThrowsError(try service.read(file, root: root)) { XCTAssertEqual($0 as? WorkspaceError, .fileTooLarge) }
    }

    func testCreatesAllFormatsInRootAndNestedFolder() throws {
        let nested = root.appendingPathComponent("Displayed", isDirectory: true)
        try FileManager.default.createDirectory(at: nested, withIntermediateDirectories: true)
        let bpmn = try service.createDiagram(name: "Flow", format: .bpmn, destination: root, root: root)
        let dataGraph = try service.createDiagram(name: "Schema", format: .dataGraph, destination: nested, root: root)
        XCTAssertEqual(bpmn.lastPathComponent, "Flow.bpmn")
        XCTAssertEqual(try String(contentsOf: bpmn, encoding: .utf8), DiagramFormat.bpmn.starterXML)
        XCTAssertEqual(dataGraph.lastPathComponent, "Schema.dgraph")
        let created = try JSONSerialization.jsonObject(with: Data(contentsOf: dataGraph)) as? [String: Any]
        XCTAssertEqual(created?["version"] as? Int, 2)
        XCTAssertEqual((created?["contextStore"] as? [String: Any])?["uri"] as? String, "Schema.context.sqlite")
    }

    func testNameValidationAndCompatibleExtensions() throws {
        XCTAssertEqual(try service.resolvedFilename(" flow ", format: .bpmn), "flow.bpmn")
        XCTAssertEqual(try service.resolvedFilename("flow.bpmn20.xml", format: .bpmn), "flow.bpmn20.xml")
        for invalid in ["", " ", ".", "..", "folder/name", "folder\\name"] {
            XCTAssertThrowsError(try service.resolvedFilename(invalid, format: .bpmn)) {
                XCTAssertEqual($0 as? WorkspaceError, .invalidDiagramName)
            }
        }
    }

    func testCreationRejectsOutsideMissingAndFileDestinations() throws {
        let outside = root.deletingLastPathComponent().appendingPathComponent(UUID().uuidString, isDirectory: true)
        try FileManager.default.createDirectory(at: outside, withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: outside) }
        let fileDestination = root.appendingPathComponent("not-a-folder")
        try "x".write(to: fileDestination, atomically: true, encoding: .utf8)
        for destination in [outside, root.appendingPathComponent("missing"), fileDestination] {
            XCTAssertThrowsError(try service.createDiagram(name: "Flow", format: .bpmn, destination: destination, root: root)) {
                XCTAssertEqual($0 as? WorkspaceError, .invalidDestination)
            }
        }
    }

    func testCreationNeverOverwritesExistingTarget() throws {
        let target = root.appendingPathComponent("Flow.bpmn")
        try "existing".write(to: target, atomically: true, encoding: .utf8)
        XCTAssertThrowsError(try service.createDiagram(name: "Flow", format: .bpmn, destination: root, root: root)) {
            XCTAssertEqual($0 as? WorkspaceError, .fileAlreadyExists)
        }
        XCTAssertEqual(try String(contentsOf: target, encoding: .utf8), "existing")
    }

    func testMigratesLegacyDataGraphExtensionAndVersion() throws {
        let legacy = root.appendingPathComponent("Orders.dgraph.json")
        let versionOne = """
        {"format":"diagram-studio/data-graph","version":1,"nodes":[],"edges":[],"properties":[]}
        """
        try versionOne.write(to: legacy, atomically: true, encoding: .utf8)
        let migrated = try service.migrateLegacyDataGraph(legacy, root: root)
        XCTAssertEqual(migrated.lastPathComponent, "Orders.dgraph")
        let value = try JSONSerialization.jsonObject(with: Data(contentsOf: migrated)) as? [String: Any]
        XCTAssertEqual(value?["version"] as? Int, 2)
        XCTAssertEqual((value?["contextStore"] as? [String: Any])?["uri"] as? String, "Orders.context.sqlite")
        XCTAssertThrowsError(try service.migrateLegacyDataGraph(legacy, root: root)) { XCTAssertEqual($0 as? WorkspaceError, .fileAlreadyExists) }
    }
}
