import XCTest

final class DiagramStudioUITests: XCTestCase {
    private var workspaceURL: URL!

    override func setUpWithError() throws {
        continueAfterFailure = false
        workspaceURL = FileManager.default.homeDirectoryForCurrentUser
            .appendingPathComponent("Library/Containers/com.harlanshober.DiagramStudio/Data/Documents", isDirectory: true)
            .appendingPathComponent("UITest-\(UUID().uuidString)", isDirectory: true)
        let nested = workspaceURL.appendingPathComponent("Architecture", isDirectory: true)
        try FileManager.default.createDirectory(at: nested, withIntermediateDirectories: true)
        try "<bpmn/>".write(to: workspaceURL.appendingPathComponent("existing.bpmn"), atomically: true, encoding: .utf8)
    }

    override func tearDownWithError() throws { try? FileManager.default.removeItem(at: workspaceURL) }

    @MainActor
    private func launchWithWorkspace() -> XCUIApplication {
        let app = XCUIApplication()
        app.launchArguments += ["-UITestWorkspacePath", workspaceURL.path]
        app.launch()
        let newDiagram = app.menuItems["New Diagram…"]
        XCTAssertTrue(newDiagram.waitForExistence(timeout: 5))
        expectation(for: NSPredicate(format: "enabled == true"), evaluatedWith: newDiagram)
        waitForExpectations(timeout: 5)
        return app
    }

    @MainActor
    private func openNewDiagramSheet(in app: XCUIApplication) {
        app.menuItems["New Diagram…"].click()
        XCTAssertTrue(app.staticTexts["New Diagram"].waitForExistence(timeout: 3))
    }

    @MainActor
    func testLaunchShowsEmptyWorkspace() {
        let app = XCUIApplication()
        app.launchArguments += ["-workspaceBookmark", ""]
        app.launch()
        XCTAssertTrue(app.staticTexts["No Folder Open"].waitForExistence(timeout: 5))
        XCTAssertTrue(app.menuItems["Open Folder…"].exists)
        XCTAssertFalse(app.menuItems["New Diagram…"].isEnabled)
        XCTAssertTrue(app.splitGroups.firstMatch.exists)
    }

    @MainActor
    func testConfigureBPMNInWorkspaceRoot() {
        let app = launchWithWorkspace()
        XCTAssertTrue(app.menuItems["New Diagram…"].isEnabled)
        openNewDiagramSheet(in: app)
        let name = app.textFields["new-diagram-name"]
        name.click()
        name.typeText("Customer Flow")
        XCTAssertTrue(app.buttons["Create"].isEnabled)
        app.buttons["Cancel"].click()
    }

    @MainActor
    func testValidationCollisionRecoveryAndCancel() {
        let app = launchWithWorkspace()
        openNewDiagramSheet(in: app)
        let name = app.textFields["new-diagram-name"]
        XCTAssertFalse(app.buttons["Create"].isEnabled)
        name.click()
        name.typeText("existing.bpmn")
        XCTAssertFalse(app.buttons["Create"].isEnabled)
        name.typeKey("a", modifierFlags: .command)
        name.typeText("Recovered")
        XCTAssertTrue(app.buttons["Create"].isEnabled)
        app.buttons["Cancel"].click()
        XCTAssertFalse(app.staticTexts["New Diagram"].exists)
        XCTAssertFalse(FileManager.default.fileExists(atPath: workspaceURL.appendingPathComponent("Recovered.bpmn").path))
    }
}
