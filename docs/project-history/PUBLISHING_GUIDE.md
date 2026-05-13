# Publishing Guide - Phase 1 Improvements

## Prerequisites

Before running the publish script, ensure you have:

### 1. Crates.io Account & API Token

If you haven't already:

```powershell
# Login to crates.io (one-time setup)
cargo login

# This will prompt you to visit: https://crates.io/me
# Generate a new API token and paste it when prompted
```

Your token is stored in: `~/.cargo/credentials.toml`

### 2. Verify You're a Crate Owner

Check that you own these crates on crates.io:

- aetherdsp-core
- aetherdsp-nodes
- aetherdsp-ndk
- aetherdsp-ndk-macro
- aetherdsp-manifest
- aetherdsp-registry
- aetherdsp-midi
- aetherdsp-sampler
- aetherdsp-timbre

Visit: https://crates.io/users/[YOUR_USERNAME]

---

## What Will Be Published

### Version Bumps

| Crate               | Old Version | New Version |
| ------------------- | ----------- | ----------- |
| aetherdsp-core      | 0.1.1       | 0.1.2       |
| aetherdsp-nodes     | 0.2.1       | 0.2.2       |
| aetherdsp-ndk       | 0.1.1       | 0.1.2       |
| aetherdsp-ndk-macro | 0.1.1       | 0.1.2       |
| aetherdsp-manifest  | 0.1.1       | 0.1.2       |
| aetherdsp-registry  | 0.1.1       | 0.1.2       |
| aetherdsp-midi      | 0.1.1       | 0.1.2       |
| aetherdsp-sampler   | 0.2.0       | 0.2.1       |
| aetherdsp-timbre    | 0.1.1       | 0.1.2       |

### New Content

**CHANGELOG.md files (9 crates):**

- Complete version history from v0.1.0 to current
- Follows [Keep a Changelog](https://keepachangelog.com/) format
- Semantic versioning compliant

**Working Examples (6 new):**

1. `aether-core/examples/minimal.rs` - Simplest oscillator example
2. `aether-core/examples/graph_chain.rs` - Multi-node graph
3. `aether-core/examples/command_ring.rs` - Control → RT communication
4. `aether-ndk/examples/simple_gain.rs` - Minimal custom node
5. `aether-midi/examples/tuning_comparison.rs` - Tuning systems demo

---

## Publishing Steps

### Step 1: Dry Run (Recommended)

```powershell
.\scripts\bump_and_publish.ps1 -DryRun
```

This will show you what changes will be made without actually applying them.

### Step 2: Publish for Real

```powershell
.\scripts\bump_and_publish.ps1
```

This will:

1. Bump versions in all Cargo.toml files
2. Update CHANGELOG.md files with new version entries
3. Commit the version bump
4. Publish all 9 crates to crates.io in dependency order
5. Wait 15 seconds between each publish for indexing

**Expected Duration:** ~5-7 minutes (15s delay × 9 crates + build time)

### Step 3: Push to GitHub

```powershell
git push origin main
```

---

## After Publishing

### Immediate Verification (5 minutes)

1. **Check crates.io:**
   - https://crates.io/crates/aetherdsp-core
   - Verify version shows 0.1.2
   - Check that CHANGELOG.md appears in the file list

2. **Check docs.rs:**
   - https://docs.rs/aetherdsp-core
   - Wait 5-10 minutes for docs to build
   - Verify examples appear in the sidebar

### Announce (30 minutes)

**Reddit r/rust:**

```markdown
Title: "AetherDSP: Real-time audio engine with world music tuning - Now with better docs"

Just shipped major documentation improvements for AetherDSP, a lock-free real-time audio engine:

✅ Complete CHANGELOG files for all crates
✅ 6 new working examples (minimal synth, graph chains, tuning systems)
✅ Ethiopian, Arabic, and Indian tuning support

Key features:

- Lock-free RT scheduler (no allocations, no locks)
- Parallel BFS execution with Rayon
- 14 world music tuning systems
- Generational arena for safe node management

Check it out: https://crates.io/crates/aetherdsp-core
Examples: https://github.com/1yos/aether-dsp/tree/main/crates/aether-core/examples

Feedback welcome!
```

**Rust Users Forum:**

- Category: "Announcements"
- Similar content to Reddit post
- Link: https://users.rust-lang.org/

**Twitter/X (if applicable):**

```
Just improved docs for AetherDSP 🎵

Lock-free RT audio engine with:
✅ Parallel execution
✅ World music tuning (Ethiopian, Arabic, Indian)
✅ 6 new examples

Check it out: https://crates.io/crates/aetherdsp-core

#rustlang #audio #dsp
```

### Monitor Metrics (Weekly)

Track these metrics weekly:

1. **Crates.io downloads:**
   - https://crates.io/crates/aetherdsp-core/stats
   - Expected: 2-3× increase within 2 weeks

2. **GitHub stars:**
   - https://github.com/1yos/aether-dsp
   - Expected: +20-30 within 1 month

3. **Issues:**
   - Track "How do I...?" questions
   - Expected: 50% reduction

4. **Docs.rs page views:**
   - Check analytics if available
   - Expected: 5× increase

---

## Troubleshooting

### Error: "crate already exists"

This means the version is already published. You need to bump the version number.

**Solution:** The script handles this automatically by bumping to 0.1.2.

### Error: "authentication required"

You're not logged into crates.io.

**Solution:**

```powershell
cargo login
# Visit https://crates.io/me and generate a token
```

### Error: "not an owner of crate"

You don't have permission to publish this crate.

**Solution:** Contact the crate owner to add you as a co-owner, or skip that crate.

### Error: "failed to verify"

The crate doesn't compile or has missing dependencies.

**Solution:**

```powershell
# Test locally first
cargo build --workspace
cargo test --workspace
```

### Publishing Interrupted

If publishing fails midway, you can resume by editing the script to skip already-published crates.

---

## Expected Impact

### Week 1-2 (Immediate)

- ✅ Improved docs.rs appearance
- ✅ Better crates.io search ranking
- ✅ Reduced "How do I...?" questions

### Week 3-4 (Growth)

- 📈 **2-3× increase in downloads** (50/month → 100-150/month)
- ⭐ **+20-30 GitHub stars**
- 📚 **Better discoverability**
- 🔗 **First reverse dependencies**

### Month 2-3 (Compound)

- 📈 **4-5× total increase** (with Phase 2 API docs)
- 🌟 **50+ GitHub stars**
- 📦 **10+ reverse dependencies**
- 💬 **Active community discussions**

---

## Next Phase (Optional)

After monitoring Phase 1 results for 2 weeks, consider **Phase 2: Inline API Documentation**.

**Effort:** 2-3 days  
**Impact:** 4-5× total increase (compounds with Phase 1)

**Top 10 APIs to document:**

1. `Scheduler::new()`
2. `Scheduler::process_block()`
3. `DspGraph::add_node()`
4. `DspGraph::connect()`
5. `DspNode` trait
6. `Param::new()`
7. `Param::set_target()`
8. `Arena::insert()`
9. `TuningTable::ethiopian_tizita()`
10. `#[aether_node]` macro

See `CRATES_IO_IMPROVEMENTS.md` for full details.

---

## Questions?

- GitHub Discussions: https://github.com/1yos/aether-dsp/discussions
- Reddit: r/rust
- Rust Users Forum: https://users.rust-lang.org/

---

**Last Updated:** May 12, 2026  
**Status:** Ready to publish
