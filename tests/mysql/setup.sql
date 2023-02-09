-- https://github.com/prisma/database-schema-examples/tree/master/postgres/basic-twitter#basic-twitter
CREATE TABLE tweet
(
    id         BIGINT PRIMARY KEY AUTO_INCREMENT,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    text       TEXT      NOT NULL,
    owner_id   BIGINT
);

CREATE TABLE tweet_reply
(
    id         BIGINT PRIMARY KEY AUTO_INCREMENT,
    tweet_id   BIGINT    NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    text       TEXT      NOT NULL,
    owner_id   BIGINT,
    CONSTRAINT tweet_id_fk FOREIGN KEY (tweet_id) REFERENCES tweet(id)
);

CREATE TABLE products (
    product_no INTEGER,
    name TEXT,
    price NUMERIC CHECK (price > 0)
);
