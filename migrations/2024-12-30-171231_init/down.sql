-- This file should undo anything in `up.sql`
DROP TABLE IF EXISTS "Addresses";
DROP TABLE IF EXISTS "AddressAllocationStates";
DROP TABLE IF EXISTS "Autsyses";
DROP TABLE IF EXISTS "Rirs";
DROP TABLE IF EXISTS "ServiceUnits";
DROP TABLE IF EXISTS "Services";
DROP SEQUENCE IF EXISTS "Services_id_seq";
