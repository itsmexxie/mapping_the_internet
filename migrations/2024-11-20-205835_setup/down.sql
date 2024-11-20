-- This file should undo anything in `up.sql`
CREATE TABLE "addressservers"(
	"id" INT4 NOT NULL PRIMARY KEY,
	"address_id" INET NOT NULL,
	"server_id" INT4 NOT NULL,
	"port" INT4 NOT NULL,
	FOREIGN KEY ("address_id") REFERENCES "Addresses"("id"),
	FOREIGN KEY ("server_id") REFERENCES "Servers"("id")
);

CREATE TABLE "addresses"(
	"id" INET NOT NULL PRIMARY KEY,
	"state_id" INT4 NOT NULL,
	"routed" BOOL NOT NULL,
	"online" BOOL NOT NULL,
	"rir_id" INT4 NOT NULL,
	"asn_id" INT4 NOT NULL,
	FOREIGN KEY ("rir_id") REFERENCES "Rirs"("id"),
	FOREIGN KEY ("asn_id") REFERENCES "Asns"("id")
);

CREATE TABLE "asns"(
	"id" INT4 NOT NULL PRIMARY KEY
);

CREATE TABLE "rirs"(
	"id" INT4 NOT NULL PRIMARY KEY,
	"name" VARCHAR(255) NOT NULL
);

CREATE TABLE "servers"(
	"id" INT4 NOT NULL PRIMARY KEY,
	"name" VARCHAR(255) NOT NULL,
	"description" VARCHAR(255) NOT NULL
);

CREATE TABLE "serviceunits"(
	"id" VARCHAR(36) NOT NULL PRIMARY KEY,
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

DROP TABLE IF EXISTS "addresstypes";
DROP TABLE IF EXISTS "serviceunits";
DROP TABLE IF EXISTS "services";
DROP TABLE IF EXISTS "addresses";
DROP TABLE IF EXISTS "types";
