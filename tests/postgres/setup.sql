-- https://www.postgresql.org/docs/current/sql-createtype.html
CREATE TYPE status AS ENUM ('new', 'open', 'closed');

-- https://www.postgresql.org/docs/current/rowtypes.html#ROWTYPES-DECLARING
CREATE TYPE inventory_item AS
(
    name        TEXT,
    supplier_id INT,
    price       BIGINT
);

-- https://github.com/prisma/database-schema-examples/tree/master/postgres/basic-twitter#basic-twitter
CREATE TABLE tweet
(
    id         BIGSERIAL PRIMARY KEY,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    text       TEXT        NOT NULL,
    owner_id   BIGINT
);

CREATE TYPE float_range AS RANGE
(
    subtype = float8,
    subtype_diff = float8mi
);

CREATE TABLE products (
    product_no INTEGER,
    name TEXT,
    price NUMERIC CHECK (price > 0)
);
