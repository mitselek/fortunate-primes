# Contributing to Fortunate Primes Calculator

Welcome! This guide explains how to work on this project following our proven process.

## Before You Start

This project uses **Test-Driven Development (TDD)** and emphasizes mathematical correctness over speed. Read this entire file before writing code.

## The Feature Request Workflow (TL;DR)

```text
Issue → Design Discuss → Feature Branch → TDD → Quality Gates → Merge & Tag
```

**Every feature follows this path.** No exceptions. No shortcuts.

## 1. Create an Issue

Start here. **Always**.

```markdown
Title: Clear, concise description
Body:

- What problem does this solve?
- Why should we implement it?
- Expected behavior or outcome?
- (For optimizations): Expected speedup?
```

Example:

```markdown
Title: Add Rayon parallel candidate testing

## Problem

Single-threaded candidate loop is bottleneck for n > 200.

## Solution

Use Rayon to test candidates in parallel across CPU cores.

## Expected Outcome

4-8x speedup on n=300-500 range.

## Success Criteria

- All tests pass (including OEIS A005235)
- Benchmark shows ≥4x improvement on n=300
- No regressions on n < 200
```

**Label your issue**: `enhancement`, `optimization`, `bug`, `phase-1`, etc.

## 2. Design Discussion (For Algorithmic Changes)

If your feature involves algorithms, primality tests, or core logic:

**Comment on the issue** with design details:

```markdown
## Design Proposal

### Current Approach

[Explain what we do now]

### Your Approach

[Explain what you'll change]

### Why It's Better

[Technical justification + expected benefit]

### Risks

[What could go wrong?]
[How will you mitigate?]

### Validation Plan

[How will you prove it's correct?]
[OEIS A005235 through n=___?]
[Benchmark comparison plan?]
```

**Wait for approval before coding.** This prevents wasted effort.

## 3. Create Feature Branch

```bash
git checkout -b feature/ISSUE_NUMBER-description
# Example: git checkout -b feature/42-rayon-parallel
```

**Branch naming**: `feature/N-kebab-case` where N is the issue number.

## 4. TDD Workflow

**Write test first. Always.**

```bash
# 1. Write test in src/lib.rs (in #[cfg(test)] mod tests)
#[test]
fn test_parallel_gives_same_results() {
    let calc = PrimeBasedCalculator::new(primes);
    let result = calc.fortunate_number(100).unwrap();
    assert_eq!(result, 641); // Known value from OEIS
}

# 2. Make it fail
cargo test test_parallel_gives_same_results
# Should fail: not implemented yet

# 3. Implement minimal code to pass
# Add your feature implementation

# 4. Make it pass
cargo test test_parallel_gives_same_results
# Should pass now

# 5. Refactor if needed
# Repeat for next feature aspect
```

**Each commit should have passing tests.** No exceptions.

## 5. Quality Gates (Required Before PR)

Run these before creating a PR:

```bash
# Format code
cargo fmt

# Check formatting
cargo fmt -- --check

# Lint
cargo clippy

# Test everything
cargo test

# Benchmark (for optimizations)
./benchmark.sh > after.txt
# Compare with baseline from main branch
```

**All must pass.** If any fail, fix before PR.

## 6. Verify OEIS Correctness

For any change affecting Fortunate calculation:

```bash
# Ensure tests validate against OEIS A005235
# Tests should check:
#   - n=1 → 3
#   - n=5 → 23
#   - n=100 → 641
#   - ... up to your tested range
```

See [test_fortunate_numbers_oeis](src/lib.rs#L457) for example.

## 7. Create PR

```text
Title: Closes #42: Add Rayon parallel candidate testing

Description:
- What changed
- Why it's better (with numbers for optimizations)
- How it was tested
- Checklist confirmation (see below)
```

**Link the issue**: "Closes #42" in PR description.

## 8. Acceptance Checklist

Before merging, confirm **ALL** of these:

- [ ] ✅ **Correctness**: All tests pass (`cargo test`)
- [ ] ✅ **OEIS Validation**: Tests check known Fortunate numbers
- [ ] ✅ **Code Quality**: `cargo fmt` and `cargo clippy` pass
- [ ] ✅ **Performance** (if optimization): Benchmark shows ≥5% improvement
- [ ] ✅ **No Regressions**: Benchmarks don't slow down any existing range
- [ ] ✅ **Documentation**: README/DEVELOPMENT.md updated if needed
- [ ] ✅ **Changelog**: Entry added to CHANGELOG.md
- [ ] ✅ **Version Bump**: Cargo.toml version updated

## 9. Merge & Tag

```bash
# After PR approved:
git checkout main
git pull
git merge --ff-only feature/42-rayon-parallel

# Bump version (if not already done in PR)
# Edit Cargo.toml: 0.1.0 → 0.2.0 (for minor feature)

# Update CHANGELOG.md with your changes

# Commit version bump
git commit -am "bump: v0.1.0 → v0.2.0"

# Tag it
git tag -a v0.2.0 -m "Add Rayon parallel testing"

# Push
git push origin main
git push origin --tags
```

**Versioning** (Semantic Versioning):

- `MAJOR.MINOR.PATCH` (e.g., 1.2.3)
- **Major**: Breaking changes
- **Minor**: New features
- **Patch**: Bug fixes

## Common Patterns

### Pattern: Adding an Optimization

```bash
# 1. Issue with expected speedup numbers
# 2. Feature branch: feature/N-optimization-name
# 3. Baseline: ./benchmark.sh > baseline.txt
# 4. Implement with tests proving correctness
# 5. Benchmark: ./benchmark.sh > after.txt
# 6. Compare: diff baseline.txt after.txt
# 7. PR with benchmark comparison
# 8. Merge only if speedup ≥ expected
```

### Pattern: Adding a New Algorithm

```bash
# 1. Issue with design discussion
# 2. Create struct implementing PrimalityTest trait
# 3. Write tests against known primes + Carmichael numbers
# 4. Add integration test: new algorithm vs Miller-Rabin
# 5. Benchmark comparison in README
# 6. PR with algorithm comparison table
# 7. Merge with updated docs
```

### Pattern: Fixing a Bug

```bash
# 1. Issue describing the bug
# 2. Write failing test that reproduces bug
# 3. Implement fix
# 4. Test passes
# 5. Verify no other tests broke
# 6. PR with test demonstrating fix
# 7. Merge (usually as patch version bump)
```

## Key Rules (No Exceptions)

1. **TDD First** — Tests before code, always
2. **OEIS Validation** — Fortune's conjecture must hold
3. **Benchmark Proof** — Optimizations must be measurable
4. **No Regressions** — Tests pass, benchmarks don't worsen
5. **Document Everything** — README, DEVELOPMENT.md, code comments
6. **Semantic Versioning** — Clear version communication
7. **Issue First** — No code without issue discussion

## When We Say "No"

We politely decline features that:

- ❌ Break existing tests
- ❌ Cause benchmark regression >5%
- ❌ Change core algorithm without proof
- ❌ Violate Fortune's conjecture validation
- ❌ Add complexity with unclear benefit

## Getting Help

**Before coding**, ask questions:

1. Comment on the issue with your approach
2. Ask in PR description if unsure
3. Check [DEVELOPMENT.md](DEVELOPMENT.md) for detailed architecture
4. Look at existing tests for patterns ([src/lib.rs](src/lib.rs#L385))

**Don't have commit access?** Fork the repo, follow the same process on your fork, then submit PR to main repo.

## Project Structure

```text
fortunate-primes/
├── src/
│   ├── lib.rs          # Core logic + tests
│   ├── main.rs         # CLI interface
│   └── primes.rs       # Hardcoded primes
├── Cargo.toml          # Dependencies + version
├── README.md           # User guide
├── DEVELOPMENT.md      # Architecture + detailed workflow
├── CHANGELOG.md        # Version history
├── CONTRIBUTING.md     # This file
└── Makefile            # Convenience targets
```

**Key files**:

- **src/lib.rs** — All traits, implementations, tests live here
- **Cargo.toml** — Bump version here on release
- **CHANGELOG.md** — Document changes here
- **README.md** — User-facing documentation

## Example: Your First Feature

Let's say you want to add support for extended prime list (beyond 1,224).

```text
1. Create Issue #1: "Extend hardcoded prime list to 10,000 primes"
2. Discuss: "We have primes up to 9,973. Would enable n up to 1,229."
3. Branch: git checkout -b feature/1-extend-primes
4. Test: Write test checking primorial(1000) works
5. Implement: Generate and add more primes to src/primes.rs
6. Test passes: cargo test
7. Benchmark: ./benchmark.sh (should be slightly faster, no regression)
8. PR: "Closes #1: Extend prime list to 10,000 primes"
9. Merge & tag: bump to v0.2.0, update CHANGELOG
```

**That's it.** Same pattern for every feature.

## Questions?

1. **How do I run the project?** → See [README.md](README.md)
2. **What's the architecture?** → See [DEVELOPMENT.md](DEVELOPMENT.md)
3. **How do tests work?** → See [src/lib.rs tests section](src/lib.rs#L385)
4. **What if I'm stuck?** → Comment on issue with your blocker

---

**Thank you for contributing!** 🚀

Your code will be built on solid foundations: tested, verified, and documented.
