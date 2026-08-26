import Foundation
import XCTest
@testable import DiagramStudio

#if !SWIFT_PACKAGE
@MainActor
final class WorkspaceStoreTests: XCTestCase {
    private func makeStore() throws -> (WorkspaceStore, URL) {
        let root = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString, isDirectory: true)
        try FileManager.default.createDirectory(at: root, withIntermediateDirectories: true)
        try "<bpmn/>".write(to: root.appendingPathComponent("a.bpmn"), atomically: true, encoding: .utf8)
        try DiagramFormat.dataGraph.starterXML.write(to: root.appendingPathComponent("b.dgraph"), atomically: true, encoding: .utf8)
        let defaults = try XCTUnwrap(UserDefaults(suiteName: "WorkspaceStoreTests-\(UUID().uuidString)"))
        let store = WorkspaceStore(bookmarks: SecurityScopedBookmarkStore(defaults: defaults))
        try store.activate(root)
        return (store, root)
    }

    func testCanonicalOpenIdentityAndRefreshPreserveSelectedTab() throws {
        let (store, root) = try makeStore()
        defer { try? FileManager.default.removeItem(at: root) }
        let nodes = store.fileTree.filter { !$0.isDirectory }
        store.open(try XCTUnwrap(nodes.first))
        let selected = store.selectedTabID
        store.open(try XCTUnwrap(nodes.first))
        XCTAssertEqual(store.tabs.count, 1)
        store.refresh()
        XCTAssertEqual(store.selectedTabID, selected)
    }

    func testSaveAllKeepsOnlyFailedTabDirty() async throws {
        let (store, root) = try makeStore()
        defer { try? FileManager.default.removeItem(at: root) }
        for node in store.fileTree where !node.isDirectory { store.open(node) }
        XCTAssertEqual(store.tabs.count, 2)
        for tab in store.tabs { tab.isDirty = true; tab.isReady = true }
        store.tabs[0].exportXMLForSave = { "<saved/>" }
        store.tabs[1].exportXMLForSave = { throw BridgeError.exportFailed("fixture failure") }

        let failures = await store.saveAll()
        XCTAssertEqual(failures.count, 1)
        XCTAssertFalse(store.tabs[0].isDirty)
        XCTAssertTrue(store.tabs[1].isDirty)
    }

    func testConflictOverwriteRequiresExplicitDecision() async throws {
        let (store, root) = try makeStore()
        defer { try? FileManager.default.removeItem(at: root) }
        store.open(try XCTUnwrap(store.fileTree.first))
        let tab = try XCTUnwrap(store.selectedTab)
        tab.isDirty = true
        tab.isReady = true
        tab.exportXMLForSave = { "<editor/>" }
        try "<external change/>".write(to: tab.url, atomically: true, encoding: .utf8)
        store.conflictDecisionProvider = { _ in .overwrite }

        let succeeded = await store.saveSelected()
        XCTAssertTrue(succeeded)
        XCTAssertEqual(try String(contentsOf: tab.url, encoding: .utf8), "<editor/>")
        XCTAssertFalse(tab.isDirty)
    }

    func testUnsavedCancelAndDiscardDecisions() throws {
        let (store, root) = try makeStore()
        defer { try? FileManager.default.removeItem(at: root) }
        store.open(try XCTUnwrap(store.fileTree.first))
        let tab = try XCTUnwrap(store.selectedTab)
        tab.isDirty = true
        store.unsavedDecisionProvider = { _, _ in .cancel }
        store.close(tab)
        XCTAssertEqual(store.tabs.count, 1)

        store.unsavedDecisionProvider = { _, _ in .discard }
        store.close(tab)
        XCTAssertTrue(store.tabs.isEmpty)
    }

    func testClosingTabReleasesControllerOwnership() throws {
        let (store, root) = try makeStore()
        defer { try? FileManager.default.removeItem(at: root) }
        store.open(try XCTUnwrap(store.fileTree.first))
        weak var controller = store.selectedTab?.controller
        store.close(try XCTUnwrap(store.selectedTab))
        XCTAssertNil(controller)
    }

    func testDisplayedDestinationsIncludeRootAndNestedFoldersInHierarchyOrder() throws {
        let (store, root) = try makeStore()
        defer { try? FileManager.default.removeItem(at: root) }
        let parent = root.appendingPathComponent("Parent", isDirectory: true)
        let child = parent.appendingPathComponent("Child", isDirectory: true)
        try FileManager.default.createDirectory(at: child, withIntermediateDirectories: true)
        try "<bpmn/>".write(to: child.appendingPathComponent("nested.bpmn"), atomically: true, encoding: .utf8)
        store.refresh()

        XCTAssertEqual(store.diagramDestinations.map(\.label), ["Workspace Root", "Parent", "Child"])
        XCTAssertEqual(store.diagramDestinations.map(\.depth), [0, 1, 2])
    }

    func testNewDiagramPresentationCancelAndValidation() throws {
        let (store, root) = try makeStore()
        defer { try? FileManager.default.removeItem(at: root) }
        store.beginNewDiagram()
        XCTAssertTrue(store.isPresentingNewDiagram)
        XCTAssertNotNil(store.creationValidationMessage(name: "", format: .bpmn, destination: root))
        XCTAssertNil(store.creationValidationMessage(name: "Flow", format: .bpmn, destination: root))
        store.cancelNewDiagram()
        XCTAssertFalse(store.isPresentingNewDiagram)
    }

    func testCreateRefreshesAndOpensCleanCanonicalTab() throws {
        let (store, root) = try makeStore()
        defer { try? FileManager.default.removeItem(at: root) }
        store.beginNewDiagram()
        XCTAssertTrue(store.createDiagram(name: "New Flow", format: .bpmn, destination: root))
        let tab = try XCTUnwrap(store.selectedTab)
        XCTAssertEqual(tab.url.lastPathComponent, "New Flow.bpmn")
        XCTAssertFalse(tab.isDirty)
        XCTAssertTrue(store.fileTree.contains { $0.url.lastPathComponent == "New Flow.bpmn" })
        XCTAssertFalse(store.isPresentingNewDiagram)
    }

    func testCreationKeepsSheetOpenForCollisionAndStaleDestination() throws {
        let (store, root) = try makeStore()
        defer { try? FileManager.default.removeItem(at: root) }
        store.beginNewDiagram()
        XCTAssertFalse(store.createDiagram(name: "a.bpmn", format: .bpmn, destination: root))
        XCTAssertTrue(store.isPresentingNewDiagram)
        XCTAssertEqual(store.newDiagramError, WorkspaceError.fileAlreadyExists.localizedDescription)

        let stale = root.appendingPathComponent("Stale", isDirectory: true)
        try FileManager.default.createDirectory(at: stale, withIntermediateDirectories: true)
        try FileManager.default.removeItem(at: stale)
        store.clearNewDiagramError()
        XCTAssertFalse(store.createDiagram(name: "new", format: .bpmn, destination: stale))
        XCTAssertEqual(store.newDiagramError, WorkspaceError.invalidDestination.localizedDescription)
    }
}
#endif
