-- Sets updated_at on every update that changes a row.
create function set_updated_at()
    returns trigger as
$$
begin
    new.updated_at = now();
    return new;
end;
$$ language plpgsql;

-- Usage after creating a table: select trigger_updated_at('<table name>');
create function trigger_updated_at(tablename regclass)
    returns void as
$$
begin
    execute format('create trigger set_updated_at
        before update
        on %s
        for each row
        when (old is distinct from new)
    execute function set_updated_at();', tablename);
end;
$$ language plpgsql;

-- Compares text ignoring case, so that a unique column rejects values that only differ in case.
create collation case_insensitive (provider = icu, locale = 'und-u-ks-level2', deterministic = false);
