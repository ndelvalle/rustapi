create table cat (
    cat_id uuid primary key default uuidv7(),
    user_id uuid not null references "user" (user_id) on delete cascade,
    name text not null,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now()
);

create index cat_user_id_created_at_idx on cat (user_id, created_at);

select trigger_updated_at('cat');
