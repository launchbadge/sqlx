SELECT p.id as "post_id",
       p.title,
       p.body,
       u.id as "author_id",
       u.username as "author_username"
FROM users u
         JOIN posts p on u.id = p.user_id;