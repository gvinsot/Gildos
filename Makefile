# Gildos — convenience Makefile.
#
# Mirrors the CI checks documented in CONTRIBUTING.md §8 so contributors
# can run them locally without depending on GitHub Actions being wired up.
# Each target is intentionally a one-liner that wraps a standard cargo
# command — no magic, easy to read, easy to copy into CI.

.PHONY: check lint fmt test ci clean help
.DEFAULT_GOAL := help

help:                ## Show this help.
	@awk 'BEGIN {FS = ":.*##"; printf "Targets:\n"} \
	      /^[a-zA-Z_-]+:.*##/ { printf "  \033[36m%-10s\033[0m %s\n", $$1, $$2 }' \
	      $(MAKEFILE_LIST)

check:               ## Typecheck the whole workspace.
	cargo check --workspace --all-targets

lint:                ## Run clippy with warnings as errors.
	cargo clippy --workspace --all-targets -- -D warnings

fmt:                 ## Verify formatting (does not modify files).
	cargo fmt --all -- --check

fmt-fix:             ## Apply rustfmt to all source files.
	cargo fmt --all

test:                ## Run all tests.
	cargo nextest run --workspace || cargo test --workspace

ci: check lint fmt test  ## Run everything CI runs.

clean:               ## Remove build artifacts.
	cargo clean
