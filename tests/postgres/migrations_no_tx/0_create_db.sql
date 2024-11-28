-- no-transaction

CREATE DATABASE test_db;

CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(50) NOT NULL,
    email VARCHAR(100) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX CONCURRENTLY idx_users_email ON users(email);

INSERT INTO users (username, email) VALUES ('test_user', 'test_user@example.com');