create table post (
    post_id uuid primary key default gen_random_uuid(),
    user_id uuid not null references "user"(user_id),
    content text not null,
    created_at timestamptz not null default now()
);

create index on post(created_at desc);
