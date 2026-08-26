import Foundation
import XCTest
@testable import DiagramStudio

final class ContextCaptureServiceTests: XCTestCase {
    final class CountingProvider: EmbeddingProvider, @unchecked Sendable { let provider="counting",model="m"; var calls=0; func embed(_ text:String)throws->[Float]{calls += 1; return [1,0]} }
    struct FailingProvider: EmbeddingProvider { let provider="failing",model="m"; func embed(_ text:String)throws->[Float]{throw ContextStoreError.invalidEmbedding} }
    func testCaptureWritesMarkdownAndEmbeddingThenAdvancesManifest() throws {
        let root=FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString,isDirectory:true)
        try FileManager.default.createDirectory(at:root,withIntermediateDirectories:true); defer{try? FileManager.default.removeItem(at:root)}
        let diagram=root.appendingPathComponent("model.dgraph")
        let source="""
        {"format":"diagram-studio/data-graph","version":2,"nodes":[{"id":"person","type":"objectNode","label":"Person","x":0,"y":0}],"edges":[],"properties":[],"contextStore":{"kind":"sqlite","uri":"model.context.sqlite","schemaVersion":1,"diagramId":"diagram-1","revision":0}}
        """
        try source.write(to:diagram,atomically:true,encoding:.utf8)
        let metadata=try WorkspaceFileService().metadata(for:diagram)
        let markdown="# Overview\nPerson context\n"
        let section=ContextDraftSectionPayload(sectionId:"overview",markdown:markdown,contentHash:ContextCaptureService.sha256(markdown),headingPath:["Overview"],ordinal:0)
        let payload=ContextCapturePayload(diagram:source,drafts:[ContextDraftPayload(contextId:"context-person",nodeId:"person",nodeType:"objectNode",nodeLabel:"Person",markdown:markdown,sections:[section])],tombstones:[])
        let (result,_)=try ContextCaptureService(provider:DeterministicEmbeddingProvider()).capture(payload,diagramURL:diagram,root:root,expected:metadata)
        XCTAssertEqual(result.revision,1)
        let rows=try SQLiteContextStore(url:root.appendingPathComponent("model.context.sqlite"),diagramID:"diagram-1").sections(revision:1)
        XCTAssertEqual(rows.first?.markdown,markdown); XCTAssertEqual(rows.first?.embedding.count,8); XCTAssertTrue(rows.first?.embedding.allSatisfy(\.isFinite) == true)
        let saved=try JSONSerialization.jsonObject(with:Data(contentsOf:diagram)) as? [String:Any]
        XCTAssertEqual((saved?["contextStore"] as? [String:Any])?["revision"] as? Int,1)
    }

    func testReusesUnchangedEmbeddingsAndLeavesRevisionOnManifestConflict() throws {
        let root=FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString,isDirectory:true); try FileManager.default.createDirectory(at:root,withIntermediateDirectories:true); defer{try? FileManager.default.removeItem(at:root)}
        let diagram=root.appendingPathComponent("reuse.dgraph"); let source="""
        {"format":"diagram-studio/data-graph","version":2,"nodes":[{"id":"n","type":"objectNode","label":"N","x":0,"y":0}],"edges":[],"properties":[],"contextStore":{"kind":"sqlite","uri":"reuse.context.sqlite","schemaVersion":1,"diagramId":"d","revision":0}}
        """; try source.write(to:diagram,atomically:true,encoding:.utf8); let provider=CountingProvider(); let service=ContextCaptureService(provider:provider); let markdown="# A\nSame\n"; let section=ContextDraftSectionPayload(sectionId:"a",markdown:markdown,contentHash:ContextCaptureService.sha256(markdown),headingPath:["A"],ordinal:0); let draft=ContextDraftPayload(contextId:"c",nodeId:"n",nodeType:"objectNode",nodeLabel:"N",markdown:markdown,sections:[section])
        let first=try service.capture(ContextCapturePayload(diagram:source,drafts:[draft],tombstones:[]),diagramURL:diagram,root:root,expected:WorkspaceFileService().metadata(for:diagram)); XCTAssertEqual(provider.calls,1)
        _=try service.capture(ContextCapturePayload(diagram:first.0.diagram,drafts:[draft],tombstones:[]),diagramURL:diagram,root:root,expected:first.1); XCTAssertEqual(provider.calls,1)
        let current=try String(contentsOf:diagram,encoding:.utf8); let metadata=try WorkspaceFileService().metadata(for:diagram); try (current+" ").write(to:diagram,atomically:true,encoding:.utf8)
        XCTAssertThrowsError(try service.capture(ContextCapturePayload(diagram:current,drafts:[draft],tombstones:[]),diagramURL:diagram,root:root,expected:metadata))
        XCTAssertEqual(try SQLiteContextStore(url:root.appendingPathComponent("reuse.context.sqlite"),diagramID:"d").unreferencedRevisions(authoritative:2),[3])
    }

    func testProviderFailureDoesNotCommitCapture() throws {
        let root=FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString,isDirectory:true); try FileManager.default.createDirectory(at:root,withIntermediateDirectories:true); defer{try? FileManager.default.removeItem(at:root)}
        let diagram=root.appendingPathComponent("fail.dgraph"); let source="""
        {"format":"diagram-studio/data-graph","version":2,"nodes":[],"edges":[],"properties":[],"contextStore":{"kind":"sqlite","uri":"fail.context.sqlite","schemaVersion":1,"diagramId":"d","revision":0}}
        """; try source.write(to:diagram,atomically:true,encoding:.utf8); let markdown="# A\nB\n"; let section=ContextDraftSectionPayload(sectionId:"a",markdown:markdown,contentHash:ContextCaptureService.sha256(markdown),headingPath:["A"],ordinal:0); let draft=ContextDraftPayload(contextId:"c",nodeId:"n",nodeType:"objectNode",nodeLabel:"N",markdown:markdown,sections:[section])
        XCTAssertThrowsError(try ContextCaptureService(provider:FailingProvider()).capture(ContextCapturePayload(diagram:source,drafts:[draft],tombstones:[]),diagramURL:diagram,root:root,expected:WorkspaceFileService().metadata(for:diagram)))
        let store=try SQLiteContextStore(url:root.appendingPathComponent("fail.context.sqlite"),diagramID:"d"); XCTAssertThrowsError(try store.sections(revision:1))
    }
}
