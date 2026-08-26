import Foundation

@MainActor
final class SecurityScopedBookmarkStore {
    private let defaults: UserDefaults
    private let key = "workspaceBookmark"
    private var activeURL: URL?

    init(defaults: UserDefaults = .standard) { self.defaults = defaults }

    func persist(_ url: URL) throws {
        let data = try url.bookmarkData(options: [.withSecurityScope], includingResourceValuesForKeys: nil, relativeTo: nil)
        defaults.set(data, forKey: key)
    }

    func restore() throws -> (url: URL, stale: Bool)? {
        guard let data = defaults.data(forKey: key) else { return nil }
        var stale = false
        let url = try URL(resolvingBookmarkData: data, options: [.withSecurityScope], relativeTo: nil, bookmarkDataIsStale: &stale)
        return (url, stale)
    }

    @discardableResult
    func startAccessing(_ url: URL) -> Bool {
        stopAccessing()
        let granted = url.startAccessingSecurityScopedResource()
        if granted { activeURL = url }
        return granted
    }

    func stopAccessing() {
        activeURL?.stopAccessingSecurityScopedResource()
        activeURL = nil
    }

    deinit { activeURL?.stopAccessingSecurityScopedResource() }
}
