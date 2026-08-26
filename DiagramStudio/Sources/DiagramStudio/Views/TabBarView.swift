import SwiftUI

struct TabBarView: View {
    @EnvironmentObject private var workspace: WorkspaceStore

    var body: some View {
        ScrollView(.horizontal) {
            HStack(spacing: 1) {
                ForEach(workspace.tabs) { tab in
                    HStack(spacing: 6) {
                        Button {
                            workspace.selectedTabID = tab.id
                        } label: {
                            HStack(spacing: 5) {
                                if tab.isDirty { Circle().frame(width: 7, height: 7).foregroundStyle(.orange) }
                                Text(tab.title).lineLimit(1)
                            }
                        }
                        .buttonStyle(.plain)
                        .accessibilityIdentifier("tab-\(tab.title)")
                        Button { workspace.close(tab) } label: { Image(systemName: "xmark") }
                            .buttonStyle(.borderless)
                            .accessibilityLabel("Close \(tab.title)")
                    }
                    .padding(.horizontal, 10)
                    .frame(height: 34)
                    .background(workspace.selectedTabID == tab.id ? Color.accentColor.opacity(0.22) : Color.clear)
                }
            }
        }
        .background(.bar)
        .accessibilityIdentifier("editor-tabs")
    }
}
