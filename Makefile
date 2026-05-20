.DELETE_ON_ERROR:
.ONESHELL:
.SUFFIXES:

.PHONY: help install validate explain score lint audit status graph codegen \
        proof-status coverage demo test build bench fmt fmt-check clippy \
        coverage-test ci comply comply-init clean \
        notebooks-build notebooks-fmt notebooks-fmt-check notebooks-lint \
        notebooks-test notebooks-ci

PV ?= pv
PMAT ?= pmat

CONTRACTS := contracts/complexity-constant-v1.yaml \
             contracts/complexity-logarithmic-v1.yaml \
             contracts/complexity-linear-v1.yaml \
             contracts/complexity-linearithmic-v1.yaml \
             contracts/complexity-quadratic-v1.yaml \
             contracts/complexity-exponential-v1.yaml \
             contracts/iterator-fusion-v1.yaml \
             contracts/complexity-preserved-across-transpile-v1.yaml

help:
	@echo "big-o-python-to-rust — companion repo for the Big O: Python to Rust course"
	@echo ""
	@echo "Three modes of proof: empirical (criterion), structural (recurrence/amortization),"
	@echo "formal (Lean). Each complexity class is named as a YAML contract that gates the build."
	@echo ""
	@echo "  Install:"
	@echo "    make install       — cargo install aprender-contracts-cli (provides pv) and pmat"
	@echo ""
	@echo "  pv contract gates:"
	@echo "    make validate      — pv validate per contract  (schema gate)"
	@echo "    make score         — pv score per contract     (5-dim rubric)"
	@echo "    make lint          — pv lint per contract"
	@echo "    make audit         — pv audit per contract"
	@echo ""
	@echo "  pmat compliance:"
	@echo "    make comply-init   — pmat comply init  (one-time, creates .pmat/project.toml)"
	@echo "    make comply        — pmat comply check (contract coverage gate)"
	@echo ""
	@echo "  Build + bench + test:"
	@echo "    make build         — cargo build --workspace --release"
	@echo "    make test          — cargo test  --workspace --release"
	@echo "    make bench         — cargo bench --workspace"
	@echo "    make demo          — run every demo binary on native target"
	@echo ""
	@echo "  Quality gates:"
	@echo "    make ci            — Rust + notebook gates (fmt + clippy + test + coverage + lint + comply + notebooks-ci)"
	@echo "    make coverage-test — cargo llvm-cov --fail-under-lines 100 (100% coverage required)"
	@echo ""
	@echo "  Python companion notebooks (uv + ruff + nbclient):"
	@echo "    make notebooks-build      — regenerate notebooks/*.ipynb from scripts/build_notebooks.py"
	@echo "    make notebooks-fmt        — ruff format notebooks/ and scripts/"
	@echo "    make notebooks-fmt-check  — ruff format --check (CI mode)"
	@echo "    make notebooks-lint       — ruff check notebooks/ and scripts/"
	@echo "    make notebooks-test       — execute every notebook top-to-bottom; asserts must pass"
	@echo "    make notebooks-ci         — build + fmt-check + lint + test"

install:
	@if command -v $(PV) >/dev/null 2>&1; then \
		echo "[install] pv already on PATH ($$($(PV) --version 2>&1 | head -1))"; \
	else \
		cargo install aprender-contracts-cli || exit 1; \
	fi
	@if command -v $(PMAT) >/dev/null 2>&1; then \
		echo "[install] pmat already on PATH"; \
	else \
		cargo install pmat || exit 1; \
	fi

validate:
	@for c in $(CONTRACTS); do echo "--- pv validate $$c ---"; $(PV) validate $$c; done

explain:
	@for c in $(CONTRACTS); do echo "--- pv explain $$c ---"; $(PV) explain $$c; done

score:
	@for c in $(CONTRACTS); do echo "--- pv score $$c ---"; $(PV) score $$c; done

lint:
	@for c in $(CONTRACTS); do echo "--- pv lint $$c ---"; $(PV) lint $$c; done

audit:
	@for c in $(CONTRACTS); do echo "--- pv audit $$c ---"; $(PV) audit $$c; done

status:
	@for c in $(CONTRACTS); do echo "--- pv status $$c ---"; $(PV) status $$c; done

graph:
	@for c in $(CONTRACTS); do echo "--- pv graph $$c ---"; $(PV) graph $$c; done

codegen:
	@mkdir -p target/pv
	@$(PV) codegen contracts/ --output target/pv/all-assertions.rs

proof-status:
	$(PV) proof-status contracts/

comply-init:
	$(PMAT) comply init

comply:
	$(PMAT) comply check

build:
	cargo build --workspace --release

test:
	PROPTEST_CASES=256 cargo test --workspace --release

bench:
	cargo bench --workspace

demo:
	@echo "=== m1-modes: three modes of proof tour ==="
	@cargo run --release --bin m1-tour
	@echo "=== m2-empirical: criterion contract harness ==="
	@cargo run --release --bin m2-empirical-demo
	@echo "=== m3-structural: master theorem + amortized analysis ==="
	@cargo run --release --bin m3-structural-demo
	@echo "=== m4-systems: cache + parallel complexity demo ==="
	@cargo run --release --bin m4-systems-demo
	@echo "=== m5-capstone: novel-algorithm characterization ==="
	@cargo run --release --bin m5-capstone

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all -- --check

clippy:
	cargo clippy --workspace --all-targets -- -D warnings

coverage-test:
	cargo llvm-cov --workspace --release --ignore-filename-regex 'main\.rs|benches/' --fail-under-lines 100

# ============================================================================
# Python companion notebooks (one per module, Colab-runnable)
# ============================================================================
# All targets use uv-managed virtualenv. ruff is the lint + format tool;
# nbclient executes each notebook top-to-bottom to verify every assert passes.
# ----------------------------------------------------------------------------

notebooks-build:
	uv run python scripts/build_notebooks.py
	uv run ruff format notebooks/ scripts/

notebooks-fmt:
	uv run ruff format notebooks/ scripts/

notebooks-fmt-check:
	uv run ruff format --check notebooks/ scripts/

notebooks-lint:
	uv run ruff check notebooks/ scripts/

notebooks-test:
	@for nb in notebooks/*.ipynb; do \
		echo "--- executing $$nb ---"; \
		uv run jupyter nbconvert --to notebook --execute --inplace --log-level=WARN "$$nb" || exit 1; \
	done
	uv run ruff format notebooks/

notebooks-ci: notebooks-build notebooks-fmt-check notebooks-lint notebooks-test

ci: fmt-check clippy test coverage-test lint comply notebooks-ci

clean:
	cargo clean || exit 1
	rm -rf target/pv .pmat/.cache || true
