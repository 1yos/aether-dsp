# Security Policy

## Supported Versions

We actively support the following versions with security updates:

| Version | Supported | Status      |
| ------- | --------- | ----------- |
| 0.2.x   | ✅ Yes    | Current     |
| 0.1.x   | ✅ Yes    | Maintenance |
| < 0.1   | ❌ No     | Deprecated  |

---

## Reporting a Vulnerability

**⚠️ DO NOT open a public GitHub issue for security vulnerabilities.**

### How to Report

**Email:** security@aetherdsp.dev  
**PGP Key:** Available on request

### What to Include

Please include the following information in your report:

1. **Description** - Clear description of the vulnerability
2. **Steps to Reproduce** - Detailed steps to reproduce the issue
3. **Potential Impact** - What could an attacker achieve?
4. **Affected Versions** - Which versions are affected?
5. **Suggested Fix** - (Optional) Your proposed solution
6. **Disclosure Timeline** - Your preferred disclosure timeline

### Example Report

```
Subject: [SECURITY] Buffer overflow in parameter smoothing

Description:
A buffer overflow can occur in Param::fill_buffer() when the buffer
size exceeds BUFFER_SIZE constant.

Steps to Reproduce:
1. Create a Param with target value
2. Call fill_buffer() with buffer larger than BUFFER_SIZE
3. Observe out-of-bounds write

Potential Impact:
- Memory corruption
- Potential code execution in audio thread
- Audio glitches or crashes

Affected Versions:
- 0.1.0 through 0.1.4
- 0.2.0 through 0.2.3

Suggested Fix:
Add bounds checking in fill_buffer() or use slice length instead
of BUFFER_SIZE constant.
```

---

## Response Timeline

We take security seriously and will respond promptly:

| Stage                 | Timeline |
| --------------------- | -------- |
| **Initial Response**  | 48 hours |
| **Triage**            | 7 days   |
| **Fix Development**   | 14 days  |
| **Release**           | 21 days  |
| **Public Disclosure** | 30 days  |

**Critical vulnerabilities** (RCE, memory corruption) will be expedited:

- Initial response: 24 hours
- Fix release: 7 days
- Public disclosure: 14 days

---

## Security Considerations

### Real-Time Thread Safety

AetherDSP is designed for hard real-time audio processing. Security issues in this context have unique characteristics:

#### Potential Risks

1. **Audio Glitches from Blocking**
   - Mutex deadlocks can cause audio dropouts
   - I/O operations can block the audio thread
   - Unbounded loops can exceed deadline

2. **Memory Corruption**
   - Buffer overflows in DSP nodes
   - Use-after-free from arena generation bugs
   - Race conditions in parallel execution

3. **Denial of Service**
   - Malicious graph structures (cycles)
   - Resource exhaustion (too many nodes)
   - Command queue flooding

#### Mitigations

✅ **Lock-free data structures** - No mutexes in RT thread  
✅ **Pre-allocated buffers** - No heap allocation in RT thread  
✅ **Generational arena** - Prevents use-after-free  
✅ **Topological sort** - Rejects cyclic graphs  
✅ **Bounded command queue** - Prevents queue flooding  
✅ **Input validation** - Parameter bounds checking

---

## Safe Usage Guidelines

### For Library Users

#### ✅ DO: Safe Practices

```rust
// ✅ Pre-allocate buffers
struct SafeNode {
    buffer: Vec<f32>,
}

impl SafeNode {
    fn new() -> Self {
        Self {
            buffer: vec![0.0; BUFFER_SIZE]
        }
    }
}

impl DspNode for SafeNode {
    fn process(&mut self, ...) {
        // ✅ Reuse pre-allocated buffer
        self.buffer.fill(0.0);
    }
}
```

```rust
// ✅ Use lock-free structures
use arc_swap::ArcSwap;

struct SafeShared {
    data: Arc<ArcSwap<Vec<f32>>>,
}

impl DspNode for SafeShared {
    fn process(&mut self, ...) {
        // ✅ Lock-free read
        let data = self.data.load();
    }
}
```

```rust
// ✅ Validate inputs on control thread
fn add_node_safe(graph: &mut DspGraph, node: Box<dyn DspNode>) -> Result<NodeId, Error> {
    // ✅ Validate before adding
    if graph.node_count() >= MAX_NODES {
        return Err(Error::TooManyNodes);
    }

    graph.add_node(node)
}
```

#### ❌ DON'T: Unsafe Practices

```rust
// ❌ Allocate in process()
impl DspNode for UnsafeNode {
    fn process(&mut self, ...) {
        let buffer = vec![0.0; 1024]; // ❌ HEAP ALLOCATION!
    }
}
```

```rust
// ❌ Use Mutex in RT thread
use std::sync::Mutex;

struct UnsafeNode {
    data: Arc<Mutex<Vec<f32>>>, // ❌ CAN BLOCK!
}

impl DspNode for UnsafeNode {
    fn process(&mut self, ...) {
        let data = self.data.lock().unwrap(); // ❌ DEADLOCK RISK!
    }
}
```

```rust
// ❌ Do I/O in process()
impl DspNode for UnsafeNode {
    fn process(&mut self, ...) {
        std::fs::write("output.wav", data); // ❌ BLOCKS!
        println!("Processing..."); // ❌ BLOCKS!
    }
}
```

```rust
// ❌ Unbounded loops
impl DspNode for UnsafeNode {
    fn process(&mut self, ...) {
        while self.condition { // ❌ UNBOUNDED!
            // Could run forever
        }
    }
}
```

---

## Known Security Limitations

### 1. No Sandboxing

**Issue:** Custom DSP nodes run in the same process as the host application.

**Impact:** Malicious nodes can:

- Access host memory
- Execute arbitrary code
- Crash the application

**Mitigation:** Only load nodes from trusted sources. Future versions may add WASM sandboxing.

### 2. No Input Sanitization for Audio Data

**Issue:** Audio input data is not sanitized or validated.

**Impact:** Malicious audio input could:

- Trigger buffer overflows in poorly written nodes
- Cause NaN/Infinity propagation
- Exploit DSP algorithm vulnerabilities

**Mitigation:** Validate audio data in your nodes. Use `f32::is_finite()` checks.

### 3. Limited Resource Limits

**Issue:** No hard limits on CPU usage per node.

**Impact:** A single node can:

- Consume excessive CPU time
- Cause audio dropouts
- Starve other nodes

**Mitigation:** Monitor CPU usage. Set watchdog timers. Future versions may add per-node CPU limits.

### 4. No Memory Protection

**Issue:** Nodes can access arbitrary memory via unsafe code.

**Impact:** Malicious nodes can:

- Read sensitive data
- Corrupt memory
- Bypass Rust's safety guarantees

**Mitigation:** Audit node code before use. Avoid `unsafe` in nodes unless absolutely necessary.

---

## Security Best Practices

### For Node Developers

1. **Avoid `unsafe` code** - Use safe Rust whenever possible
2. **Validate all inputs** - Check parameter ranges, buffer sizes
3. **Handle NaN/Infinity** - Use `f32::is_finite()` checks
4. **Bound all loops** - No `while` loops, use `for` with fixed ranges
5. **Pre-allocate buffers** - No allocation in `process()`
6. **No I/O operations** - No file/network I/O in `process()`
7. **No locks** - Use lock-free structures (arc-swap, atomics)
8. **Test edge cases** - Fuzz test with property-based testing

### For Application Developers

1. **Validate graph structure** - Check for cycles, excessive nodes
2. **Limit command rate** - Rate-limit graph mutations
3. **Monitor CPU usage** - Detect runaway nodes
4. **Sandbox untrusted nodes** - (Future: WASM sandboxing)
5. **Audit dependencies** - Review node code before use
6. **Use latest versions** - Keep AetherDSP updated
7. **Enable security features** - Use all available safety checks
8. **Log security events** - Monitor for suspicious activity

---

## Security Audit History

| Date       | Auditor        | Scope         | Findings | Status   |
| ---------- | -------------- | ------------- | -------- | -------- |
| 2026-05-13 | Internal       | Core crates   | 0        | Complete |
| TBD        | External (TBD) | Full codebase | TBD      | Planned  |

**Note:** We plan to conduct a professional security audit before v1.0 release.

---

## Vulnerability Disclosure Policy

### Coordinated Disclosure

We follow responsible disclosure practices:

1. **Private Reporting** - Report vulnerabilities privately via email
2. **Acknowledgment** - We acknowledge receipt within 48 hours
3. **Investigation** - We investigate and develop a fix
4. **Coordinated Release** - We coordinate disclosure with reporter
5. **Public Disclosure** - We publish advisory after fix is released
6. **Credit** - We credit reporters (unless they prefer anonymity)

### Public Disclosure Timeline

- **Day 0:** Vulnerability reported privately
- **Day 2:** Initial response sent
- **Day 7:** Triage complete, severity assessed
- **Day 14:** Fix developed and tested
- **Day 21:** Patched version released
- **Day 30:** Public advisory published

**Exceptions:**

- Critical vulnerabilities: Expedited timeline (7-14 days)
- Already public: Immediate response
- Disputed severity: Extended discussion period

---

## Security Advisories

Security advisories are published at:

- **GitHub:** https://github.com/1yos/aether-dsp/security/advisories
- **RustSec:** https://rustsec.org/advisories/
- **Email:** security@aetherdsp.dev (subscribe for notifications)

### Advisory Format

```
AETHER-YYYY-NNNN: [Title]

Severity: [Critical|High|Medium|Low]
Affected Versions: [version range]
Fixed In: [version]
CVE: CVE-YYYY-NNNNN (if assigned)

Description:
[Detailed description]

Impact:
[What can an attacker do?]

Mitigation:
[How to protect yourself]

Credits:
[Reporter name] (if not anonymous)
```

---

## Contact

**Security Email:** security@aetherdsp.dev  
**General Issues:** https://github.com/1yos/aether-dsp/issues  
**Discussions:** https://github.com/1yos/aether-dsp/discussions

---

## Acknowledgments

We thank the following security researchers for responsibly disclosing vulnerabilities:

- (No vulnerabilities reported yet)

---

**Last Updated:** May 13, 2026  
**Version:** 1.0
