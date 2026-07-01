set shell := ["powershell.exe", "-c"]

# ── Docker ──────────────────────────────────────────────────────────
up:
    docker compose up -d

wait:
    Write-Host "Waiting for bitcoind..."; Start-Sleep -Seconds 10

down:
    docker compose down -v

# ── Rust ───────────────────────────────────────────────────────────
build:
    Push-Location rust; cargo build; if (-not $?) { exit 1 }; Pop-Location

run:
    Push-Location rust; cargo run; if (-not $?) { exit 1 }; Pop-Location

fmt:
    Push-Location rust; cargo fmt; Pop-Location

fmt-check:
    Push-Location rust; cargo fmt --check; Pop-Location

clippy:
    Push-Location rust; cargo clippy --all-targets --all-features -- -D warnings; Pop-Location

lint: fmt-check clippy

# ── Test (delegates to test.sh) ─────────────────────────────────────
test:
    bash test.sh
