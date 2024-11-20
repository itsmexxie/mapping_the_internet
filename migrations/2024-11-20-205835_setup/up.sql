-- Your SQL goes here
DROP TABLE IF EXISTS "addressservers";
DROP TABLE IF EXISTS "addresses";
DROP TABLE IF EXISTS "asns";
DROP TABLE IF EXISTS "rirs";
DROP TABLE IF EXISTS "servers";
DROP TABLE IF EXISTS "serviceunits";
DROP TABLE IF EXISTS "services";
CREATE TABLE "addresstypes"(
	"id" INT4 NOT NULL PRIMARY KEY,
	"address_id" INT4 NOT NULL,
	"type_id" INT4 NOT NULL,
	FOREIGN KEY ("address_id") REFERENCES "Addresses"("id"),
	FOREIGN KEY ("type_id") REFERENCES "Types"("id")
);

CREATE TABLE "serviceunits"(
	"id" VARCHAR(16) NOT NULL PRIMARY KEY,
	"service_id" INT4 NOT NULL,
	"address" VARCHAR(16),
	"port" INT4,
	FOREIGN KEY ("service_id") REFERENCES "Services"("id")
);

CREATE TABLE "services"(
	"id" INT4 NOT NULL PRIMARY KEY,
	"name" VARCHAR(255) NOT NULL,
	"password" VARCHAR(255) NOT NULL
);

CREATE TABLE "addresses"(
	"id" INT4 NOT NULL PRIMARY KEY,
	"assigned" BOOL NOT NULL,
	"routed" BOOL NOT NULL,
	"online" BOOL NOT NULL,
	"reserved" BOOL NOT NULL,
	"description" VARCHAR(255)
);

CREATE TABLE "types"(
	"id" INT4 NOT NULL PRIMARY KEY,
	"name" VARCHAR(255) NOT NULL,
	"description" VARCHAR(255) NOT NULL
);

