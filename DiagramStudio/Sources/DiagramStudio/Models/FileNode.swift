import Foundation

struct FileNode: Identifiable, Hashable, Sendable {
    let url: URL
    let relativePath: String
    let isDirectory: Bool
    var children: [FileNode]?

    var id: String { relativePath }
    var name: String { url.lastPathComponent }
    var format: DiagramFormat? { DiagramFormat.detect(url: url) }

    static func sorted(_ nodes: [FileNode]) -> [FileNode] {
        nodes.sorted {
            if $0.isDirectory != $1.isDirectory { return $0.isDirectory }
            return $0.name.localizedStandardCompare($1.name) == .orderedAscending
        }
    }
}
