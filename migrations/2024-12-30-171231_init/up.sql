-- Your SQL goes here
CREATE TABLE "AddressAllocationStates" (
    "id" character varying(16) NOT NULL,
    "name" character varying(255) NOT NULL,
    CONSTRAINT "AddressAllocationStates_pkey" PRIMARY KEY ("id")
);

CREATE TABLE "Addresses" (
    "id" inet NOT NULL,
    "allocation_state_id" character varying(16) NOT NULL,
    "allocation_state_comment" character varying(255),
    "routed" boolean DEFAULT false NOT NULL,
    "online" boolean DEFAULT false NOT NULL,
    "top_rir_id" character varying(16),
    "rir_id" character varying(16),
    "autsys_id" integer,
    "country" character varying(3),
    "updated_at" timestamptz DEFAULT now() NOT NULL,
    CONSTRAINT "Addresses_id" PRIMARY KEY ("id")
);

CREATE TABLE "Autsyses" (
    "id" integer NOT NULL,
    CONSTRAINT "Asns_id" PRIMARY KEY ("id")
);

CREATE TABLE "Rirs" (
    "id" character varying(16) NOT NULL,
    "name" character varying(255) NOT NULL,
    CONSTRAINT "Rirs_pkey" PRIMARY KEY ("id")
);

CREATE TABLE "ServiceUnits" (
    "id" character varying(36) NOT NULL,
    "service_id" integer NOT NULL,
    "address" character varying(255),
    "port" integer,
    "created_at" timestamptz DEFAULT now() NOT NULL,
    CONSTRAINT "ServiceUnits_id" PRIMARY KEY ("id")
);

CREATE SEQUENCE "Services_id_seq";

CREATE TABLE "Services" (
    "id" integer DEFAULT nextval('"Services_id_seq"') NOT NULL,
    "name" character varying(255) NOT NULL,
    "password" character varying(255) NOT NULL,
    "max_one" boolean DEFAULT false NOT NULL,
    CONSTRAINT "Services_pkey" PRIMARY KEY ("id")
);

ALTER TABLE ONLY "Addresses" ADD CONSTRAINT "Addresses_allocation_state_id_fkey" FOREIGN KEY (allocation_state_id) REFERENCES "AddressAllocationStates"(id) NOT DEFERRABLE;
ALTER TABLE ONLY "Addresses" ADD CONSTRAINT "Addresses_asn_id_fkey" FOREIGN KEY (autsys_id) REFERENCES "Autsyses"(id) NOT DEFERRABLE;
ALTER TABLE ONLY "Addresses" ADD CONSTRAINT "Addresses_rir_id_fkey" FOREIGN KEY (rir_id) REFERENCES "Rirs"(id) NOT DEFERRABLE;
ALTER TABLE ONLY "Addresses" ADD CONSTRAINT "Addresses_top_rir_id_fkey" FOREIGN KEY (top_rir_id) REFERENCES "Rirs"(id) NOT DEFERRABLE;
