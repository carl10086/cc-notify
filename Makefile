BIN = ccnotify-rs
INSTALL_PATH = ~/.cargo/bin/$(BIN)

install: build
	@mkdir -p ~/.cargo/bin
	@cp target/release/$(BIN) $(INSTALL_PATH)
	@chmod +x $(INSTALL_PATH)

build:
	@cargo build --release

test:
	@cargo test

clean:
	@cargo clean
