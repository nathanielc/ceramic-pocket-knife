CARGO ?= RUSTFLAGS="-D warnings" cargo

FEATURES = ceramic ipld multibase multihash p2p

all: build test check-fmt check-clippy

.PHONY: build
build:
	${CARGO} build --all-features
	$(foreach f,$(FEATURES),$(CARGO) build --no-default-features --features $(f) &&) true

.PHONY: test
test:
	${CARGO} test --all-features
	$(foreach f,$(FEATURES),$(CARGO) test --no-default-features --features $(f) &&) true

.PHONY: check-fmt
check-fmt:
	${CARGO} fmt --all -- --check

.PHONY: check-clippy
check-clippy:
	${CARGO} clippy --workspace --all-features
	$(foreach f,$(FEATURES),$(CARGO) clippy --workspace --no-default-features --features $(f) &&) true

