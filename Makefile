BIN = ccnotify-rs
INSTALL_PATH = ~/.cargo/bin/$(BIN)
CODESIGN_ID = com.ccnotify.ccnotify-rs

.PHONY: help build install test clean
help: ## Show this help
	@echo "ccnotify-rs Makefile"
	@echo ""
	@echo "Targets:"
	@echo "  build   Build release binary    (cargo build --release + codesign)"
	@echo "  install Build and copy to ~/.cargo/bin/"
	@echo "  test    Run tests              (cargo test)"
	@echo "  clean   Clean build artifacts  (cargo clean)"
	@echo "  help    Show this help"

install: build
	@mkdir -p ~/.cargo/bin
	@cp target/release/$(BIN) $(INSTALL_PATH)
	@chmod +x $(INSTALL_PATH)

build:
	@cargo build --release
	@codesign -s - -i $(CODESIGN_ID) --force target/release/$(BIN)

test:
	@cargo test

clean:
	@cargo clean
