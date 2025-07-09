-- additional SQL to execute for MariaDB databases

CREATE TABLE tweet_with_uuid
(
    -- UUID is only a bespoke datatype in MariaDB.
    id         UUID PRIMARY KEY   DEFAULT UUID(),
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    text       TEXT      NOT NULL,
    owner_id   UUID
);
