import Foundation
import XCTest
@testable import DiagramStudio

final class SQLiteContextStoreTests: XCTestCase {
    private func section(_ id: String, node: String, revision: Int, vector: [Float], tombstone: Bool = false) -> ContextSection {
        ContextSection(diagramID:"diagram-1",contextID:"context-\(node)",nodeID:node,nodeType:"objectNode",nodeLabel:node.capitalized,sectionID:id,headingPath:["Overview"],ordinal:0,markdown:tombstone ? "" : "# Overview\n\(node)",contentHash:String(repeating:tombstone ? "b":"a",count:64),embedding:vector,embeddingProvider:tombstone ? "":"test",embeddingModel:tombstone ? "":"fixture-3",capturedAt:Date(timeIntervalSince1970:1_700_000_000),captureRevision:revision,isTombstone:tombstone)
    }

    func testCreatesReopensQueriesAndCompactsDatabase() throws {
        let root=FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString,isDirectory:true)
        try FileManager.default.createDirectory(at:root,withIntermediateDirectories:true); defer{try? FileManager.default.removeItem(at:root)}
        let url=root.appendingPathComponent("model.context.sqlite")
        do {
            let store=try SQLiteContextStore(url:url,diagramID:"diagram-1")
            try store.commit(revision:1,diagramID:"diagram-1",sections:[section("section-a",node:"person",revision:1,vector:[0.1,0.2,0.3]),section("section-b",node:"company",revision:1,vector:[0.3,0.2,0.1]),section("section-c",node:"deleted",revision:1,vector:[],tombstone:true)])
            XCTAssertEqual(try store.sections(nodeID:"person",revision:1).first?.embedding,[0.1,0.2,0.3])
            try store.commit(revision:2,diagramID:"diagram-1",sections:[section("section-a",node:"person",revision:2,vector:[0.2,0.2,0.2])])
            XCTAssertEqual(try store.unreferencedRevisions(authoritative:1),[2])
        }
        let reopened=try SQLiteContextStore(url:url,diagramID:"diagram-1")
        XCTAssertEqual(try reopened.sections(revision:1).count,3)
        try reopened.compact(keeping:[2]); XCTAssertThrowsError(try reopened.sections(revision:1))
    }

    func testRejectsInvalidRowsAndMismatchedDiagram() throws {
        XCTAssertThrowsError(try section("bad",node:"person",revision:1,vector:[.nan]).validated())
        let url=FileManager.default.temporaryDirectory.appendingPathComponent("\(UUID().uuidString).context.sqlite"); defer{try? FileManager.default.removeItem(at:url)}
        _=try SQLiteContextStore(url:url,diagramID:"diagram-1")
        XCTAssertThrowsError(try SQLiteContextStore(url:url,diagramID:"diagram-2")){XCTAssertEqual($0 as? ContextStoreError,.diagramMismatch)}
    }
}
