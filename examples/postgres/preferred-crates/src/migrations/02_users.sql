create table users(
    id uuid primary key default gen_random_uuid(),
    username text not null,
    password_hash text not null,
    created_at timestamptz not null default now(),
    updated_at timestamptz
);

create unique index users_username_unique on users(lower(username));

select trigger_updated_at('users');
