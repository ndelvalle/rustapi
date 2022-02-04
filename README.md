# rustapi

[![CI](https://github.com/ndelvalle/rustapi/actions/workflows/ci.yml/badge.svg)](https://github.com/ndelvalle/rustapi/actions/workflows/ci.yml)

RESTful API template built with Rust lang. It uses [MongoDB](https://docs.mongodb.com/)
database and [Axum](https://github.com/tokio-rs/axum) HTTP framework.

### Requirements

- [Rust](https://www.rust-lang.org/tools/install)
- [MongoDB](https://docs.mongodb.com/manual/installation/)

### How to

To use this template as your project starting point, click "Use this template" at the top of this page, or click [here](https://github.com/ndelvalle/rustapi/generate).


### Project structure

```bash
├── Cargo.lock
├── Cargo.toml
├── README.md
├── config
│   ├── default.json    # Default configuration
│   └── production.json # Production configuration (Overwrites the default)
├── rustfmt.toml
└── src
    ├── context.rs      # Shared state and functionality across the APP
    ├── database.rs
    ├── errors.rs
    ├── lib             # Custom helpers not related to the business model
    │   ├── authenticate_request.rs
    │   ├── date.rs
    │   ├── mod.rs
    │   └── token.rs
    ├── logger.rs
    ├── main.rs
    ├── models
    │   ├── cat.rs
    │   ├── mod.rs
    │   └── user.rs
    ├── routes
    │   ├── cat.rs
    │   ├── mod.rs
    │   └── user.rs
    └── settings.rs
```

## Contributing

Contributors are welcome, please fork and send pull requests! If you find a bug
or have any ideas on how to improve this project please submit an issue.
