import SwiftUI

struct FileTreeView: View {
    @EnvironmentObject private var workspace: WorkspaceStore

    var body: some View {
        Group {
            if workspace.rootURL == nil {
                ContentUnavailableView("No Folder Open", systemImage: "folder", description: Text("Choose File > Open Folder to browse diagrams."))
            } else if workspace.fileTree.isEmpty {
                ContentUnavailableView("No Diagrams", systemImage: "doc.text.magnifyingglass", description: Text("This folder has no supported BPMN or Data Graph files."))
            } else {
                List(workspace.fileTree, children: \.children) { node in
                    Label(node.name, systemImage: node.isDirectory ? "folder" : node.format == .bpmn ? "point.3.connected.trianglepath.dotted" : "square.3.layers.3d")
                        .contentShape(Rectangle())
                        .onTapGesture { workspace.open(node) }
                        .accessibilityLabel(node.isDirectory ? "Folder \(node.name)" : "Diagram \(node.name)")
                        .accessibilityIdentifier("file-\(node.relativePath)")
                }
                .listStyle(.sidebar)
            }
        }
        .navigationTitle(workspace.rootURL?.lastPathComponent ?? "Workspace")
        .frame(minWidth: 220)
        .accessibilityIdentifier("workspace-sidebar")
    }
}
