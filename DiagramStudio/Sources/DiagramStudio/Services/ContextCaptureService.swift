import CryptoKit
import Foundation

struct ContextCaptureService {
    let provider: any EmbeddingProvider
    let fileService: WorkspaceFileService

    init(provider: any EmbeddingProvider = AppleSentenceEmbeddingProvider(), fileService: WorkspaceFileService = WorkspaceFileService()) { self.provider = provider; self.fileService = fileService }

    func capture(_ payload: ContextCapturePayload, diagramURL: URL, root: URL, expected: SourceMetadata) throws -> (ContextCaptureResult, SourceMetadata) {
        guard var diagram = try JSONSerialization.jsonObject(with: Data(payload.diagram.utf8)) as? [String: Any],
              diagram["format"] as? String == "diagram-studio/data-graph", diagram["version"] as? Int == 2,
              var manifest = diagram["contextStore"] as? [String: Any], let diagramID = manifest["diagramId"] as? String,
              let uri = manifest["uri"] as? String, let oldRevision = manifest["revision"] as? Int else { throw BridgeError.invalidMessage }
        let databaseURL = try fileService.contextDatabaseURL(for: diagramURL, manifestURI: uri, root: root)
        let store = try SQLiteContextStore(url: databaseURL, diagramID: diagramID)
        let revision = oldRevision + 1
        let previous = oldRevision > 0 ? try store.sections(revision: oldRevision) : []
        let reuse = Dictionary(uniqueKeysWithValues: previous.filter { !$0.isTombstone && $0.embeddingProvider == provider.provider && $0.embeddingModel == provider.model }.map { ("\($0.sectionID)\u{0}\($0.contentHash)", $0.embedding) })
        let capturedAt = Date()
        var rows: [ContextSection] = []
        for draft in payload.drafts {
            guard !draft.markdown.isEmpty else { continue }
            for section in draft.sections {
                guard Self.sha256(section.markdown) == section.contentHash else { throw ContextStoreError.invalidHash }
                let vector: [Float]
                if let existing = reuse["\(section.sectionId)\u{0}\(section.contentHash)"] { vector = existing }
                else { vector = try provider.embed(section.markdown) }
                rows.append(ContextSection(diagramID:diagramID,contextID:draft.contextId,nodeID:draft.nodeId,nodeType:draft.nodeType,nodeLabel:draft.nodeLabel,sectionID:section.sectionId,headingPath:section.headingPath,ordinal:section.ordinal,markdown:section.markdown,contentHash:section.contentHash,embedding:vector,embeddingProvider:provider.provider,embeddingModel:provider.model,capturedAt:capturedAt,captureRevision:revision,isTombstone:false))
            }
        }
        for nodeID in payload.tombstones { rows.append(ContextSection(diagramID:diagramID,contextID:"context-\(nodeID)",nodeID:nodeID,nodeType:"deleted",nodeLabel:"",sectionID:"tombstone-\(nodeID)-\(revision)",headingPath:[],ordinal:0,markdown:"",contentHash:String(repeating:"0",count:64),embedding:[],embeddingProvider:"",embeddingModel:"",capturedAt:capturedAt,captureRevision:revision,isTombstone:true)) }
        try store.commit(revision:revision,diagramID:diagramID,sections:rows)
        manifest["revision"] = revision; diagram["contextStore"] = manifest
        let updated = String(data:try JSONSerialization.data(withJSONObject:diagram,options:[.prettyPrinted,.sortedKeys]),encoding:.utf8)! + "\n"
        let metadata = try fileService.atomicWrite(updated,to:diagramURL,root:root,expected:expected)
        return (ContextCaptureResult(revision:revision,diagram:updated),metadata)
    }

    static func sha256(_ text: String) -> String { SHA256.hash(data:Data(text.utf8)).map { String(format:"%02x",$0) }.joined() }
}
