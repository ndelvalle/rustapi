# rustapi

[![Tests](https://github.com/ndelvalle/rustapi/actions/workflows/test.yml/badge.svg?branch=master)](https://github.com/ndelvalle/rustapi/actions/workflows/test.yml)

RESTful API template built with Rust lang. It uses a [PostgreSQL](https://www.postgresql.org/)
database through [sqlx](https://github.com/launchbadge/sqlx) and the [Axum](https://github.com/tokio-rs/axum) HTTP framework.

### Requirements

- [Rust](https://www.rust-lang.org/tools/install)
- [PostgreSQL](https://www.postgresql.org/download/)

### How to use this template

To use this template as your project starting point, click "Use this template" at the top of this page, or click [here](https://github.com/ndelvalle/rustapi/generate).

### Feature highlights

* Authentication. Based on [jsonwebtoken](https://github.com/Keats/jsonwebtoken)
* Layered configuration. Based on [config-rs](https://github.com/rust-cli/config-rs)
* SQL migrations and [UUIDv7](https://www.rfc-editor.org/rfc/rfc9562#name-uuid-version-7) IDs
* Logs. Based on [tracing](https://github.com/tokio-rs/tracing)
* Error handling
* Graceful shutdown, request timeout, body size limit and request IDs
* Status route that checks the database, usable as a readiness probe
* Pagination
* E2E Tests
* OpenAPI Specification
* CI based on Github actions
* Dependabot configuration

### Project structure

```bash
├── Cargo.lock
├── Cargo.toml
├── Makefile
├── README.md
├── config
│   ├── default.json     # Default configuration
│   ├── development.json # Development configuration (Overwrites the default)
│   ├── production.json  # Production configuration (Overwrites the default)
│   └── test.json        # Test configuration (Overwrites the default)
├── migrations           # SQL migrations, applied on startup
├── openapi.yaml         # OpenAPI specification
└── src
    ├── app.rs           # Router and middleware
    ├── database.rs      # Connection pool and migrations
    ├── errors.rs
    ├── logger.rs
    ├── main.rs
    ├── models           # Structs and the SQL queries that load and store them
    │   ├── cat.rs
    │   ├── mod.rs
    │   └── user.rs
    ├── routes
    │   ├── cat.rs
    │   ├── mod.rs
    │   ├── status.rs
    │   └── user.rs
    ├── settings.rs
    ├── tests            # E2E Tests
    └── utils            # Helpers not related to the business model
        ├── authenticate_request.rs
        ├── custom_response.rs
        ├── date.rs
        ├── mod.rs
        ├── pagination.rs
        ├── to_uuid.rs
        └── token.rs
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

### Test

To run tests make sure PostgreSQL is up and running and the test database exists.

```
createdb rustapi_test
make test
```

## Contributing

Contributors are welcome, please fork and send pull requests! If you find a bug
or have any ideas on how to improve this project please submit an issue.
