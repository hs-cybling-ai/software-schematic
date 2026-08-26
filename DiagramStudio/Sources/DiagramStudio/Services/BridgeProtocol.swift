import Foundation

struct BridgeEnvelope<Payload: Codable>: Codable {
    let version: Int
    let type: String
    let requestId: String?
    let format: String?
    let payload: Payload
}

struct EmptyPayload: Codable {}
struct LoadPayload: Codable { let xml: String; let contexts: [ContextDraftPayload]?; init(xml: String, contexts: [ContextDraftPayload]? = nil) { self.xml = xml; self.contexts = contexts } }
struct ExportPayload: Codable { let xml: String }
struct ErrorPayload: Codable { let message: String }
struct WarningsPayload: Codable { let warnings: [String] }
struct CaptureCompletedPayload: Codable { let revision: Int; let diagram: String }

enum BridgeError: LocalizedError {
    case invalidMessage
    case unsupportedVersion
    case exportFailed(String)
    case webContentUnavailable

    var errorDescription: String? {
        switch self {
        case .invalidMessage: "The editor sent an invalid message."
        case .unsupportedVersion: "The editor bridge version is not supported."
        case .exportFailed(let message): message
        case .webContentUnavailable: "Bundled editor assets are missing."
        }
    }
}
