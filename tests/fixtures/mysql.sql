CREATE TABLE accounts (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    created_at      TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    name            VARCHAR(255) NOT NULL,
    is_active       BOOLEAN,
    score           DOUBLE
);
