-- Your SQL goes here
CREATE TABLE IF NOT EXISTS "AddressMaps" (
    "id" cidr NOT NULL,
    "allocation_state_id" character varying(16) NOT NULL,
    "updated_at" timestamptz DEFAULT now() NOT NULL,
    CONSTRAINT "AddressMap_id" PRIMARY KEY ("id")
);
