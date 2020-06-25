-- https://github.com/prisma/database-schema-examples/tree/master/postgres/basic-twitter#basic-twitter
CREATE TABLE tweet
(
    id       BIGINT  NOT NULL PRIMARY KEY,
    text     TEXT    NOT NULL,
    is_sent  BOOLEAN NOT NULL DEFAULT TRUE,
    owner_id BIGINT
);

insert into tweet(id, text, owner_id) values (1, '#sqlx is pretty cool!', 1);
