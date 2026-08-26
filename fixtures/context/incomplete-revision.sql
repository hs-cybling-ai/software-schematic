CREATE TABLE metadata(key TEXT PRIMARY KEY,value TEXT NOT NULL);
CREATE TABLE capture_revisions(revision INTEGER PRIMARY KEY,captured_at TEXT NOT NULL,is_complete INTEGER NOT NULL);
INSERT INTO metadata VALUES('schema_version','1'),('diagram_id','fixture-diagram');
INSERT INTO capture_revisions VALUES(2,'2026-08-04T00:00:00Z',0);
