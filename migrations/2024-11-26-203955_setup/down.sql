-- This file should undo anything in `up.sql`
CREATE TABLE "addressallocationstates"(
	"id" VARCHAR(16) NOT NULL PRIMARY KEY,
	"name" VARCHAR(255) NOT NULL
);

CREATE TABLE "addresses"(
	"id" INET NOT NULL PRIMARY KEY,
	"allocation_state_id" VARCHAR(16) NOT NULL,
	"allocation_state_comment" VARCHAR(255),
	"routed" BOOL NOT NULL,
	"online" BOOL NOT NULL,
	"top_rir_id" VARCHAR(16) NOT NULL,
	"rir_id" VARCHAR(16) NOT NULL,
	"asn_id" INT4,
	"ports" INT4[] DEFAULT '{}' NOT NULL,
	"country" VARCHAR(3),
	"updated_at" TIMESTAMPTZ NOT NULL,
	FOREIGN KEY ("allocation_state_id") REFERENCES "AddressAllocationStates"("id"),
	FOREIGN KEY ("top_rir_id") REFERENCES "Rirs"("id"),
	FOREIGN KEY ("rir_id") REFERENCES "Rirs"("id"),
	FOREIGN KEY ("asn_id") REFERENCES "Asns"("id")
);

CREATE TABLE "asns"(
	"id" INT4 NOT NULL PRIMARY KEY
);

CREATE TABLE "rirs"(
	"id" VARCHAR(16) NOT NULL PRIMARY KEY,
	"name" VARCHAR(255) NOT NULL
);

CREATE TABLE "serviceunits"(
	"id" VARCHAR(36) NOT NULL PRIMARY KEY,
	"service_id" INT4 NOT NULL,
	"address" VARCHAR(255),
	"port" INT4,
	"created_at" TIMESTAMPTZ NOT NULL
);

CREATE TABLE "services"(
	"id" INT4 NOT NULL PRIMARY KEY,
	"name" VARCHAR(255) NOT NULL,
	"password" VARCHAR(255) NOT NULL,
	"max_one" BOOL NOT NULL
);

