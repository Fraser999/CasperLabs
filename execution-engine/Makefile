# This supports environments where $HOME/.cargo/env has not been sourced (CI, CLion Makefile runner)
CARGO  = $(or $(shell which cargo),  $(HOME)/.cargo/bin/cargo)
RUSTUP = $(or $(shell which rustup), $(HOME)/.cargo/bin/rustup)

RUST_TOOLCHAIN := $(shell cat rust-toolchain)

SYSTEM_CONTRACTS          = build-contracts/system
CONTRACTS_MANIFEST_ARG    = --manifest-path=contracts/Cargo.toml
CONTRACT_TARGET_DIR       = target/wasm32-unknown-unknown/release
PACKAGED_SYSTEM_CONTRACTS = mint_token.wasm pos.wasm

.PHONY: all
all: build

.PHONY: build
build: $(SYSTEM_CONTRACTS)
	$(CARGO) build $(CARGO_FLAGS)

build-contract/%:
	$(CARGO) build $(filter-out --release, $(CARGO_FLAGS)) -p $* --release $(CONTRACTS_MANIFEST_ARG) --target-dir=target

build-contracts/%:
	$(CARGO) build $(filter-out --release, $(CARGO_FLAGS)) --no-default-features --features=$* --release $(CONTRACTS_MANIFEST_ARG) --target-dir=target

.PHONY: test
test:
	RUST_BACKTRACE=1 $(CARGO) test $(CARGO_FLAGS) -- --nocapture

.PHONY: check-format
check-format:
	$(CARGO) fmt --all -- --check
	$(CARGO) fmt --all $(CONTRACTS_MANIFEST_ARG) -- --check

.PHONY: format
format:
	$(CARGO) fmt --all
	$(CARGO) fmt --all $(CONTRACTS_MANIFEST_ARG)

.PHONY: lint
lint:
	$(CARGO) clippy --all-targets --all -- -D warnings -A renamed_and_removed_lints
	$(CARGO) clippy --all-targets --all --exclude casperlabs-contracts $(CONTRACTS_MANIFEST_ARG) -- -D warnings -A renamed_and_removed_lints

.PHONY: check
check: \
	check-format \
	build \
	lint \
	test

.PHONY: clean
clean:
	rm -f comm/.rpm
	$(CARGO) clean
	$(CARGO) clean $(CONTRACTS_MANIFEST_ARG)

.PHONY: deb
deb:
	cd engine-grpc-server && $(CARGO) deb

engine-grpc-server/.rpm:
	cd engine-grpc-server && $(CARGO) rpm init

.PHONY: rpm
rpm: engine-grpc-server/.rpm
	cd engine-grpc-server && $(CARGO) rpm build

target/system-contracts.tar.gz: $(SYSTEM_CONTRACTS)
	tar -czf $@ -C $(CONTRACT_TARGET_DIR) $(PACKAGED_SYSTEM_CONTRACTS)

.PHONY: package-system-contracts
package-system-contracts: target/system-contracts.tar.gz

.PHONY: package
package:
	cd contract-ffi && $(CARGO) package

.PHONY: publish
publish:
	cd contract-ffi && $(CARGO) publish

.PHONY: check-publish
check-publish:
	cd contract-ffi && $(CARGO) publish --dry-run

.PHONY: bench
bench:
	$(CARGO) bench

.PHONY: setup-cargo-packagers
setup-cargo-packagers:
	$(CARGO) install cargo-rpm || exit 0
	$(CARGO) install cargo-deb || exit 0

.PHONY: setup
setup: rust-toolchain
	$(RUSTUP) update
	$(RUSTUP) toolchain install $(RUST_TOOLCHAIN)
	$(RUSTUP) target add --toolchain $(RUST_TOOLCHAIN) wasm32-unknown-unknown
	$(RUSTUP) component add --toolchain $(RUST_TOOLCHAIN) rustfmt clippy
