dev:
	cargo watch -x run


migrate:
	sqlx migrate run

openapi:
	cargo run --bin generate-openapi

install-tools:
	cargo install cargo-watch
	cargo install sqlx-cli --no-default-features --features postgres
