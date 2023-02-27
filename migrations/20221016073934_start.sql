CREATE TABLE IF NOT EXISTS "WelcomeMessage" (
	"WelcomeMessageId"	INTEGER,
	"GuildId"	INTEGER,
	"MessageId"	INTEGER,
	"ChannelId"	INTEGER,
	FOREIGN KEY("MessageId") REFERENCES "Message"("MessageId"),
	PRIMARY KEY("WelcomeMessageId")
);
CREATE TABLE IF NOT EXISTS "Embed" (
	"EmbedId"	INTEGER NOT NULL,
	"EmbedJson"	TEXT,
	PRIMARY KEY("EmbedId")
);
CREATE TABLE IF NOT EXISTS "Message" (
	"MessageId"	INTEGER,
	"Content"	TEXT NOT NULL,
	"EmbedId"	INTEGER,
	"MessageJson"	TEXT,
	FOREIGN KEY("EmbedId") REFERENCES "Embed"("EmbedId"),
	PRIMARY KEY("MessageId")
);
CREATE TABLE IF NOT EXISTS "AutoRoleMessage" (
	"AutoRoleMessageId"	INTEGER,
	PRIMARY KEY("AutoRoleMessageId")
);
CREATE TABLE IF NOT EXISTS "MutuallyExclusiveRole" (
	"GuildId"	INTEGER,
	"role1"	INTEGER,
	"role2"	INTEGER,
	PRIMARY KEY("GuildId","role1","role2")
);