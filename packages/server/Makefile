.PHONY: build deploy

build:
	cargo build --release --target x86_64-unknown-linux-musl

deploy:
	scp target/x86_64-unknown-linux-musl/release/mosaic admin@nas-ricsi.local:mosaic/
	scp gallery_index.html admin@nas-ricsi.local:mosaic/
