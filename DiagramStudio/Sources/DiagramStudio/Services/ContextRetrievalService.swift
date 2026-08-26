import Foundation

struct ContextQuery: Sendable {
    var text = ""
    var nodeID: String?
    var contextID: String?
    var sectionID: String?
    var queryEmbedding: [Float]?
    var embeddingProvider: String?
    var embeddingModel: String?
    var maximumHops = 1
    var limit = 10
}

struct ContextScore: Codable, Equatable, Sendable { let lexical: Double, semantic: Double, graph: Double, total: Double }
struct ContextSearchResult: Codable, Equatable, Sendable {
    let nodeID, contextID, sectionID, nodeLabel, markdown: String
    let headingPath: [String]
    let revision, graphDistance: Int
    let embeddingProvider, embeddingModel: String
    let score: ContextScore
}
struct ContextSearchResponse: Codable, Equatable, Sendable { let semanticCoverage: Bool; let results: [ContextSearchResult] }

struct ContextRetrievalService {
    func search(query: ContextQuery, diagram: [String: Any], rows: [ContextSection]) -> ContextSearchResponse {
        let distances = graphDistances(from:query.nodeID,diagram:diagram,maximumHops:max(0,query.maximumHops))
        let tokens = Set(query.text.lowercased().split(whereSeparator:{ !$0.isLetter && !$0.isNumber }).map(String.init))
        var semanticCoverage=false
        let matches = rows.filter { !$0.isTombstone }.compactMap { row -> ContextSearchResult? in
            if let id=query.nodeID, row.nodeID != id, distances[row.nodeID] == nil { return nil }
            if let id=query.contextID, row.contextID != id { return nil }
            if let id=query.sectionID, row.sectionID != id { return nil }
            let haystack=(row.nodeLabel+" "+row.headingPath.joined(separator:" ")+" "+row.markdown).lowercased()
            let lexical=tokens.isEmpty ? 0 : Double(tokens.filter{haystack.contains($0)}.count)/Double(tokens.count)
            var semantic=0.0
            if let vector=query.queryEmbedding, query.embeddingProvider == row.embeddingProvider, query.embeddingModel == row.embeddingModel, vector.count == row.embedding.count {
                semanticCoverage=true; semantic=cosine(vector,row.embedding)
            }
            let distance=distances[row.nodeID] ?? (query.nodeID == nil ? 0 : Int.max)
            let graph = distance == Int.max ? 0 : 1.0/Double(distance+1)
            let exact = [query.nodeID == row.nodeID,query.contextID == row.contextID,query.sectionID == row.sectionID].contains(true) ? 1.0 : 0
            let total=exact+lexical*0.35+semantic*0.55+graph*0.10
            if total == 0 && !tokens.isEmpty { return nil }
            return ContextSearchResult(nodeID:row.nodeID,contextID:row.contextID,sectionID:row.sectionID,nodeLabel:row.nodeLabel,markdown:row.markdown,headingPath:row.headingPath,revision:row.captureRevision,graphDistance:distance == Int.max ? -1:distance,embeddingProvider:row.embeddingProvider,embeddingModel:row.embeddingModel,score:ContextScore(lexical:lexical,semantic:semantic,graph:graph,total:total))
        }.sorted { $0.score.total == $1.score.total ? ($0.nodeID,$0.sectionID) < ($1.nodeID,$1.sectionID) : $0.score.total > $1.score.total }
        return ContextSearchResponse(semanticCoverage:semanticCoverage,results:Array(matches.prefix(max(0,query.limit))))
    }

    private func cosine(_ lhs:[Float],_ rhs:[Float])->Double { let dot=zip(lhs,rhs).reduce(0.0){$0+Double($1.0*$1.1)}; let a=sqrt(lhs.reduce(0.0){$0+Double($1*$1)}); let b=sqrt(rhs.reduce(0.0){$0+Double($1*$1)}); return a == 0 || b == 0 ? 0 : dot/(a*b) }
    private func graphDistances(from start:String?,diagram:[String:Any],maximumHops:Int)->[String:Int] {
        guard let start else { return [:] }; let edges=diagram["edges"] as? [[String:Any]] ?? []; var adjacency:[String:Set<String>]=[:]
        for edge in edges { guard let source=edge["source"] as? String,let target=edge["target"] as? String else{continue}; adjacency[source,default:[]].insert(target); adjacency[target,default:[]].insert(source) }
        var result=[start:0],queue=[start]; while !queue.isEmpty { let node=queue.removeFirst(),distance=result[node]!; if distance >= maximumHops{continue}; for next in (adjacency[node] ?? []).sorted() where result[next] == nil { result[next]=distance+1; queue.append(next) } }; return result
    }
}
