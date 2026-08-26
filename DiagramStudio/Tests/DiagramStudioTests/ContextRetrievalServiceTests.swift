import XCTest
@testable import DiagramStudio

final class ContextRetrievalServiceTests:XCTestCase {
    private func row(_ node:String,_ section:String,_ text:String,_ vector:[Float],tombstone:Bool=false)->ContextSection { ContextSection(diagramID:"d",contextID:"c-\(node)",nodeID:node,nodeType:"objectNode",nodeLabel:node.capitalized,sectionID:section,headingPath:["Overview"],ordinal:0,markdown:text,contentHash:String(repeating:"a",count:64),embedding:vector,embeddingProvider:"fixture",embeddingModel:"m",capturedAt:.distantPast,captureRevision:1,isTombstone:tombstone) }
    func testExactSemanticLexicalGraphAndCyclesAreDeterministic(){
        let diagram:[String:Any]=["edges":[["source":"a","target":"edge"],["source":"edge","target":"b"],["source":"b","target":"a"]]]
        let rows=[row("a","s1","person account",[1,0]),row("b","s2","company billing",[0.9,0.1]),row("z","s3","unrelated",[0,1]),row("deleted","s4","person",[1,0],tombstone:true)]
        let query=ContextQuery(text:"person",nodeID:"a",queryEmbedding:[1,0],embeddingProvider:"fixture",embeddingModel:"m",maximumHops:2,limit:10)
        let result=ContextRetrievalService().search(query:query,diagram:diagram,rows:rows)
        XCTAssertTrue(result.semanticCoverage); XCTAssertEqual(result.results.map(\.nodeID),["a","b"]); XCTAssertEqual(result.results[1].graphDistance,1)
        XCTAssertEqual(result,ContextRetrievalService().search(query:query,diagram:diagram,rows:rows))
    }
    func testIncompatibleEmbeddingFallsBackToLexical(){ let result=ContextRetrievalService().search(query:ContextQuery(text:"billing",queryEmbedding:[1],embeddingProvider:"other",embeddingModel:"x"),diagram:[:],rows:[row("b","s","billing",[1,0])]); XCTAssertFalse(result.semanticCoverage); XCTAssertEqual(result.results.first?.nodeID,"b") }
}
