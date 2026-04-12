.PHONY: serve

serve:
	caddy run

bun-dev:
	cd app && bun dev --port 3001

dev-server:
	cargo run --package mosaic-server -- --config packages/server/mosaic.toml --path ~/projects/tmp

prod-server:
	target/release/mosaic-server --config packages/server/mosaic.toml --path ~/projects/tmp