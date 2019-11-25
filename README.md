# rustapi

RESTful API template built with Rust lang. It uses [Postgresql](https://www.postgresql.org/) database and [Rocket](https://rocket.rs/) rust HTTP framework

## Development

### Requirements

- [Rust](https://www.rust-lang.org/tools/install)
- [Postgresql](https://www.postgresql.org/download/)
- [Cargo make plugin](https://github.com/sagiegurari/cargo-make)

### Running

- Start postgresql database
- Run migrations: `cargo make --makefile ./cargo.toml migration-up`
- Start API in dev mode: `cargo make --makefile ./cargo.toml dev`

## Contributing

Contributors are welcome, please fork and send pull requests! If you find a bug or have any ideas on how to improve this project please submit an issue.
