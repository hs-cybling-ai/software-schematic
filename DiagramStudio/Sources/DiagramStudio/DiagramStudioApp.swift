import SwiftUI

@main
struct DiagramStudioApp: App {
    @NSApplicationDelegateAdaptor(AppLifecycleDelegate.self) private var lifecycleDelegate
    @StateObject private var workspace = WorkspaceStore()

    var body: some Scene {
        WindowGroup("Diagram Studio") {
            WorkspaceView()
                .environmentObject(workspace)
                .frame(minWidth: 860, minHeight: 560)
                .task {
                    lifecycleDelegate.workspace = workspace
                    if let index = ProcessInfo.processInfo.arguments.firstIndex(of: "-UITestWorkspacePath"),
                       ProcessInfo.processInfo.arguments.indices.contains(index + 1) {
                        try? workspace.activate(URL(fileURLWithPath: ProcessInfo.processInfo.arguments[index + 1], isDirectory: true))
                    } else {
                        workspace.restoreWorkspace()
                    }
                }
        }
        .commands {
            CommandGroup(replacing: .newItem) {
                Button("New Diagram…") { workspace.beginNewDiagram() }
                    .keyboardShortcut("n")
                    .disabled(!workspace.canCreateDiagram)
                Button("Open Folder…") { workspace.chooseFolder() }.keyboardShortcut("o", modifiers: [.command, .shift])
            }
            CommandGroup(replacing: .saveItem) {
                Button("Save") { Task { _ = await workspace.saveSelected() } }.keyboardShortcut("s").disabled(!workspace.canSave)
                Button("Save All") { Task { _ = await workspace.saveAll() } }.keyboardShortcut("s", modifiers: [.command, .option]).disabled(!workspace.canSaveAll)
            }
            CommandGroup(after: .saveItem) {
                Button("Close Tab") { workspace.closeSelected() }.keyboardShortcut("w").disabled(workspace.selectedTab == nil)
            }
        }
    }
}
