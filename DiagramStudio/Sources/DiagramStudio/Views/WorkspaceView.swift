import SwiftUI

struct WorkspaceView: View {
    @EnvironmentObject private var workspace: WorkspaceStore

    var body: some View {
        NavigationSplitView {
            FileTreeView()
        } detail: {
            VStack(spacing: 0) {
                if !workspace.tabs.isEmpty { TabBarView() }
                if let tab = workspace.selectedTab {
                    ZStack(alignment: .bottomLeading) {
                        EditorWebView(webView: tab.webView).id(tab.id)
                        if let error = tab.errorMessage {
                            Label(error, systemImage: "exclamationmark.triangle.fill")
                                .padding(10).background(.red.opacity(0.85), in: RoundedRectangle(cornerRadius: 8)).padding()
                        } else if !tab.diagnostics.isEmpty {
                            Label("\(tab.diagnostics.count) editor warning(s)", systemImage: "exclamationmark.triangle")
                                .padding(8).background(.yellow.opacity(0.2), in: RoundedRectangle(cornerRadius: 8)).padding()
                        }
                    }
                } else {
                    ContentUnavailableView("No Diagram Selected", systemImage: "square.3.layers.3d", description: Text("Double-click a supported diagram in the sidebar."))
                }
            }
            .frame(minWidth: 520, minHeight: 420)
        }
        .navigationSplitViewStyle(.balanced)
        .accessibilityIdentifier("workspace-split-view")
        .preferredColorScheme(.dark)
        .sheet(isPresented: $workspace.isPresentingNewDiagram) {
            NewDiagramSheet()
                .environmentObject(workspace)
        }
        .alert("Workspace Error", isPresented: Binding(get: { workspace.workspaceError != nil }, set: { if !$0 { workspace.workspaceError = nil } })) {
            Button("OK") { workspace.workspaceError = nil }
        } message: { Text(workspace.workspaceError ?? "") }
        .overlay(alignment: .topTrailing) {
            if let warning = workspace.openTabWarning { Text(warning).font(.caption).padding(8).background(.orange.opacity(0.25), in: Capsule()).padding() }
        }
    }
}

private struct NewDiagramSheet: View {
    @EnvironmentObject private var workspace: WorkspaceStore
    @State private var name = ""
    @State private var format = DiagramFormat.bpmn
    @State private var destination: URL?

    private var validationMessage: String? {
        workspace.newDiagramError ?? workspace.creationValidationMessage(name: name, format: format, destination: destination)
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 18) {
            Text("New Diagram").font(.title2.bold())
            Form {
                Picker("Type", selection: $format) {
                    ForEach(DiagramFormat.allCases, id: \.self) { format in
                        Text(format.displayName).tag(format)
                    }
                }
                .accessibilityIdentifier("new-diagram-format")

                TextField("Name", text: $name, prompt: Text("Diagram name"))
                    .accessibilityIdentifier("new-diagram-name")

                Picker("Place in", selection: $destination) {
                    ForEach(workspace.diagramDestinations) { destination in
                        Text(destination.displayLabel).tag(Optional(destination.url))
                    }
                }
                .accessibilityIdentifier("new-diagram-destination")
            }
            .formStyle(.grouped)

            if let validationMessage {
                Label(validationMessage, systemImage: "exclamationmark.triangle.fill")
                    .font(.callout)
                    .foregroundStyle(.secondary)
                    .accessibilityIdentifier("new-diagram-validation")
            }

            HStack {
                Spacer()
                Button("Cancel") { workspace.cancelNewDiagram() }
                    .keyboardShortcut(.cancelAction)
                    .accessibilityIdentifier("new-diagram-cancel")
                Button("Create") {
                    guard let destination else { return }
                    _ = workspace.createDiagram(name: name, format: format, destination: destination)
                }
                .keyboardShortcut(.defaultAction)
                .disabled(validationMessage != nil)
                .accessibilityIdentifier("new-diagram-create")
            }
        }
        .padding(24)
        .frame(width: 460)
        .onAppear { destination = workspace.diagramDestinations.first?.url }
        .onChange(of: name) { _, _ in workspace.clearNewDiagramError() }
        .onChange(of: format) { _, _ in workspace.clearNewDiagramError() }
        .onChange(of: destination) { _, _ in workspace.clearNewDiagramError() }
        .onChange(of: workspace.diagramDestinations) { _, destinations in
            if !destinations.contains(where: { $0.url == destination }) { destination = destinations.first?.url }
        }
        .accessibilityIdentifier("new-diagram-sheet")
    }
}
