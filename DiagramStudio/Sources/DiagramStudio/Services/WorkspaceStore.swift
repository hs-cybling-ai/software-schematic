import AppKit
import Foundation

struct DiagramDestination: Identifiable, Hashable {
    let url: URL
    let label: String
    let depth: Int

    var id: URL { url }
    var displayLabel: String { String(repeating: "  ", count: depth) + label }
}

@MainActor
final class WorkspaceStore: ObservableObject {
    enum ConflictDecision { case overwrite, reload, cancel }
    enum UnsavedDecision { case save, discard, cancel }
    @Published private(set) var rootURL: URL?
    @Published private(set) var fileTree: [FileNode] = []
    @Published private(set) var tabs: [EditorTab] = []
    @Published var selectedTabID: UUID?
    @Published var workspaceError: String?
    @Published var openTabWarning: String?
    @Published var isPresentingNewDiagram = false
    @Published var newDiagramError: String?

    let fileService: WorkspaceFileService
    let bookmarks: SecurityScopedBookmarkStore
    private let watcher = FolderWatcher()
    var conflictDecisionProvider: ((EditorTab) -> ConflictDecision)?
    var unsavedDecisionProvider: ((EditorTab, String) -> UnsavedDecision)?

    init(fileService: WorkspaceFileService = WorkspaceFileService(), bookmarks: SecurityScopedBookmarkStore = SecurityScopedBookmarkStore()) {
        self.fileService = fileService
        self.bookmarks = bookmarks
    }

    var selectedTab: EditorTab? { tabs.first { $0.id == selectedTabID } }
    var canSave: Bool { selectedTab?.isDirty == true && selectedTab?.isReady == true }
    var canSaveAll: Bool { tabs.contains { $0.isDirty && $0.isReady } }
    var canCreateDiagram: Bool { rootURL != nil }
    var diagramDestinations: [DiagramDestination] {
        guard let rootURL else { return [] }
        var result = [DiagramDestination(url: rootURL, label: "Workspace Root", depth: 0)]
        func appendDirectories(_ nodes: [FileNode], depth: Int) {
            for node in nodes where node.isDirectory {
                result.append(DiagramDestination(url: node.url, label: node.name, depth: depth))
                appendDirectories(node.children ?? [], depth: depth + 1)
            }
        }
        appendDirectories(fileTree, depth: 1)
        return result
    }

    func beginNewDiagram() {
        guard canCreateDiagram else { return }
        newDiagramError = nil
        isPresentingNewDiagram = true
    }

    func cancelNewDiagram() {
        isPresentingNewDiagram = false
        newDiagramError = nil
    }

    func clearNewDiagramError() { newDiagramError = nil }

    func creationValidationMessage(name: String, format: DiagramFormat, destination: URL?) -> String? {
        guard let rootURL, let destination else { return WorkspaceError.invalidDestination.localizedDescription }
        do {
            let url = try fileService.creationURL(name: name, format: format, destination: destination, root: rootURL)
            if FileManager.default.fileExists(atPath: url.path) { return WorkspaceError.fileAlreadyExists.localizedDescription }
            return nil
        } catch { return error.localizedDescription }
    }

    @discardableResult
    func createDiagram(name: String, format: DiagramFormat, destination: URL) -> Bool {
        guard let rootURL else { return false }
        do {
            let createdURL = try fileService.createDiagram(name: name, format: format, destination: destination, root: rootURL)
            fileTree = try fileService.discover(root: rootURL)
            guard let node = findNode(url: createdURL, in: fileTree) else {
                throw WorkspaceError.missingFile
            }
            open(node)
            guard tabs.contains(where: { fileService.canonical($0.url) == createdURL }) else {
                if workspaceError == nil { workspaceError = "The diagram was created but could not be opened." }
                return false
            }
            isPresentingNewDiagram = false
            newDiagramError = nil
            return true
        } catch {
            newDiagramError = error.localizedDescription
            return false
        }
    }

    func restoreWorkspace() {
        do {
            guard let restored = try bookmarks.restore() else { return }
            guard !restored.stale else { workspaceError = "Folder access expired. Choose the folder again."; return }
            _ = bookmarks.startAccessing(restored.url)
            try activate(restored.url)
        } catch { workspaceError = error.localizedDescription }
    }

    func chooseFolder() {
        guard confirmDirtyTabs(reason: "open another folder") else { return }
        let panel = NSOpenPanel()
        panel.canChooseDirectories = true
        panel.canChooseFiles = false
        panel.allowsMultipleSelection = false
        panel.prompt = "Open"
        guard panel.runModal() == .OK, let url = panel.url else { return }
        do {
            try bookmarks.persist(url)
            _ = bookmarks.startAccessing(url)
            try activate(url)
        } catch { workspaceError = error.localizedDescription }
    }

    func activate(_ url: URL) throws {
        let canonical = fileService.canonical(url)
        guard FileManager.default.fileExists(atPath: canonical.path) else { throw WorkspaceError.missingFile }
        tabs.removeAll()
        selectedTabID = nil
        rootURL = canonical
        refresh()
        watcher.watch(canonical) { [weak self] in self?.refresh() }
    }

    func refresh() {
        guard let rootURL else { return }
        do { fileTree = try fileService.discover(root: rootURL); workspaceError = nil }
        catch { workspaceError = error.localizedDescription }
    }

    func open(_ node: FileNode) {
        guard !node.isDirectory, let rootURL, let format = node.format else { return }
        let canonical = fileService.canonical(node.url)
        if let existing = tabs.first(where: { fileService.canonical($0.url) == canonical }) {
            selectedTabID = existing.id
            return
        }
        do {
            let (xml, metadata) = try fileService.read(canonical, root: rootURL)
            let controller = EditorWebViewController()
            let webView = controller.makeWebView()
            let tab = EditorTab(url: canonical, format: format, webView: webView, metadata: metadata)
            tab.controller = controller
            tab.exportXMLForSave = { [weak controller] in
                guard let controller else { throw BridgeError.webContentUnavailable }
                return try await controller.requestExport()
            }
            controller.tab = tab
            controller.captureHandler = { [weak self, weak tab] payload in
                guard let self, let tab, let rootURL = self.rootURL else { throw BridgeError.webContentUnavailable }
                let (result, metadata) = try ContextCaptureService(fileService:self.fileService).capture(payload,diagramURL:tab.url,root:rootURL,expected:tab.metadata)
                tab.metadata = metadata
                return result
            }
            controller.sourceXML = xml
            if format == .dataGraph { controller.sourceContexts = try loadCapturedContexts(diagram:xml,diagramURL:canonical,root:rootURL) }
            tabs.append(tab)
            selectedTabID = tab.id
            if tabs.count > 12 { openTabWarning = "Many open diagrams may use significant memory." }
            try controller.loadHost(in: webView)
        } catch { workspaceError = error.localizedDescription }
    }

    private func loadCapturedContexts(diagram: String, diagramURL: URL, root: URL) throws -> [ContextDraftPayload] {
        guard let value = try JSONSerialization.jsonObject(with:Data(diagram.utf8)) as? [String:Any], let manifest=value["contextStore"] as? [String:Any], let revision=manifest["revision"] as? Int, revision > 0, let uri=manifest["uri"] as? String, let diagramID=manifest["diagramId"] as? String else { return [] }
        let databaseURL=try fileService.contextDatabaseURL(for:diagramURL,manifestURI:uri,root:root)
        let rows=try SQLiteContextStore(url:databaseURL,diagramID:diagramID).sections(revision:revision).filter{!$0.isTombstone}
        return Dictionary(grouping:rows,by:\.contextID).values.map { group in
            let ordered=group.sorted{$0.ordinal<$1.ordinal}; let first=ordered[0]
            return ContextDraftPayload(contextId:first.contextID,nodeId:first.nodeID,nodeType:first.nodeType,nodeLabel:first.nodeLabel,markdown:ordered.map(\.markdown).joined(),sections:ordered.map{ContextDraftSectionPayload(sectionId:$0.sectionID,markdown:$0.markdown,contentHash:$0.contentHash,headingPath:$0.headingPath,ordinal:$0.ordinal)})
        }.sorted{$0.nodeId<$1.nodeId}
    }

    func saveSelected(overwriteConflict: Bool = false) async -> Bool {
        guard let selectedTab else { return true }
        return await save(selectedTab, overwriteConflict: overwriteConflict)
    }

    func save(_ tab: EditorTab, overwriteConflict: Bool = false) async -> Bool {
        guard tab.isDirty, let rootURL else { return true }
        do {
            guard let exportXMLForSave = tab.exportXMLForSave else { throw BridgeError.webContentUnavailable }
            let xml = try await exportXMLForSave()
            tab.metadata = try fileService.atomicWrite(xml, to: tab.url, root: rootURL, expected: tab.metadata, overwriteConflict: overwriteConflict)
            tab.isDirty = false
            tab.errorMessage = nil
            return true
        } catch {
            if let workspaceError = error as? WorkspaceError, workspaceError == .sourceChanged {
                switch conflictDecisionProvider?(tab) ?? presentConflict(for: tab) {
                case .overwrite: return await save(tab, overwriteConflict: true)
                case .reload:
                    reload(tab)
                    return false
                case .cancel: break
                }
            }
            tab.errorMessage = error.localizedDescription
            return false
        }
    }

    func saveAll() async -> [URL: Error] {
        var failures: [URL: Error] = [:]
        for tab in tabs where tab.isDirty {
            if !(await save(tab)) { failures[tab.url] = BridgeError.exportFailed(tab.errorMessage ?? "Save failed") }
        }
        if !failures.isEmpty { workspaceError = "Some diagrams could not be saved." }
        return failures
    }

    func close(_ tab: EditorTab) {
        guard confirmDirtyTab(tab) else { return }
        tab.webView.stopLoading()
        tab.webView.configuration.userContentController.removeScriptMessageHandler(forName: "diagramBridge")
        tab.pendingExports.values.forEach { $0.resume(throwing: CancellationError()) }
        tab.pendingExports.removeAll()
        tabs.removeAll { $0.id == tab.id }
        if selectedTabID == tab.id { selectedTabID = tabs.last?.id }
        if tabs.count <= 12 { openTabWarning = nil }
    }

    func closeSelected() { if let selectedTab { close(selectedTab) } }
    func confirmTermination() -> Bool { confirmDirtyTabs(reason: "quit") }

    func reload(_ tab: EditorTab) {
        guard let rootURL else { return }
        do {
            let (xml, metadata) = try fileService.read(tab.url, root: rootURL)
            tab.metadata = metadata
            tab.isDirty = false
            tab.errorMessage = nil
            Task {
                do { let contexts = tab.format == .dataGraph ? try loadCapturedContexts(diagram:xml,diagramURL:tab.url,root:rootURL) : []; try await tab.controller.send(type: "load", requestId: UUID().uuidString, format: tab.format, payload: LoadPayload(xml: xml, contexts: contexts), to: tab.webView) }
                catch { tab.errorMessage = error.localizedDescription }
            }
        } catch { tab.errorMessage = error.localizedDescription }
    }

    private func findNode(url: URL, in nodes: [FileNode]) -> FileNode? {
        let target = fileService.canonical(url)
        for node in nodes {
            if !node.isDirectory, fileService.canonical(node.url) == target { return node }
            if let match = findNode(url: target, in: node.children ?? []) { return match }
        }
        return nil
    }

    private func confirmDirtyTabs(reason: String) -> Bool {
        for tab in tabs where tab.isDirty {
            guard confirmDirtyTab(tab, reason: reason) else { return false }
        }
        return true
    }

    private func confirmDirtyTab(_ tab: EditorTab, reason: String = "close this tab") -> Bool {
        guard tab.isDirty else { return true }
        switch unsavedDecisionProvider?(tab, reason) ?? presentUnsavedDecision(for: tab, reason: reason) {
        case .save:
            let semaphore = DispatchSemaphore(value: 0)
            var succeeded = false
            Task { succeeded = await save(tab); semaphore.signal() }
            while semaphore.wait(timeout: .now() + 0.01) == .timedOut { RunLoop.current.run(until: Date().addingTimeInterval(0.01)) }
            return succeeded
        case .discard: return true
        case .cancel: return false
        }
    }

    private func presentConflict(for tab: EditorTab) -> ConflictDecision {
        let alert = NSAlert()
        alert.messageText = "\(tab.title) changed on disk"
        alert.informativeText = "Overwrite the external version, reload it and discard your edits, or cancel the save."
        alert.addButton(withTitle: "Overwrite")
        alert.addButton(withTitle: "Reload")
        alert.addButton(withTitle: "Cancel")
        switch alert.runModal() {
        case .alertFirstButtonReturn: return ConflictDecision.overwrite
        case .alertSecondButtonReturn: return ConflictDecision.reload
        default: return ConflictDecision.cancel
        }
    }

    private func presentUnsavedDecision(for tab: EditorTab, reason: String) -> UnsavedDecision {
        let alert = NSAlert()
        alert.messageText = "Save changes to \(tab.title)?"
        alert.informativeText = "Your changes will be lost if you \(reason) without saving."
        alert.addButton(withTitle: "Save")
        alert.addButton(withTitle: "Don’t Save")
        alert.addButton(withTitle: "Cancel")
        switch alert.runModal() {
        case .alertFirstButtonReturn: return UnsavedDecision.save
        case .alertSecondButtonReturn: return UnsavedDecision.discard
        default: return UnsavedDecision.cancel
        }
    }
}
