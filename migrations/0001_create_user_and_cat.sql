-- The user table name is quoted everywhere because user is a reserved word.
create table "user" (
    id uuid primary key,
    name text not null,
    email text not null unique,
    password text not null,
    locked_at timestamptz,
    created_at timestamptz not null,
    updated_at timestamptz not null
);

create table cat (
    id uuid primary key,
    user_id uuid not null references "user" (id) on delete cascade,
    name text not null,
    created_at timestamptz not null,
    updated_at timestamptz not null
);

create index cat_user_id_created_at_idx on cat (user_id, created_at);
