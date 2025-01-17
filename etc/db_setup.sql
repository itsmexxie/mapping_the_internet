-- Adminer 4.8.1 PostgreSQL 17.2 (Debian 17.2-1.pgdg120+1) dump

DROP TABLE IF EXISTS "AddressAllocationStates";
CREATE TABLE "public"."AddressAllocationStates" (
    "id" character varying(16) NOT NULL,
    "name" character varying(255) NOT NULL,
    CONSTRAINT "AddressAllocationStates_pkey" PRIMARY KEY ("id")
) WITH (oids = false);


DROP TABLE IF EXISTS "AddressMaps";
CREATE TABLE "public"."AddressMaps" (
    "id" cidr NOT NULL,
    "allocation_state_id" character varying(16) NOT NULL,
    "routed" boolean NOT NULL,
    "online" boolean NOT NULL,
    "updated_at" timestamptz DEFAULT now() NOT NULL,
    CONSTRAINT "AddressMap_id" PRIMARY KEY ("id")
) WITH (oids = false);


DROP TABLE IF EXISTS "Addresses";
CREATE TABLE "public"."Addresses" (
    "id" inet NOT NULL,
    "allocation_state_id" character varying(16) NOT NULL,
    "allocation_state_comment" character varying(255),
    "routed" boolean DEFAULT false NOT NULL,
    "online" boolean DEFAULT false NOT NULL,
    "top_rir_id" character varying(16),
    "rir_id" character varying(16),
    "autsys_id" bigint,
    "country" character varying(3),
    "updated_at" timestamptz DEFAULT now() NOT NULL,
    CONSTRAINT "Addresses_id" PRIMARY KEY ("id")
) WITH (oids = false);


DROP TABLE IF EXISTS "Autsyses";
CREATE TABLE "public"."Autsyses" (
    "id" bigint NOT NULL,
    CONSTRAINT "Asns_id" PRIMARY KEY ("id")
) WITH (oids = false);


DROP TABLE IF EXISTS "Rirs";
CREATE TABLE "public"."Rirs" (
    "id" character varying(16) NOT NULL,
    "name" character varying(255) NOT NULL,
    CONSTRAINT "Rirs_pkey" PRIMARY KEY ("id")
) WITH (oids = false);


DROP TABLE IF EXISTS "ServiceUnits";
CREATE TABLE "public"."ServiceUnits" (
    "id" uuid NOT NULL,
    "service_id" character varying(16) NOT NULL,
    "address" character varying(255) NOT NULL,
    "port" integer,
    "created_at" timestamptz DEFAULT now() NOT NULL,
    CONSTRAINT "ServiceUnits_id" PRIMARY KEY ("id")
) WITH (oids = false);


DROP TABLE IF EXISTS "Services";
CREATE TABLE "public"."Services" (
    "id" character varying(16) NOT NULL,
    "password" character varying(255) NOT NULL,
    CONSTRAINT "Services_id" PRIMARY KEY ("id")
) WITH (oids = false);


ALTER TABLE ONLY "public"."Addresses" ADD CONSTRAINT "Addresses_allocation_state_id_fkey" FOREIGN KEY (allocation_state_id) REFERENCES "AddressAllocationStates"(id) NOT DEFERRABLE;
ALTER TABLE ONLY "public"."Addresses" ADD CONSTRAINT "Addresses_asn_id_fkey" FOREIGN KEY (autsys_id) REFERENCES "Autsyses"(id) NOT DEFERRABLE;
ALTER TABLE ONLY "public"."Addresses" ADD CONSTRAINT "Addresses_rir_id_fkey" FOREIGN KEY (rir_id) REFERENCES "Rirs"(id) NOT DEFERRABLE;
ALTER TABLE ONLY "public"."Addresses" ADD CONSTRAINT "Addresses_top_rir_id_fkey" FOREIGN KEY (top_rir_id) REFERENCES "Rirs"(id) NOT DEFERRABLE;

-- 2025-01-17 21:39:27.487874+01
