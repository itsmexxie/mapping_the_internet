CREATE TABLE IF NOT EXISTS "ServiceUnits"(
    "id" VARCHAR(36) NOT NULL PRIMARY KEY,
    "service_id" INT4 NOT NULL,
    "address" VARCHAR(16) NULL,
    "port" INT4 NULL,
    FOREIGN KEY ("service_id") REFERENCES "Services"("id")
);