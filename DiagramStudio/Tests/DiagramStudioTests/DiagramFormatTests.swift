import XCTest
@testable import DiagramStudio

final class DiagramFormatTests: XCTestCase {
    func testDetectsSupportedExtensionsCaseInsensitively() {
        XCTAssertEqual(DiagramFormat.detect(url: URL(fileURLWithPath: "/tmp/process.BPMN")), .bpmn)
        XCTAssertEqual(DiagramFormat.detect(url: URL(fileURLWithPath: "/tmp/process.bpmn20.xml")), .bpmn)
        XCTAssertEqual(DiagramFormat.detect(url: URL(fileURLWithPath: "/tmp/schema.DGRAPH")), .dataGraph)
        XCTAssertNil(DiagramFormat.detect(url: URL(fileURLWithPath: "/tmp/schema.dgraph.json")))
        XCTAssertNil(DiagramFormat.detect(url: URL(fileURLWithPath: "/tmp/notes.txt")))
    }

    func testEveryFormatProvidesCanonicalExtensionAndDetectableStarterXML() {
        for format in DiagramFormat.allCases {
            XCTAssertFalse(format.canonicalExtension.isEmpty)
            XCTAssertFalse(format.starterXML.isEmpty)
            XCTAssertEqual(DiagramFormat.detect(url: URL(fileURLWithPath: "new.\(format.canonicalExtension)")), format)
        }
        XCTAssertTrue(DiagramFormat.bpmn.starterXML.contains("BPMNPlane"))
        XCTAssertTrue(DiagramFormat.dataGraph.starterXML.contains("\"diagram-studio/data-graph\""))
    }

    func testStarterXMLMatchesSharedCompatibilityFixtures() throws {
        let repository = URL(fileURLWithPath: #filePath).deletingLastPathComponent().deletingLastPathComponent().deletingLastPathComponent().deletingLastPathComponent()
        let fixtures: [(DiagramFormat, String)] = [(.bpmn, "fixtures/bpmn/starter.bpmn"), (.dataGraph, "fixtures/data-graph/starter.dgraph")]
        for (format, path) in fixtures {
            let fixture = try String(contentsOf: repository.appendingPathComponent(path), encoding: .utf8)
                .trimmingCharacters(in: .newlines)
            XCTAssertEqual(format.starterXML, fixture)
        }
    }
}
