import Foundation
import XCTest
@testable import DiagramStudio

final class BridgeProtocolTests: XCTestCase {
    func testEnvelopeRoundTripPreservesXMLAsData() throws {
        let xml = "</script><script>notExecutable()</script>"
        let message = BridgeEnvelope(version: 1, type: "load", requestId: "r1", format: "bpmn", payload: LoadPayload(xml: xml))
        let data = try JSONEncoder().encode(message)
        XCTAssertEqual(try JSONDecoder().decode(BridgeEnvelope<LoadPayload>.self, from: data).payload.xml, xml)
    }

    func testEnvelopeRoundTripPreservesDataGraphJSONAsData() throws {
        let json = #"{"format":"diagram-studio/data-graph","version":2,"nodes":[]}"#
        let message = BridgeEnvelope(version: 1, type: "load", requestId: "r2", format: "dataGraph", payload: LoadPayload(xml: json))
        let data = try JSONEncoder().encode(message)
        XCTAssertEqual(try JSONDecoder().decode(BridgeEnvelope<LoadPayload>.self, from: data).payload.xml, json)
    }

    func testCapturePayloadRoundTripPreservesMarkdownAndSections() throws {
        let section = ContextDraftSectionPayload(sectionId:"s",markdown:"# Safe\n",contentHash:String(repeating:"a",count:64),headingPath:["Safe"],ordinal:0)
        let payload = ContextCapturePayload(diagram:"{}",drafts:[ContextDraftPayload(contextId:"c",nodeId:"n",nodeType:"objectNode",nodeLabel:"N",markdown:"# Safe\n",sections:[section])],tombstones:["deleted"])
        let data = try JSONEncoder().encode(BridgeEnvelope(version:1,type:"captureRequested",requestId:"c1",format:"dataGraph",payload:payload))
        let decoded = try JSONDecoder().decode(BridgeEnvelope<ContextCapturePayload>.self,from:data)
        XCTAssertEqual(decoded.payload.drafts.first?.sections.first?.markdown,"# Safe\n"); XCTAssertEqual(decoded.payload.tombstones,["deleted"])
    }
}
