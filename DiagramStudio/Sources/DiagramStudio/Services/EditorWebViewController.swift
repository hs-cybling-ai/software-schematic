import Foundation
import WebKit

@MainActor
final class EditorWebViewController: NSObject, WKScriptMessageHandler, WKNavigationDelegate, WKUIDelegate {
    weak var tab: EditorTab?
    var sourceXML: String?
    var sourceContexts: [ContextDraftPayload]?
    private let decoder = JSONDecoder()
    private let encoder = JSONEncoder()
    private var exportInFlight: Task<String, Error>?
    var captureHandler: ((ContextCapturePayload) async throws -> ContextCaptureResult)?

    func makeWebView() -> WKWebView {
        let content = WKUserContentController()
        content.add(self, name: "diagramBridge")
        let configuration = WKWebViewConfiguration()
        configuration.userContentController = content
        configuration.websiteDataStore = .nonPersistent()
        configuration.preferences.isElementFullscreenEnabled = false
        let webView = WKWebView(frame: .zero, configuration: configuration)
        webView.navigationDelegate = self
        webView.uiDelegate = self
        webView.setValue(false, forKey: "drawsBackground")
        return webView
    }

    func loadHost(in webView: WKWebView) throws {
        #if SWIFT_PACKAGE
        let resources = Bundle.module
        #else
        let resources = Bundle.main
        #endif
        guard let url = resources.url(forResource: "index", withExtension: "html", subdirectory: "Web") else { throw BridgeError.webContentUnavailable }
        webView.loadFileURL(url, allowingReadAccessTo: url.deletingLastPathComponent())
    }

    func send<P: Codable>(type: String, requestId: String? = nil, format: DiagramFormat?, payload: P, to webView: WKWebView) async throws {
        let message = BridgeEnvelope(version: 1, type: type, requestId: requestId, format: format?.rawValue, payload: payload)
        let data = try encoder.encode(message)
        let object = try JSONSerialization.jsonObject(with: data)
        let result = try await webView.callAsyncJavaScript(
            "await window.diagramStudioReceive(message); return true",
            arguments: ["message": object],
            in: nil,
            contentWorld: .page
        )
        guard result as? Bool == true else { throw BridgeError.invalidMessage }
    }

    func requestExport() async throws -> String {
        if let exportInFlight { return try await exportInFlight.value }
        let task = Task { try await performExport() }
        exportInFlight = task
        defer { exportInFlight = nil }
        return try await task.value
    }

    private func performExport() async throws -> String {
        guard let tab else { throw BridgeError.invalidMessage }
        let requestId = UUID().uuidString
        return try await withCheckedThrowingContinuation { continuation in
            tab.pendingExports[requestId] = continuation
            Task {
                do { try await send(type: "requestExport", requestId: requestId, format: tab.format, payload: EmptyPayload(), to: tab.webView) }
                catch {
                    tab.pendingExports.removeValue(forKey: requestId)?.resume(throwing: error)
                }
            }
        }
    }

    func userContentController(_ userContentController: WKUserContentController, didReceive message: WKScriptMessage) {
        guard message.name == "diagramBridge", let tab,
              JSONSerialization.isValidJSONObject(message.body),
              let data = try? JSONSerialization.data(withJSONObject: message.body),
              let header = try? decoder.decode(BridgeEnvelope<[String: JSONValue]>.self, from: data), header.version == 1 else {
            tab?.errorMessage = BridgeError.invalidMessage.localizedDescription
            return
        }
        switch header.type {
        case "ready":
            if header.requestId == nil, let xml = sourceXML {
                sourceXML = nil
                let requestId = UUID().uuidString
                Task {
                    do { try await send(type: "load", requestId: requestId, format: tab.format, payload: LoadPayload(xml: xml, contexts: sourceContexts), to: tab.webView); sourceContexts = nil }
                    catch { tab.errorMessage = error.localizedDescription }
                }
            } else {
                tab.isReady = true
            }
        case "changed": tab.isDirty = true
        case "captureRequested":
            guard let request = try? decoder.decode(BridgeEnvelope<ContextCapturePayload>.self, from: data), let captureHandler else { tab.errorMessage = BridgeError.invalidMessage.localizedDescription; return }
            Task {
                do {
                    let result = try await captureHandler(request.payload)
                    try await send(type:"captureCompleted",requestId:request.requestId,format:tab.format,payload:CaptureCompletedPayload(revision:result.revision,diagram:result.diagram),to:tab.webView)
                    tab.isDirty = false; tab.errorMessage = nil
                } catch {
                    tab.errorMessage = error.localizedDescription
                    try? await send(type:"captureFailed",requestId:request.requestId,format:tab.format,payload:ErrorPayload(message:error.localizedDescription),to:tab.webView)
                }
            }
        case "warnings":
            if let payload = try? decoder.decode(BridgeEnvelope<WarningsPayload>.self, from: data) { tab.diagnostics = payload.payload.warnings }
        case "exported":
            if let payload = try? decoder.decode(BridgeEnvelope<ExportPayload>.self, from: data), let id = payload.requestId {
                tab.pendingExports.removeValue(forKey: id)?.resume(returning: payload.payload.xml)
            }
        case "failed":
            if let payload = try? decoder.decode(BridgeEnvelope<ErrorPayload>.self, from: data) {
                tab.errorMessage = payload.payload.message
                if let id = payload.requestId { tab.pendingExports.removeValue(forKey: id)?.resume(throwing: BridgeError.exportFailed(payload.payload.message)) }
            }
        default: tab.errorMessage = BridgeError.invalidMessage.localizedDescription
        }
    }

    func webView(_ webView: WKWebView, decidePolicyFor navigationAction: WKNavigationAction, decisionHandler: @escaping @MainActor (WKNavigationActionPolicy) -> Void) {
        let allowed = Self.isAllowedNavigationURL(navigationAction.request.url, targetIsMain: navigationAction.targetFrame?.isMainFrame != false)
        decisionHandler(allowed ? .allow : .cancel)
    }

    nonisolated static func isAllowedNavigationURL(_ url: URL?, targetIsMain: Bool) -> Bool {
        url?.isFileURL == true && targetIsMain
    }

    func webView(_ webView: WKWebView, createWebViewWith configuration: WKWebViewConfiguration, for navigationAction: WKNavigationAction, windowFeatures: WKWindowFeatures) -> WKWebView? { nil }
}

enum JSONValue: Codable {
    case string(String), number(Double), bool(Bool), object([String: JSONValue]), array([JSONValue]), null
    init(from decoder: Decoder) throws {
        let box = try decoder.singleValueContainer()
        if box.decodeNil() { self = .null }
        else if let value = try? box.decode(Bool.self) { self = .bool(value) }
        else if let value = try? box.decode(Double.self) { self = .number(value) }
        else if let value = try? box.decode(String.self) { self = .string(value) }
        else if let value = try? box.decode([String: JSONValue].self) { self = .object(value) }
        else { self = .array(try box.decode([JSONValue].self)) }
    }
    func encode(to encoder: Encoder) throws {
        var box = encoder.singleValueContainer()
        switch self {
        case .string(let value): try box.encode(value)
        case .number(let value): try box.encode(value)
        case .bool(let value): try box.encode(value)
        case .object(let value): try box.encode(value)
        case .array(let value): try box.encode(value)
        case .null: try box.encodeNil()
        }
    }
}
