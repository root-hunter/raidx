CREATE TABLE "nodes" (
	"uid"	TEXT NOT NULL UNIQUE,

	"host"	TEXT NOT NULL,
	"port"	INTEGER NOT NULL CHECK("port" >= 0 AND "port" <= 65535),
	"local" BOOLEAN NOT NULL DEFAULT(false),

	PRIMARY KEY("uid"),
	CONSTRAINT unique_host_port UNIQUE ("host", "port")
);

CREATE TRIGGER check_local_node_one_insert
BEFORE INSERT ON "nodes"
FOR EACH ROW
WHEN NEW.local = true AND (SELECT COUNT(*) FROM "nodes" WHERE "local" = true) > 0
BEGIN
    SELECT RAISE(ABORT, 'Only one node can be local!');
END;

CREATE TABLE "files" (
	"id"	INTEGER NOT NULL,
	"uid"	TEXT NOT NULL UNIQUE,
	"node"  TEXT NOT NULL,

	"folder"	TEXT NOT NULL,
	"filename"	TEXT NOT NULL,
	"size"	INTEGER NOT NULL,
    
	"status" TEXT NOT NULL,
    "sync" BOOLEAN NOT NULL DEFAULT(false),
	
	"created_at"	INTEGER NOT NULL,
	"modified_at"	INTEGER NOT NULL,

	"updated_at" INTEGER NOT NULL,
	
	PRIMARY KEY("id" AUTOINCREMENT),
	FOREIGN KEY("node") REFERENCES "nodes"("uid") ON UPDATE CASCADE ON DELETE CASCADE
);

