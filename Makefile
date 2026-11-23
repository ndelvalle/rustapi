DATABASE_NAME ?= rustapi

.PHONY: run build test fmt lint check db-create db-drop db-reset

run:
	cargo run

build:
	cargo build --release

test:
	cargo test

fmt:
	cargo fmt

lint:
	cargo fmt --check
	cargo clippy --all-targets -- -D warnings

check: lint test

db-create:
	createdb $(DATABASE_NAME)

db-drop:
	dropdb --if-exists $(DATABASE_NAME)

db-reset: db-drop db-create
