-- seed some data to work with
WITH inserted_users_cte AS (
    INSERT INTO users (username)
        VALUES ('user1'),
               ('user2')
        RETURNING id as "user_id"
)
INSERT INTO posts (title, body, user_id)
VALUES ('user1 post1 title', 'user1 post1 body', (SELECT user_id FROM inserted_users_cte FETCH FIRST ROW ONLY)),
       ('user1 post2 title', 'user1 post2 body', (SELECT user_id FROM inserted_users_cte FETCH FIRST ROW ONLY)),
       ('user2 post1 title', 'user2 post2 body', (SELECT user_id FROM inserted_users_cte OFFSET 1 LIMIT 1));