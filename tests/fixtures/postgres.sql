CREATE TABLE accounts (
    id              BIGSERIAL PRIMARY KEY,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    name            TEXT NOT NULL,
    is_active       BOOLEAN,
    score           DOUBLE PRECISION
);

-- https://www.postgresql.org/docs/current/rowtypes.html#ROWTYPES-DECLARING
CREATE TYPE inventory_item AS (
    name            TEXT,
    supplier_id     INT,
    price           BIGINT
);
