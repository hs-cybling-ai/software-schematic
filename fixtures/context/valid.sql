PRAGMA foreign_keys=ON;
CREATE TABLE metadata(key TEXT PRIMARY KEY,value TEXT NOT NULL);
CREATE TABLE capture_revisions(revision INTEGER PRIMARY KEY,captured_at TEXT NOT NULL,is_complete INTEGER NOT NULL);
CREATE TABLE context_sections(diagram_id TEXT,context_id TEXT,node_id TEXT,node_type TEXT,node_label TEXT,section_id TEXT,heading_path_json TEXT,ordinal INTEGER,markdown TEXT,content_hash TEXT,embedding_blob BLOB,embedding_provider TEXT,embedding_model TEXT,embedding_dimensions INTEGER,captured_at TEXT,capture_revision INTEGER,is_tombstone INTEGER);
INSERT INTO metadata VALUES('schema_version','1'),('diagram_id','fixture-diagram');
INSERT INTO capture_revisions VALUES(1,'2026-08-04T00:00:00Z',1);
INSERT INTO context_sections VALUES('fixture-diagram','context-person','person','objectNode','Person','person-overview','["Overview"]',0,'# Overview\nA person.','aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',X'CDCCCC3DCDCC4C3E9A99993E','fixture','fixture-3',3,'2026-08-04T00:00:00Z',1,0);
INSERT INTO context_sections VALUES('fixture-diagram','context-company','company','objectNode','Company','company-overview','["Overview"]',0,'# Overview\nA company.','bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb',X'9A99993ECDCC4C3ECDCCCC3D','fixture','fixture-3',3,'2026-08-04T00:00:00Z',1,0);
INSERT INTO context_sections VALUES('fixture-diagram','context-deleted','deleted','objectNode','Deleted','deleted-tombstone','[]',0,'','cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc',X'','','',0,'2026-08-04T00:00:00Z',1,1);
