CREATE TABLE IF NOT EXISTS todos (
    id          SERIAL PRIMARY KEY,
    description TEXT NOT NULL,
    done        BOOLEAN NOT NULL DEFAULT FALSE
);
