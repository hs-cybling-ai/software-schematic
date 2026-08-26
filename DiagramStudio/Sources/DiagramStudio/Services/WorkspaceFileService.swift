import Foundation

enum WorkspaceError: LocalizedError, Equatable {
    case outsideWorkspace
    case unsupportedFormat
    case fileTooLarge
    case sourceChanged
    case missingFile
    case invalidDiagramName
    case invalidDestination
    case fileAlreadyExists

    var errorDescription: String? {
        switch self {
        case .outsideWorkspace: "The selected path is outside the authorized workspace."
        case .unsupportedFormat: "This diagram format is not supported."
        case .fileTooLarge: "The diagram exceeds the 20 MB safety limit."
        case .sourceChanged: "The file changed on disk after it was opened."
        case .missingFile: "The file is no longer available."
        case .invalidDiagramName: "Enter a filename without folders and use an extension that matches the selected diagram type."
        case .invalidDestination: "Choose an available folder inside the current workspace."
        case .fileAlreadyExists: "A file or folder with that name already exists. Choose another name or destination."
        }
    }
}

struct WorkspaceFileService {
    let fileManager: FileManager

    init(fileManager: FileManager = .default) { self.fileManager = fileManager }

    func canonical(_ url: URL) -> URL { url.standardizedFileURL.resolvingSymlinksInPath() }

    func resolvedFilename(_ name: String, format: DiagramFormat) throws -> String {
        let trimmed = name.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty,
              trimmed != ".", trimmed != "..",
              !trimmed.contains("/"), !trimmed.contains("\\"),
              !trimmed.contains("\0") else { throw WorkspaceError.invalidDiagramName }

        let candidate = URL(fileURLWithPath: trimmed).lastPathComponent
        if DiagramFormat.detect(url: URL(fileURLWithPath: candidate)) == format { return candidate }
        if candidate.contains(".") { throw WorkspaceError.invalidDiagramName }
        return candidate + "." + format.canonicalExtension
    }

    func creationURL(name: String, format: DiagramFormat, destination: URL, root: URL) throws -> URL {
        let canonicalRoot = canonical(root)
        let canonicalDestination = canonical(destination)
        var isDirectory: ObjCBool = false
        guard contains(canonicalDestination, root: canonicalRoot),
              fileManager.fileExists(atPath: canonicalDestination.path, isDirectory: &isDirectory),
              isDirectory.boolValue else { throw WorkspaceError.invalidDestination }
        return canonicalDestination.appendingPathComponent(try resolvedFilename(name, format: format), isDirectory: false)
    }

    @discardableResult
    func createDiagram(name: String, format: DiagramFormat, destination: URL, root: URL) throws -> URL {
        let url = try creationURL(name: name, format: format, destination: destination, root: root)
        do {
            let source: String
            if format == .dataGraph {
                let contextName = url.deletingPathExtension().lastPathComponent + ".context.sqlite"
                let manifest: [String: Any] = ["format": "diagram-studio/data-graph", "version": 2, "nodes": [], "edges": [], "properties": [], "contextStore": ["kind": "sqlite", "uri": contextName, "schemaVersion": 1, "diagramId": UUID().uuidString.lowercased(), "revision": 0]]
                source = String(data: try JSONSerialization.data(withJSONObject: manifest, options: [.prettyPrinted, .sortedKeys]), encoding: .utf8)! + "\n"
            } else { source = format.starterXML }
            try Data(source.utf8).write(to: url, options: .withoutOverwriting)
            return canonical(url)
        } catch let error as CocoaError where error.code == .fileWriteFileExists {
            throw WorkspaceError.fileAlreadyExists
        }
    }

    func contextDatabaseURL(for diagramURL: URL, manifestURI: String, root: URL) throws -> URL {
        guard DiagramFormat.detect(url: diagramURL) == .dataGraph,
              !manifestURI.isEmpty, !manifestURI.contains("/"), !manifestURI.contains("\\"),
              manifestURI.hasSuffix(".context.sqlite") else { throw WorkspaceError.invalidDiagramName }
        let resolved = canonical(diagramURL.deletingLastPathComponent().appendingPathComponent(manifestURI))
        guard contains(resolved, root: root), resolved.deletingLastPathComponent() == canonical(diagramURL.deletingLastPathComponent()) else { throw WorkspaceError.outsideWorkspace }
        return resolved
    }

    func migratedDataGraphSource(_ source: String, url: URL) throws -> String {
        guard var object = try JSONSerialization.jsonObject(with: Data(source.utf8)) as? [String: Any] else { throw WorkspaceError.unsupportedFormat }
        guard object["format"] as? String == "diagram-studio/data-graph" else { return source }
        if object["version"] as? Int == 1 {
            let base = url.lastPathComponent.replacingOccurrences(of: ".dgraph.json", with: "").replacingOccurrences(of: ".dgraph", with: "")
            object["version"] = 2
            object["contextStore"] = ["kind": "sqlite", "uri": base + ".context.sqlite", "schemaVersion": 1, "diagramId": stableDiagramID(for: url), "revision": 0]
            return String(data: try JSONSerialization.data(withJSONObject: object, options: [.prettyPrinted, .sortedKeys]), encoding: .utf8)! + "\n"
        }
        return source
    }

    func migrateLegacyDataGraph(_ legacyURL: URL, root: URL) throws -> URL {
        guard legacyURL.lastPathComponent.lowercased().hasSuffix(".dgraph.json"), contains(legacyURL, root: root) else { throw WorkspaceError.unsupportedFormat }
        let name = String(legacyURL.lastPathComponent.dropLast(".dgraph.json".count))
        let target = legacyURL.deletingLastPathComponent().appendingPathComponent(name + ".dgraph")
        let database = legacyURL.deletingLastPathComponent().appendingPathComponent(name + ".context.sqlite")
        guard !fileManager.fileExists(atPath: target.path), !fileManager.fileExists(atPath: database.path) else { throw WorkspaceError.fileAlreadyExists }
        let source = try String(contentsOf: legacyURL, encoding: .utf8)
        let migrated = try migratedDataGraphSource(source, url: target)
        try Data(migrated.utf8).write(to: target, options: .withoutOverwriting)
        return canonical(target)
    }

    private func stableDiagramID(for url: URL) -> String {
        var hash: UInt64 = 14_695_981_039_346_656_037
        for byte in canonical(url).path.utf8 { hash = (hash ^ UInt64(byte)) &* 1_099_511_628_211 }
        return String(format: "migrated-%016llx", hash)
    }

    func contains(_ url: URL, root: URL) -> Bool {
        let rootPath = canonical(root).path.hasSuffix("/") ? canonical(root).path : canonical(root).path + "/"
        let candidate = canonical(url).path
        return candidate == canonical(root).path || candidate.hasPrefix(rootPath)
    }

    func metadata(for url: URL) throws -> SourceMetadata {
        guard fileManager.fileExists(atPath: url.path) else { throw WorkspaceError.missingFile }
        let attributes = try fileManager.attributesOfItem(atPath: url.path)
        return SourceMetadata(
            modificationDate: attributes[.modificationDate] as? Date,
            fileSize: (attributes[.size] as? NSNumber)?.intValue
        )
    }

    func read(_ url: URL, root: URL) throws -> (String, SourceMetadata) {
        guard contains(url, root: root) else { throw WorkspaceError.outsideWorkspace }
        guard DiagramFormat.detect(url: url) != nil else { throw WorkspaceError.unsupportedFormat }
        let current = try metadata(for: url)
        if (current.fileSize ?? 0) > DiagramFormat.maximumFileSize { throw WorkspaceError.fileTooLarge }
        let source = try String(contentsOf: url, encoding: .utf8)
        return (DiagramFormat.detect(url: url) == .dataGraph ? try migratedDataGraphSource(source, url: url) : source, current)
    }

    func discover(root: URL) throws -> [FileNode] {
        try children(of: canonical(root), root: canonical(root))
    }

    private func children(of directory: URL, root: URL) throws -> [FileNode] {
        let keys: Set<URLResourceKey> = [.isDirectoryKey, .isSymbolicLinkKey]
        let urls = try fileManager.contentsOfDirectory(at: directory, includingPropertiesForKeys: Array(keys), options: [.skipsHiddenFiles])
        var nodes: [FileNode] = []
        for url in urls {
            let values = try url.resourceValues(forKeys: keys)
            let resolved = canonical(url)
            guard contains(resolved, root: root) else { continue }
            if values.isDirectory == true {
                let descendants = try children(of: resolved, root: root)
                if !descendants.isEmpty {
                    nodes.append(FileNode(url: resolved, relativePath: relative(resolved, root: root), isDirectory: true, children: descendants))
                }
            } else if DiagramFormat.detect(url: resolved) != nil {
                nodes.append(FileNode(url: resolved, relativePath: relative(resolved, root: root), isDirectory: false, children: nil))
            }
        }
        return FileNode.sorted(nodes)
    }

    private func relative(_ url: URL, root: URL) -> String {
        String(canonical(url).path.dropFirst(canonical(root).path.count)).trimmingCharacters(in: CharacterSet(charactersIn: "/"))
    }

    func atomicWrite(_ xml: String, to url: URL, root: URL, expected: SourceMetadata, overwriteConflict: Bool = false) throws -> SourceMetadata {
        guard contains(url, root: root) else { throw WorkspaceError.outsideWorkspace }
        if !overwriteConflict, try metadata(for: url) != expected { throw WorkspaceError.sourceChanged }
        try Data(xml.utf8).write(to: url, options: .atomic)
        return try metadata(for: url)
    }
}
