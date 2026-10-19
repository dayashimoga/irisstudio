# Iris Studio: Release Process & Quality Gates

## 1. Release Cadence & Versioning
Iris Studio follows Semantic Versioning (`MAJOR.MINOR.PATCH`):
- `MAJOR`: Significant architectural overhauls or breaking project schema format changes.
- `MINOR`: New functional workspaces, codec additions, or major editing capabilities.
- `PATCH`: Bug fixes, security patches, performance improvements, and minor UI polishes.

---

## 2. Mandatory Quality Gates Before Release

Before any tag or binary artifact can be published:
1. **Compilation**: Clean compilation on all target platforms (Windows x64/ARM64, Linux x64/ARM64, macOS Universal, Android APK/AAB, iOS IPA).
2. **Linter & Formatting**:
   - `cargo fmt --all -- --check`
   - `cargo clippy --workspace --all-targets -- -D warnings`
   - `flutter analyze`
3. **Automated Test Suite**:
   - 100% of unit, integration, and regression tests must pass (`cargo test --workspace`).
4. **Performance Benchmark Verification**:
   - Criterion microbenchmarks must not regress by >5% compared to baseline.
5. **Security & Fuzzing**:
   - Zero crashes detected over 100,000 fuzzing cycles on media decoders and project deserializers.
   - Zero critical CVEs in dependencies (`cargo audit`).
6. **Artifact Generation & Integrity**:
   - Generate SHA-256 checksums and CycloneDX / SPDX Software Bill of Materials (SBOM) for all released installers.
   - Code signing with valid platform certificates (Windows Authenticode, Apple Developer ID).
