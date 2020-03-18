BEGIN TRANSACTION;
DROP TABLE IF EXISTS "chat";
CREATE TABLE IF NOT EXISTS "chat" (
	"message_id"	INTEGER NOT NULL UNIQUE,
	"message"	TEXT,
	"send_date"	INTEGER NOT NULL,
	"sender"	TEXT NOT NULL,
	"to"	TEXT,
	PRIMARY KEY("message_id")
);
DROP TABLE IF EXISTS "users";
CREATE TABLE IF NOT EXISTS "users" (
    "user_id"	INTEGER NOT NULL UNIQUE,
	"user_name"	TEXT NOT NULL,
	"session_id"	TEXT,
	"role"	TEXT,
	"joined"	INTEGER,
	"state"	TEXT,
	PRIMARY KEY("user_name")
);
DROP TABLE IF EXISTS "sessions";
CREATE TABLE IF NOT EXISTS "sessions" (
	"id"	TEXT NOT NULL UNIQUE,
	"created"	INTEGER NOT NULL,
	"active"	INTEGER NOT NULL,
	"settings"	TEXT,
	PRIMARY KEY("id")
);
COMMIT;
