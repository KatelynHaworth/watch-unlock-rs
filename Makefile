default: build

lint-cli:
	@echo "Linting CLI"
	@cargo clippy --bin watch_unlock_cli --features="cli"

lint-pam:
	@echo "Linting PAM"
	@cargo clippy --lib --features="lib"

lint: lint-cli lint-pam

build-cli-release:
	@echo "Building CLI [release]"
	@cargo build --release --bin watch_unlock_cli --features="cli"

build-cli-dev:
	@echo "Building CLI [dev]"
	@cargo build --bin watch_unlock_cli --features="cli"

build-pam-release:
	@echo "Building PAM [release]"
	@cargo build --release --lib --features="lib"

build-pam-dev:
	@echo "Building PAM [release]"
	@cargo build --lib --features="lib"

build: build-cli-release build-pam-release
build-dev: build-cli-dev build-pam-dev

install:
	@cp ./conf/pam.d/apple-watch /etc/pam.d/
	@cp ./conf/security/apple_watch.conf /etc/security/
	@cp ./target/release/libpam_apple_watch.so /lib/security/pam_apple_watch.so
	@cp ./target/release/watch_unlock_cli /usr/bin/
