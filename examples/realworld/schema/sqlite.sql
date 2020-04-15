CREATE TABLE IF NOT EXISTS users (
    user_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,

    email TEXT UNIQUE NOT NULL,
    password TEXT NOT NULL,
    username TEXT UNIQUE NOT NULL,
    bio TEXT,
    image TEXT,

    created_at INTEGER NOT NULL DEFAULT (STRFTIME('%s', 'now')),
    updated_at INTEGER NOT NULL DEFAULT (STRFTIME('%s', 'now'))
);

CREATE VIEW profiles AS
SELECT user_id, username, bio, image
FROM users;


CREATE TABLE IF NOT EXISTS articles (
    article_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,

    title TEXT UNIQUE NOT NULL,
    description TEXT NOT NULL,
    slug TEXT UNIQUE NOT NULL,
    body TEXT NOT NULL,
    author_id INTEGER NOT NULL REFERENCES users (user_id) ON DELETE CASCADE ,

    created_at INTEGER NOT NULL DEFAULT (STRFTIME('%s', 'now')),
    updated_at INTEGER NOT NULL DEFAULT (STRFTIME('%s', 'now'))
);

CREATE INDEX idx_articles_slug ON articles (slug);

CREATE TABLE IF NOT EXISTS followers (
    leader_id INTEGER NOT NULL,
    follower_id INTEGER NOT NULL,

    FOREIGN KEY (leader_id) REFERENCES users (user_id) ON DELETE CASCADE,
    FOREIGN KEY (follower_id) REFERENCES users (user_id) ON DELETE CASCADE,

    UNIQUE (leader_id, follower_id)
);

CREATE TABLE IF NOT EXISTS favorite_articles (
     user_id INTEGER NOT NULL REFERENCES users (user_id) ON DELETE CASCADE,
     article_id INTEGER NOT NULL REFERENCES articles (article_id) ON DELETE CASCADE,

     UNIQUE (user_id, article_id)
);

CREATE TABLE IF NOT EXISTS tags (
    tag_name TEXT NOT NULL,
    article_id INTEGER NOT NULL REFERENCES articles (article_id) ON DELETE CASCADE,

    UNIQUE (tag_name, article_id)
);

CREATE TABLE IF NOT EXISTS comments (
    comment_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    body TEXT NOT NULL,

    article_id INT NOT NULL REFERENCES articles (article_id) ON DELETE CASCADE,
    author_id INT NOT NULL REFERENCES users (user_id) ON DELETE CASCADE,

    created_at INTEGER NOT NULL DEFAULT (STRFTIME('%s', 'now')),
    updated_at INTEGER NOT NULL DEFAULT (STRFTIME('%s', 'now'))
);
