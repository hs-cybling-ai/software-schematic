import Foundation

enum DiagramFormat: String, Codable, CaseIterable, Sendable {
    case bpmn
    case dataGraph

    static let maximumFileSize = 20 * 1_024 * 1_024

    static func detect(url: URL) -> DiagramFormat? {
        let name = url.lastPathComponent.lowercased()
        if name.hasSuffix(".bpmn") || name.hasSuffix(".bpmn20.xml") { return .bpmn }
        if name.hasSuffix(".dgraph") { return .dataGraph }
        return nil
    }

    var displayName: String {
        switch self { case .bpmn: "BPMN"; case .dataGraph: "Data Graph" }
    }

    var canonicalExtension: String {
        switch self { case .bpmn: "bpmn"; case .dataGraph: "dgraph" }
    }

    var starterXML: String {
        switch self {
        case .bpmn:
            """
            <?xml version="1.0" encoding="UTF-8"?>
            <bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL" xmlns:bpmndi="http://www.omg.org/spec/BPMN/20100524/DI" id="Definitions_1" targetNamespace="http://diagramstudio.example/bpmn">
              <bpmn:process id="Process_1" isExecutable="false" />
              <bpmndi:BPMNDiagram id="BPMNDiagram_1">
                <bpmndi:BPMNPlane id="BPMNPlane_1" bpmnElement="Process_1" />
              </bpmndi:BPMNDiagram>
            </bpmn:definitions>
            """
        case .dataGraph:
            """
            {
              "format": "diagram-studio/data-graph",
              "version": 2,
              "nodes": [],
              "edges": [],
              "properties": [],
              "contextStore": {
                "kind": "sqlite",
                "uri": "diagram.context.sqlite",
                "schemaVersion": 1,
                "diagramId": "starter-diagram",
                "revision": 0
              }
            }
            """
        }
    }
}
