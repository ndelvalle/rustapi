-- The table name is quoted everywhere because user is a reserved word.
create table "user" (
    -- UUIDv7 IDs increase with time, so they sort by creation order.
    user_id uuid primary key default uuidv7(),
    name text not null,
    email text collate case_insensitive unique not null,
    password text not null,
    locked_at timestamptz,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now()
);

select trigger_updated_at('"user"');
