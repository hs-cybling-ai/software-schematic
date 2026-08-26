import WebKit
import XCTest
@testable import DiagramStudio

@MainActor
final class WebNavigationPolicyTests: XCTestCase {
    func testWebViewUsesEphemeralDataStoreAndSingleBridgeHandler() {
        let controller = EditorWebViewController()
        let webView = controller.makeWebView()
        XCTAssertNotNil(webView.configuration.websiteDataStore)
        XCTAssertTrue(EditorWebViewController.isAllowedNavigationURL(URL(fileURLWithPath: "/tmp/index.html"), targetIsMain: true))
        XCTAssertFalse(EditorWebViewController.isAllowedNavigationURL(URL(string: "https://example.com"), targetIsMain: true))
        XCTAssertFalse(EditorWebViewController.isAllowedNavigationURL(URL(fileURLWithPath: "/tmp/popup.html"), targetIsMain: false))
    }
}
