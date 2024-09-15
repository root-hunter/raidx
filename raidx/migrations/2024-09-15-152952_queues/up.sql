-- Your SQL goes here
CREATE TABLE "messages_incoming" (
	"id"	INTEGER NOT NULL,
	"uid"	TEXT NOT NULL UNIQUE,
	
	"message_type" TEXT NOT NULL,
	"data" BLOB,
	"from" TEXT NOT NULL,

	"created_at"	INTEGER NOT NULL,
	
	PRIMARY KEY("id" AUTOINCREMENT),
	FOREIGN KEY("from") REFERENCES "nodes"("uid") ON UPDATE CASCADE ON DELETE CASCADE
);

CREATE TABLE "messages_outgoing" (
	"id"	INTEGER NOT NULL,
	"uid"	TEXT NOT NULL UNIQUE,

	"message_type" TEXT NOT NULL,
	"data" BLOB,
	"to" TEXT NOT NULL,

	"created_at"	INTEGER NOT NULL,
	
	PRIMARY KEY("id" AUTOINCREMENT),
	FOREIGN KEY("to") REFERENCES "nodes"("uid") ON UPDATE CASCADE ON DELETE CASCADE
);