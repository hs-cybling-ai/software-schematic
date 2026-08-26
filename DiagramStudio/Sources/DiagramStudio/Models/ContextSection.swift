import Foundation

struct ContextLimits: Sendable {
    var maximumMarkdownBytes = 1_048_576
    var maximumSections = 2_048
    var maximumEmbeddingDimensions = 16_384
    var maximumDatabaseBytes = 512 * 1_024 * 1_024
}

enum ContextStoreError: LocalizedError, Equatable {
    case invalidIdentifier(String), invalidHash, invalidEmbedding, invalidRevision
    case schemaMismatch, diagramMismatch, databaseTooLarge, sqlite(String)

    var errorDescription: String? {
        switch self {
        case .invalidIdentifier(let field): "Context has an invalid \(field)."
        case .invalidHash: "Context section has an invalid SHA-256 content hash."
        case .invalidEmbedding: "Context section embedding is missing, non-finite, or has the wrong dimensions."
        case .invalidRevision: "The requested context capture revision is incomplete or unavailable."
        case .schemaMismatch: "The context database schema is unsupported."
        case .diagramMismatch: "The context database belongs to a different diagram."
        case .databaseTooLarge: "The context database exceeds its configured size limit."
        case .sqlite(let message): "SQLite context store failed: \(message)"
        }
    }
}

struct ContextSection: Codable, Equatable, Sendable {
    let diagramID: String, contextID: String, nodeID: String, nodeType: String, nodeLabel: String, sectionID: String
    let headingPath: [String]
    let ordinal: Int
    let markdown: String, contentHash: String
    let embedding: [Float]
    let embeddingProvider: String, embeddingModel: String
    let capturedAt: Date
    let captureRevision: Int
    let isTombstone: Bool

    var embeddingDimensions: Int { embedding.count }

    func validated(limits: ContextLimits = ContextLimits()) throws -> ContextSection {
        for (name, value) in [("diagram ID", diagramID), ("context ID", contextID), ("node ID", nodeID), ("node type", nodeType), ("section ID", sectionID)] where value.isEmpty { throw ContextStoreError.invalidIdentifier(name) }
        guard ordinal >= 0, captureRevision > 0 else { throw ContextStoreError.invalidRevision }
        guard markdown.utf8.count <= limits.maximumMarkdownBytes else { throw ContextStoreError.databaseTooLarge }
        guard contentHash.range(of: "^[0-9a-f]{64}$", options: .regularExpression) != nil else { throw ContextStoreError.invalidHash }
        if !isTombstone {
            guard !markdown.isEmpty, !embeddingProvider.isEmpty, !embeddingModel.isEmpty, !embedding.isEmpty,
                  embedding.count <= limits.maximumEmbeddingDimensions, embedding.allSatisfy(\.isFinite) else { throw ContextStoreError.invalidEmbedding }
        }
        return self
    }

    var embeddingData: Data { embedding.withUnsafeBufferPointer { Data(buffer: $0) } }

    static func embedding(from data: Data, dimensions: Int) throws -> [Float] {
        guard dimensions >= 0, data.count == dimensions * MemoryLayout<Float>.size else { throw ContextStoreError.invalidEmbedding }
        let values = data.withUnsafeBytes { Array($0.bindMemory(to: Float.self)) }
        guard values.allSatisfy(\.isFinite) else { throw ContextStoreError.invalidEmbedding }
        return values
    }
}
