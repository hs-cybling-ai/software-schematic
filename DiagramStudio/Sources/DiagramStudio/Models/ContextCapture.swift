import Foundation
import NaturalLanguage

struct ContextDraftSectionPayload: Codable, Sendable {
    let sectionId: String, markdown: String, contentHash: String
    let headingPath: [String]
    let ordinal: Int
}
struct ContextDraftPayload: Codable, Sendable {
    let contextId: String, nodeId: String, nodeType: String, nodeLabel: String, markdown: String
    let sections: [ContextDraftSectionPayload]
}
struct ContextCapturePayload: Codable, Sendable {
    let diagram: String
    let drafts: [ContextDraftPayload]
    let tombstones: [String]
}
struct ContextCaptureResult: Codable, Sendable { let revision: Int; let diagram: String }

protocol EmbeddingProvider: Sendable {
    var provider: String { get }
    var model: String { get }
    func embed(_ text: String) throws -> [Float]
}

struct AppleSentenceEmbeddingProvider: EmbeddingProvider {
    let provider = "apple-natural-language"
    let model = "sentence-en"
    func embed(_ text: String) throws -> [Float] {
        guard let embedding = NLEmbedding.sentenceEmbedding(for: .english), let vector = embedding.vector(for: text), !vector.isEmpty else { throw ContextStoreError.invalidEmbedding }
        return vector.map(Float.init)
    }
}

struct DeterministicEmbeddingProvider: EmbeddingProvider {
    let provider = "fixture", model = "deterministic-8"
    func embed(_ text: String) throws -> [Float] {
        var values = Array(repeating: Float(0), count: 8)
        for (index, byte) in text.utf8.enumerated() { values[index % values.count] += Float(byte) / 255 }
        let magnitude = sqrt(values.reduce(0) { $0 + $1 * $1 })
        return magnitude == 0 ? values : values.map { $0 / magnitude }
    }
}

struct UnavailableEmbeddingProvider: EmbeddingProvider {
    let provider = "unavailable", model = "unavailable"
    func embed(_ text: String) throws -> [Float] { throw ContextStoreError.invalidEmbedding }
}
