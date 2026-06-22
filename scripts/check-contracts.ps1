# check-contracts.ps1 — Show current contract test compilation status.
#
# The Phase 2 contract tests are intentional compile-time failures until
# Phase 4 (models) and Phase 5 (client methods) land. This script makes
# that state explicit rather than silently excluded from the pre-commit hook.
#
# Usage:
#   pwsh scripts/check-contracts.ps1
#   powershell scripts/check-contracts.ps1
#
# Expected output until Phase 4+: unresolved import and missing method errors.
# Expected output after Phase 5: all tests pass.

Write-Host "=== Contract test status (failures expected until Phase 4/5) ===" -ForegroundColor Cyan
Write-Host ""
cargo test
$exitCode = $LASTEXITCODE
Write-Host ""
Write-Host "=== Current passing gate ===" -ForegroundColor Cyan
Write-Host "  cargo fmt --check"
Write-Host "  cargo clippy --lib --all-features -- -D warnings"
Write-Host "  cargo check --lib"
Write-Host ""
Write-Host "Restore --all-targets and cargo test in .githooks/pre-commit once Phase 4+ is complete." -ForegroundColor Yellow
exit $exitCode
