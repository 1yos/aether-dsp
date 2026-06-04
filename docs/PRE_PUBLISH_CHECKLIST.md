# Pre-Publish Checklist for crates.io

**Target:** aetherdsp-midi v0.1.5 (tuning systems update)

## ✅ Code Quality

- [x] All tuning systems implemented (13 total)
- [x] Source attribution added to all tunings
- [x] Documentation updated (precision, pitch-bend)
- [x] Tests passing (14 passed, 1 ignored)
- [x] Clippy clean (no warnings)
- [ ] **TODO:** Run `cargo fmt --all`
- [ ] **TODO:** Bump version to 0.1.5

## ✅ Documentation

- [x] README.md updated with all 17 tuning systems (7 Ethiopian, 3 Arabic, 1 Indian, 3 Gamelan, 3 Western)
- [x] Code examples show new functions
- [x] Module-level docs explain precision and pitch-bend
- [x] All tunings have source attribution comments
- [ ] **TODO:** Run `cargo doc -p aetherdsp-midi --no-deps` to verify

## ⚠️ Version Bumping

Current version: **0.1.4**  
New version: **0.1.5** (minor feature addition)

**Files to update:**

1. `Cargo.toml` (workspace version)
2. `crates/aether-midi/CHANGELOG.md` (add v0.1.5 entry)

**Changes in v0.1.5:**

- Added `arabic_maqam_hijaz()` - Augmented 2nd tetrachord
- Added `ethiopian_ambassel()` - Pentatonic with raised 4th
- Added `gamelan_slendro_stretched()` - Ethnomusicologically accurate (1210-cent octaves)
- Added `just_intonation_7_limit()` - Septimal intervals for blues/barbershop
- Added comprehensive source attribution to all tuning systems
- Documented f32 precision limits and pitch-bend behavior
- Total: 17 tuning systems — all 7 Ethiopian qenet modes implemented (was 13)

## 📋 Pre-Publish Commands

```bash
# 1. Format code
cargo fmt --all

# 2. Update version in Cargo.toml
# Change: version = "0.1.4"
# To:     version = "0.1.5"

# 3. Update CHANGELOG
# Add v0.1.5 section with changes

# 4. Verify docs build
cargo doc -p aetherdsp-midi --no-deps --open

# 5. Test package
cargo package -p aetherdsp-midi --allow-dirty

# 6. Dry-run publish
cargo publish -p aetherdsp-midi --dry-run

# 7. Commit version bump
git add -A
git commit -m "Bump aetherdsp-midi to v0.1.5"
git tag -a aetherdsp-midi-v0.1.5 -m "Add 4 new tuning systems and source attribution"
git push origin main --tags

# 8. Publish to crates.io
cargo publish -p aetherdsp-midi
```

## ⚠️ Important Notes

1. **Don't publish aether-host** - It's not ready (has failing tests)
2. **Don't publish aether-ui** - It's marked as not published
3. **Only publish aetherdsp-midi** - It's the only crate with changes

## 🔍 What Changed

**New Tuning Systems (4):**

1. `arabic_maqam_hijaz()` - Missing from docs, now implemented
2. `ethiopian_ambassel()` - Missing from docs, now implemented
3. `gamelan_slendro_stretched()` - Ethnomusicologically accurate variant
4. `just_intonation_7_limit()` - Septimal intervals (7/4, 7/6, 7/5)

**Documentation Improvements:**

- Source attribution for all tunings (research papers, standards)
- f32 precision documentation (~0.0001 Hz at 440 Hz)
- Pitch-bend behavior documentation (relative to tuned pitch)
- Musicological notes and TODO markers for validation

**No Breaking Changes:**

- All existing functions unchanged
- Only additions (new functions)
- Semver: Minor version bump (0.1.4 → 0.1.5)

## ✅ Post-Publish

- [ ] Verify on crates.io: https://crates.io/crates/aetherdsp-midi
- [ ] Check docs.rs: https://docs.rs/aetherdsp-midi
- [ ] Update main README if needed
- [ ] Announce in GitHub Discussions

## 🚫 Known Issues (Not Blocking)

- aether-host has failing property-based tests (not being published)
- aether-ui has dead code warnings (not being published)
- These don't affect aetherdsp-midi publication
