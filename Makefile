.PHONY: help install validate explain score lint audit status graph codegen \
        kani-stubs lean-stubs probar-stubs invariants scaffold generate \
        proof-status coverage demo test build fmt fmt-check clippy \
        coverage-test ci clean

PV ?= pv
CONTRACTS := contracts/money-v1.yaml \
             contracts/connection-v1.yaml \
             contracts/safe-div-v1.yaml

help:
	@echo "Design by Provable Contracts — pv-centric Makefile"
	@echo ""
	@echo "Every learner command goes THROUGH pv. cargo test still runs"
	@echo "the demos, but the contracts are the source of truth — pv"
	@echo "validates them, scores them, and emits the runtime assertions"
	@echo "the demo binaries assert against."
	@echo ""
	@echo "  Install:"
	@echo "    make install       — cargo install aprender-contracts-cli (provides pv)"
	@echo ""
	@echo "  pv contract gates (one per contract — see CONTRACTS var):"
	@echo "    make validate      — pv validate per contract  (schema gate)"
	@echo "    make explain       — pv explain per contract   (human-readable)"
	@echo "    make score         — pv score per contract     (rubric grade)"
	@echo "    make lint          — pv lint per contract      (validate + audit + score)"
	@echo "    make audit         — pv audit per contract     (traceability audit)"
	@echo "    make status        — pv status per contract    (equations + obligations + coverage)"
	@echo "    make graph         — pv graph per contract     (contract dependency graph)"
	@echo ""
	@echo "  pv artifact generators (emit Rust / Lean / Kani / probar):"
	@echo "    make codegen       — pv codegen → debug_assert! statements"
	@echo "    make kani-stubs    — pv kani    → Kani proof harness stubs"
	@echo "    make lean-stubs    — pv lean    → Lean 4 theorem stubs"
	@echo "    make probar-stubs  — pv probar  → probar property test stubs"
	@echo "    make invariants    — pv invariants → type invariant trait + Kani harness"
	@echo "    make scaffold      — pv scaffold → Rust trait + test scaffolding"
	@echo "    make generate      — pv generate → all artifacts to disk"
	@echo ""
	@echo "  pv project reports:"
	@echo "    make proof-status  — pv proof-status across all contracts (L1-L5 levels)"
	@echo "    make coverage      — pv coverage report across contracts"
	@echo ""
	@echo "  Demo runs (the three crates whose contracts pv gates):"
	@echo "    make demo          — cargo run --bin money-demo, typestate-demo, safediv-demo"
	@echo "    make test          — cargo test --workspace --release"
	@echo "    make build         — cargo build --workspace --release"
	@echo ""
	@echo "  Quality gates:"
	@echo "    make ci            — fmt-check + clippy + test + coverage + lint (full pre-merge)"
	@echo "    make fmt | fmt-check | clippy | coverage-test"
	@echo "    make clean         — cargo clean"

# ---------------------------------------------------------------------------
# Install — the one-time prereq.
# ---------------------------------------------------------------------------

install:
	@command -v $(PV) >/dev/null 2>&1 \
		&& echo "[install] pv already on PATH ($$($(PV) --version 2>&1 | head -1))" \
		|| cargo install aprender-contracts-cli

# ---------------------------------------------------------------------------
# pv contract gates — every one runs against every contract.
# ---------------------------------------------------------------------------

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

# ---------------------------------------------------------------------------
# pv artifact generators — emit Rust assertions, Kani harnesses, Lean stubs.
# ---------------------------------------------------------------------------

codegen:
	@mkdir -p target/pv
	@for c in $(CONTRACTS); do \
		out=target/pv/$$(basename $$c .yaml)-assertions.rs; \
		echo "--- pv codegen $$c -> $$out ---"; \
		$(PV) codegen $$c --output $$out || true; \
	done

kani-stubs:
	@mkdir -p target/pv
	@for c in $(CONTRACTS); do \
		out=target/pv/$$(basename $$c .yaml)-kani.rs; \
		echo "--- pv kani $$c -> $$out ---"; \
		$(PV) kani $$c --output $$out || true; \
	done

lean-stubs:
	@mkdir -p target/pv
	@for c in $(CONTRACTS); do \
		out=target/pv/$$(basename $$c .yaml).lean; \
		echo "--- pv lean $$c -> $$out ---"; \
		$(PV) lean $$c --output $$out || true; \
	done

probar-stubs:
	@mkdir -p target/pv
	@for c in $(CONTRACTS); do \
		out=target/pv/$$(basename $$c .yaml)-probar.rs; \
		echo "--- pv probar $$c -> $$out ---"; \
		$(PV) probar $$c --output $$out || true; \
	done

invariants:
	@mkdir -p target/pv
	@for c in $(CONTRACTS); do \
		out=target/pv/$$(basename $$c .yaml)-invariants.rs; \
		echo "--- pv invariants $$c -> $$out ---"; \
		$(PV) invariants $$c --output $$out || true; \
	done

scaffold:
	@mkdir -p target/pv
	@for c in $(CONTRACTS); do \
		out=target/pv/$$(basename $$c .yaml)-scaffold.rs; \
		echo "--- pv scaffold $$c -> $$out ---"; \
		$(PV) scaffold $$c --output $$out || true; \
	done

generate:
	@mkdir -p target/pv
	@for c in $(CONTRACTS); do \
		echo "--- pv generate $$c (all artifacts) ---"; \
		$(PV) generate $$c --output-dir target/pv/ || true; \
	done

# ---------------------------------------------------------------------------
# pv project reports — across the contract set.
# ---------------------------------------------------------------------------

proof-status:
	$(PV) proof-status contracts/

coverage:
	$(PV) coverage contracts/

# ---------------------------------------------------------------------------
# Demo runs — the binaries whose runtime behaviour the contracts gate.
# ---------------------------------------------------------------------------

demo:
	@echo "=== M2 newtype: Money<C> currency safety ==="
	@cargo run --release --bin money-demo
	@echo ""
	@echo "=== M3 typestate: Connection<Authenticated> ==="
	@cargo run --release --bin typestate-demo
	@echo ""
	@echo "=== M4 proptest: safe_div over i32 x i32 ==="
	@cargo run --release --bin safediv-demo -- 10 2
	@cargo run --release --bin safediv-demo -- 5 0

test:
	cargo test --workspace --release

build:
	cargo build --workspace --release

# ---------------------------------------------------------------------------
# Quality gates — full pre-merge check.
# ---------------------------------------------------------------------------

ci: fmt-check clippy test coverage-test lint

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all -- --check

clippy:
	cargo clippy --workspace --all-targets -- -D warnings

coverage-test:
	cargo llvm-cov --workspace --release --fail-under-lines 100

clean:
	cargo clean
	rm -rf target/pv

# ---------------------------------------------------------------------------
# Lean 4 proofs — `lake build` against lean/ProvableContracts/
# ---------------------------------------------------------------------------

.PHONY: lean-build lean-clean

lean-build:
	cd lean && lake build

lean-clean:
	cd lean && lake clean
