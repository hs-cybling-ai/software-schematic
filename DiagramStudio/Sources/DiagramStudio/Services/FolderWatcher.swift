import Foundation
import Darwin

@MainActor
final class FolderWatcher {
    private var source: DispatchSourceFileSystemObject?
    private var descriptor: Int32 = -1
    private var timer: Timer?

    func watch(_ url: URL, onChange: @escaping @MainActor () -> Void) {
        stop()
        descriptor = open(url.path, O_EVTONLY)
        guard descriptor >= 0 else { return }
        let source = DispatchSource.makeFileSystemObjectSource(fileDescriptor: descriptor, eventMask: [.write, .delete, .rename], queue: .main)
        source.setEventHandler {
            MainActor.assumeIsolated {
                onChange()
            }
        }
        source.setCancelHandler { [descriptor] in close(descriptor) }
        self.source = source
        source.resume()
        timer = Timer.scheduledTimer(withTimeInterval: 2, repeats: true) { _ in
            MainActor.assumeIsolated { onChange() }
        }
    }

    func stop() {
        source?.cancel()
        source = nil
        timer?.invalidate()
        timer = nil
        descriptor = -1
    }

    deinit {
        source?.cancel()
        MainActor.assumeIsolated { timer?.invalidate() }
    }
}
