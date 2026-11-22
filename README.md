# rustapi

[![Tests](https://github.com/ndelvalle/rustapi/actions/workflows/test.yml/badge.svg?branch=master)](https://github.com/ndelvalle/rustapi/actions/workflows/test.yml)

RESTful API template built with Rust lang. It uses a [PostgreSQL](https://www.postgresql.org/)
database through [sqlx](https://github.com/launchbadge/sqlx) and the [Axum](https://github.com/tokio-rs/axum) HTTP framework.

### Requirements

- [Rust](https://www.rust-lang.org/tools/install)
- [PostgreSQL](https://www.postgresql.org/download/) 18 or newer, for its built-in `uuidv7()`

### How to use this template

To use this template as your project starting point, click "Use this template" at the top of this page, or click [here](https://github.com/ndelvalle/rustapi/generate).

### Feature highlights

* Authentication. Based on [jsonwebtoken](https://github.com/Keats/jsonwebtoken)
* Layered configuration. Based on [config-rs](https://github.com/rust-cli/config-rs)
* SQL migrations. The database generates [UUIDv7](https://www.rfc-editor.org/rfc/rfc9562#name-uuid-version-7) IDs and maintains the timestamps
* Logs. Based on [tracing](https://github.com/tokio-rs/tracing)
* Error handling, with the invalid fields listed in validation errors
* Graceful shutdown, request timeout, body size limit and request IDs
* Status route that checks the database
* Pagination
* E2E Tests
* CI based on Github actions
* Dependabot configuration

### Project structure

```bash
├── Cargo.lock
├── Cargo.toml
├── LICENSE
├── Makefile
├── README.md
├── config
│   ├── default.json     # Default configuration
│   ├── development.json # Development configuration (Overwrites the default)
│   ├── production.json  # Production configuration (Overwrites the default)
│   └── test.json        # Test configuration (Overwrites the default)
├── migrations           # SQL migrations, applied on startup
├── src
│   ├── app.rs           # Router, middleware and the context shared with the handlers
│   ├── common           # Helpers not related to the business model
│   │   ├── authenticate_request.rs
│   │   ├── custom_response.rs
│   │   ├── date.rs
│   │   ├── mod.rs
│   │   ├── pagination.rs
│   │   ├── to_uuid.rs
│   │   └── token.rs
│   ├── database.rs      # Connection pool and migrations
│   ├── errors.rs
│   ├── lib.rs           # The API as a library, shared by the binary and the tests
│   ├── logger.rs
│   ├── main.rs          # Loads the settings and starts the server
│   ├── models           # Structs and the SQL queries that load and store them
│   │   ├── cat.rs
│   │   ├── mod.rs
│   │   └── user.rs
│   ├── routes           # Handlers with their request and response bodies
│   │   ├── cat.rs
│   │   ├── mod.rs
│   │   ├── status.rs
│   │   └── user.rs
│   └── settings.rs
└── tests
    └── api              # E2E Tests
```

### Run

Create the database and start the server. Migrations are applied on startup.

```
createdb rustapi
cargo run
```

Configuration is layered. `config/default.json` is loaded first, then
`config/<RUN_MODE>.json` (`development` when `RUN_MODE` is not set), then an
optional `config/local.json`, and finally environment variables, where `__`
separates the levels. For example, `DATABASE__URI` overrides `database.uri`.

Only development and test ship with a JWT secret. In any other mode the server
refuses to start until `AUTH__SECRET` is set.

Browsers may only call the API from the origins in `cors.allowed_origins`. It
is `*` (any origin) in development and empty everywhere else, so set it for
production, for example `CORS__ALLOWED_ORIGINS=https://app.example.com,https://admin.example.com`.

### API documentation

This template is deliberately minimal, so it does not generate or serve an
OpenAPI specification. Most private APIs never need to publish one. If yours
does, [utoipa](https://github.com/juhaku/utoipa) generates it from the handlers
and integrates well with axum.

### Test

To run tests make sure PostgreSQL is up and running.

```
cargo test
```

Every test starts its own instance of the API on a random port, backed by a
temporary database that is created and migrated for that test, so tests run in
parallel. The databases are created on the server that `DATABASE_URL` points
to, which defaults to `postgres://localhost:5432/postgres` (see
`.cargo/config.toml`).

## Contributing

Contributors are welcome, please fork and send pull requests! If you find a bug
or have any ideas on how to improve this project please submit an issue.

## License

[MIT](LICENSE)
