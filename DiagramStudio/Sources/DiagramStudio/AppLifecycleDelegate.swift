import AppKit

@MainActor
final class AppLifecycleDelegate: NSObject, NSApplicationDelegate {
    weak var workspace: WorkspaceStore?

    func applicationShouldTerminate(_ sender: NSApplication) -> NSApplication.TerminateReply {
        workspace?.confirmTermination() == false ? .terminateCancel : .terminateNow
    }
}
