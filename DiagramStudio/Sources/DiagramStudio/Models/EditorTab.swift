import Foundation
import WebKit

struct SourceMetadata: Equatable, Sendable {
    let modificationDate: Date?
    let fileSize: Int?
}

@MainActor
final class EditorTab: ObservableObject, Identifiable {
    let id = UUID()
    let url: URL
    let format: DiagramFormat
    let webView: WKWebView
    var controller: EditorWebViewController!
    @Published var isDirty = false
    @Published var isReady = false
    @Published var diagnostics: [String] = []
    @Published var errorMessage: String?
    var metadata: SourceMetadata
    var pendingExports: [String: CheckedContinuation<String, Error>] = [:]
    var exportXMLForSave: (() async throws -> String)?

    init(url: URL, format: DiagramFormat, webView: WKWebView, metadata: SourceMetadata) {
        self.url = url
        self.format = format
        self.webView = webView
        self.metadata = metadata
    }

    var title: String { url.lastPathComponent }
}
