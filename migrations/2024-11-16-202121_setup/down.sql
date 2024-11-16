-- This file should undo anything in `up.sql`
CREATE TABLE "addresstypes"(
	"id" INT4 NOT NULL PRIMARY KEY,
	"address_id" INT4 NOT NULL,
	"type_id" INT4 NOT NULL,
	FOREIGN KEY ("address_id") REFERENCES "Addresses"("id"),
	FOREIGN KEY ("type_id") REFERENCES "Types"("id")
);

CREATE TABLE "addresses"(
	"id" INT4 NOT NULL PRIMARY KEY,
	"assigned" BOOL NOT NULL,
	"routed" BOOL NOT NULL,
	"online" BOOL NOT NULL
);

CREATE TABLE "services"(
	"id" INT4 NOT NULL PRIMARY KEY,
	"name" VARCHAR(255) NOT NULL,
	"password" VARCHAR(255) NOT NULL
);

CREATE TABLE "types"(
	"id" INT4 NOT NULL PRIMARY KEY,
	"name" VARCHAR(255) NOT NULL,
	"description" VARCHAR(255) NOT NULL
);

