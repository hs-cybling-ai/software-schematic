import Foundation
#if SWIFT_PACKAGE
import CSQLite
#else
import SQLite3
#endif

final class SQLiteContextStore {
    static let schemaVersion = 1
    private let url: URL
    private let limits: ContextLimits
    private var database: OpaquePointer?

    init(url: URL, diagramID: String, limits: ContextLimits = ContextLimits()) throws {
        self.url = url; self.limits = limits
        guard sqlite3_open_v2(url.path, &database, SQLITE_OPEN_CREATE | SQLITE_OPEN_READWRITE | SQLITE_OPEN_FULLMUTEX, nil) == SQLITE_OK else { throw ContextStoreError.sqlite(Self.message(database)) }
        do {
            try execute("PRAGMA foreign_keys=ON; PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL;")
            try execute(Self.schema)
            try verifyMetadata(diagramID: diagramID)
            try enforceSize()
        } catch { sqlite3_close(database); database = nil; throw error }
    }

    deinit { sqlite3_close(database) }

    func commit(revision: Int, diagramID: String, sections: [ContextSection]) throws {
        guard revision > 0, sections.count <= limits.maximumSections, sections.allSatisfy({ $0.captureRevision == revision && $0.diagramID == diagramID }) else { throw ContextStoreError.invalidRevision }
        let validated = try sections.map { try $0.validated(limits: limits) }
        try execute("BEGIN IMMEDIATE")
        do {
            try execute("INSERT INTO capture_revisions(revision,captured_at,is_complete) VALUES(\(revision),strftime('%Y-%m-%dT%H:%M:%fZ','now'),0)")
            for section in validated { try insert(section) }
            try execute("UPDATE capture_revisions SET is_complete=1 WHERE revision=\(revision)")
            try execute("COMMIT"); try enforceSize()
        } catch { try? execute("ROLLBACK"); throw error }
    }

    func sections(revision: Int) throws -> [ContextSection] {
        guard try scalarInt("SELECT is_complete FROM capture_revisions WHERE revision=\(revision)") == 1 else { throw ContextStoreError.invalidRevision }
        var statement: OpaquePointer?
        try prepare("SELECT diagram_id,context_id,node_id,node_type,node_label,section_id,heading_path_json,ordinal,markdown,content_hash,embedding_blob,embedding_provider,embedding_model,embedding_dimensions,captured_at,capture_revision,is_tombstone FROM context_sections WHERE capture_revision=? ORDER BY node_id,ordinal,section_id", into: &statement)
        defer { sqlite3_finalize(statement) }
        sqlite3_bind_int64(statement, 1, sqlite3_int64(revision))
        var result: [ContextSection] = []
        while sqlite3_step(statement) == SQLITE_ROW { result.append(try decode(statement)) }
        return result
    }

    func sections(nodeID: String, revision: Int) throws -> [ContextSection] { try sections(revision: revision).filter { $0.nodeID == nodeID && !$0.isTombstone } }
    func unreferencedRevisions(authoritative: Int) throws -> [Int] { try ints("SELECT revision FROM capture_revisions WHERE is_complete=1 AND revision>\(authoritative) ORDER BY revision") }

    func compact(keeping revisions: Set<Int>) throws {
        let keep = revisions.sorted().map(String.init).joined(separator: ",")
        let predicate = keep.isEmpty ? "1=1" : "revision NOT IN (\(keep))"
        try execute("BEGIN IMMEDIATE")
        do {
            try execute("DELETE FROM context_sections WHERE capture_revision IN (SELECT revision FROM capture_revisions WHERE \(predicate))")
            try execute("DELETE FROM capture_revisions WHERE \(predicate)")
            try execute("COMMIT"); try execute("PRAGMA wal_checkpoint(TRUNCATE); VACUUM")
        } catch { try? execute("ROLLBACK"); throw error }
    }

    private func verifyMetadata(diagramID: String) throws {
        let existing = try scalarText("SELECT value FROM metadata WHERE key='diagram_id'")
        if let existing, existing != diagramID { throw ContextStoreError.diagramMismatch }
        if existing == nil { try execute("INSERT INTO metadata(key,value) VALUES('schema_version','\(Self.schemaVersion)'),('diagram_id','\(escaped(diagramID))')") }
        guard try scalarText("SELECT value FROM metadata WHERE key='schema_version'") == String(Self.schemaVersion) else { throw ContextStoreError.schemaMismatch }
    }

    private func insert(_ value: ContextSection) throws {
        var statement: OpaquePointer?
        try prepare("INSERT INTO context_sections VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)", into: &statement)
        defer { sqlite3_finalize(statement) }
        let heading = String(data: try JSONEncoder().encode(value.headingPath), encoding: .utf8)!
        let texts = [value.diagramID,value.contextID,value.nodeID,value.nodeType,value.nodeLabel,value.sectionID,heading]
        for (offset, text) in texts.enumerated() { bind(text, offset + 1, statement) }
        sqlite3_bind_int64(statement,8,sqlite3_int64(value.ordinal)); bind(value.markdown,9,statement); bind(value.contentHash,10,statement)
        _ = value.embeddingData.withUnsafeBytes { sqlite3_bind_blob(statement,11,$0.baseAddress,Int32($0.count),SQLITE_TRANSIENT) }
        bind(value.embeddingProvider,12,statement); bind(value.embeddingModel,13,statement); sqlite3_bind_int64(statement,14,sqlite3_int64(value.embeddingDimensions))
        bind(ISO8601DateFormatter().string(from:value.capturedAt),15,statement); sqlite3_bind_int64(statement,16,sqlite3_int64(value.captureRevision)); sqlite3_bind_int(statement,17,value.isTombstone ? 1 : 0)
        guard sqlite3_step(statement) == SQLITE_DONE else { throw ContextStoreError.sqlite(Self.message(database)) }
    }

    private func decode(_ statement: OpaquePointer?) throws -> ContextSection {
        func text(_ index: Int32) -> String { String(cString: sqlite3_column_text(statement,index)) }
        let dimensions = Int(sqlite3_column_int64(statement,13)); let pointer = sqlite3_column_blob(statement,10)
        let data = pointer == nil ? Data() : Data(bytes:pointer!,count:Int(sqlite3_column_bytes(statement,10)))
        return ContextSection(diagramID:text(0),contextID:text(1),nodeID:text(2),nodeType:text(3),nodeLabel:text(4),sectionID:text(5),headingPath:try JSONDecoder().decode([String].self,from:Data(text(6).utf8)),ordinal:Int(sqlite3_column_int64(statement,7)),markdown:text(8),contentHash:text(9),embedding:try ContextSection.embedding(from:data,dimensions:dimensions),embeddingProvider:text(11),embeddingModel:text(12),capturedAt:ISO8601DateFormatter().date(from:text(14)) ?? .distantPast,captureRevision:Int(sqlite3_column_int64(statement,15)),isTombstone:sqlite3_column_int(statement,16) != 0)
    }

    private func enforceSize() throws { let size = ((try? FileManager.default.attributesOfItem(atPath:url.path)[.size]) as? NSNumber)?.intValue ?? 0; if size > limits.maximumDatabaseBytes { throw ContextStoreError.databaseTooLarge } }
    private func prepare(_ sql:String,into statement:inout OpaquePointer?) throws { guard sqlite3_prepare_v2(database,sql,-1,&statement,nil) == SQLITE_OK else { throw ContextStoreError.sqlite(Self.message(database)) } }
    private func execute(_ sql:String) throws { var error:UnsafeMutablePointer<CChar>?; guard sqlite3_exec(database,sql,nil,nil,&error) == SQLITE_OK else { let message=error.map { String(cString:$0) } ?? Self.message(database); sqlite3_free(error); throw ContextStoreError.sqlite(message) } }
    private func scalarText(_ sql:String) throws -> String? { var s:OpaquePointer?; try prepare(sql,into:&s); defer{sqlite3_finalize(s)}; return sqlite3_step(s) == SQLITE_ROW ? String(cString:sqlite3_column_text(s,0)) : nil }
    private func scalarInt(_ sql:String) throws -> Int? { var s:OpaquePointer?; try prepare(sql,into:&s); defer{sqlite3_finalize(s)}; return sqlite3_step(s) == SQLITE_ROW ? Int(sqlite3_column_int64(s,0)) : nil }
    private func ints(_ sql:String) throws -> [Int] { var s:OpaquePointer?; try prepare(sql,into:&s); defer{sqlite3_finalize(s)}; var out:[Int]=[]; while sqlite3_step(s) == SQLITE_ROW { out.append(Int(sqlite3_column_int64(s,0))) }; return out }
    private func bind(_ value:String,_ index:Int,_ statement:OpaquePointer?) { sqlite3_bind_text(statement,Int32(index),value,-1,SQLITE_TRANSIENT) }
    private func escaped(_ value:String)->String { value.replacingOccurrences(of:"'",with:"''") }
    private static func message(_ database:OpaquePointer?)->String { database.map{String(cString:sqlite3_errmsg($0))} ?? "Unable to open database" }
    private static let schema = """
    CREATE TABLE IF NOT EXISTS metadata(key TEXT PRIMARY KEY,value TEXT NOT NULL);
    CREATE TABLE IF NOT EXISTS capture_revisions(revision INTEGER PRIMARY KEY,captured_at TEXT NOT NULL,is_complete INTEGER NOT NULL CHECK(is_complete IN (0,1)));
    CREATE TABLE IF NOT EXISTS context_sections(diagram_id TEXT NOT NULL,context_id TEXT NOT NULL,node_id TEXT NOT NULL,node_type TEXT NOT NULL,node_label TEXT NOT NULL,section_id TEXT NOT NULL,heading_path_json TEXT NOT NULL,ordinal INTEGER NOT NULL CHECK(ordinal>=0),markdown TEXT NOT NULL,content_hash TEXT NOT NULL CHECK(length(content_hash)=64),embedding_blob BLOB NOT NULL,embedding_provider TEXT NOT NULL,embedding_model TEXT NOT NULL,embedding_dimensions INTEGER NOT NULL CHECK(embedding_dimensions>=0),captured_at TEXT NOT NULL,capture_revision INTEGER NOT NULL REFERENCES capture_revisions(revision) ON DELETE CASCADE,is_tombstone INTEGER NOT NULL CHECK(is_tombstone IN (0,1)),PRIMARY KEY(capture_revision,section_id));
    CREATE INDEX IF NOT EXISTS context_sections_node_revision ON context_sections(node_id,capture_revision,ordinal);
    """
}

private let SQLITE_TRANSIENT = unsafeBitCast(-1, to: sqlite3_destructor_type.self)
