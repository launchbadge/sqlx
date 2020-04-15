CREATE TABLE IF NOT EXISTS users (
    user_id SERIAL PRIMARY KEY,

    email TEXT UNIQUE NOT NULL,
    password TEXT NOT NULL,
    username TEXT UNIQUE NOT NULL,
    bio TEXT,
    image TEXT,

    created_at TIMESTAMPTZ NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc'),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc')
);

-- This is implemented as a view for demonstration purposes
CREATE VIEW profiles AS
SELECT user_id, username, bio, image
FROM users;

CREATE TABLE IF NOT EXISTS articles (
    article_id SERIAL PRIMARY KEY,
    title TEXT UNIQUE NOT NULL,
    description TEXT NOT NULL,
    slug TEXT UNIQUE NOT NULL,
    body TEXT NOT NULL,
    author_id INT NOT NULL REFERENCES users (user_id) ON DELETE CASCADE,

    created_at TIMESTAMPTZ NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc'),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc')
);

-- many queries are performed via slug
CREATE INDEX ON articles (slug);

CREATE TABLE IF NOT EXISTS followers (
    leader_id INT NOT NULL,
    follower_id INT NOT NULL,

    FOREIGN KEY (leader_id) REFERENCES users (user_id) ON DELETE CASCADE,
    FOREIGN KEY (follower_id) REFERENCES users (user_id) ON DELETE CASCADE,

    UNIQUE (leader_id, follower_id)
);

CREATE TABLE IF NOT EXISTS favorite_articles (
    user_id INT NOT NULL REFERENCES users (user_id) ON DELETE CASCADE,
    article_id INT NOT NULL REFERENCES articles (article_id) ON DELETE CASCADE,

    UNIQUE (user_id, article_id)
);

CREATE TABLE IF NOT EXISTS tags (
    tag_name TEXT NOT NULL,
    article_id INT NOT NULL REFERENCES articles (article_id) ON DELETE CASCADE,

    UNIQUE (tag_name, article_id)
);

CREATE TABLE IF NOT EXISTS comments (
    comment_id SERIAL PRIMARY KEY,
    body TEXT NOT NULL,

    article_id INT NOT NULL REFERENCES articles (article_id) ON DELETE CASCADE,
    author_id INT NOT NULL REFERENCES users (user_id) ON DELETE CASCADE,

    created_at TIMESTAMPTZ NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc'),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc')
);
