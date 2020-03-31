CREATE TABLE IF NOT EXISTS users (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,

    created_at INTEGER NOT NULL DEFAULT (STRFTIME('%s', 'now')),
    updated_at INTEGER,

    email TEXT UNIQUE NOT NULL,
    username TEXT UNIQUE NOT NULL,

    password TEXT
);
