# BSM Token ID Reference & Parsing Architecture

## Complete Token ID Mapping

### Subject Process Tokens (bsm_tokens.rs)
```
Token ID  Hex   Name                   Parser Function          Extracted Fields
────────────────────────────────────────────────────────────────────────────────
0x24      0x24  SUBJECT_TOKEN_32       parse_subject_token()    pid, uid, gid, euid, egid, ruid, rgid, auid, sid, tid
0x25      0x25  SUBJECT_TOKEN_64       parse_subject_token()    pid, uid, gid, euid, egid, ruid, rgid, auid, sid, tid
0x26      0x26  SUBJECT_TOKEN_EX       parse_subject_token()    pid, uid, gid, euid, egid, ruid, rgid, auid, sid, tid (NEW)
```

**Calling pattern (all handlers):**
```rust
let subject_token = bsm_tokens::scan_tokens(&tokens, bsm_tokens::token_type::SUBJECT_TOKEN_32)
    .into_iter()
    .find_map(|t| bsm_tokens::parse_subject_token(&t))?;

let pid = subject_token.pid?;
let uid = subject_token.uid?;
```

---

### Path & Name Tokens (bsm_tokens.rs)
```
Token ID  Hex   Name               Parser Function        Extracted Fields
────────────────────────────────────────────────────────────────────────────
0x14      0x14  PATH_TOKEN         parse_path_token()     String (filesystem path)
0x29      0x29  TEXT_TOKEN         parse_text_token()     String (arbitrary text)
```

**Calling pattern:**
```rust
// File write handler extracts path
let path = bsm_tokens::scan_tokens(&tokens, bsm_tokens::token_type::PATH_TOKEN)
    .into_iter()
    .find_map(|t| bsm_tokens::parse_path_token(&t))
    .unwrap_or_else(|| "/unknown".to_string());

// Exec handler extracts cwd
let cwd = bsm_tokens::scan_tokens(&tokens, bsm_tokens::token_type::TEXT_TOKEN)
    .into_iter()
    .find_map(|t| bsm_tokens::parse_text_token(&t));
```

---

### Argument Tokens (bsm_tokens.rs)
```
Token ID  Hex   Name              Parser Function      Extracted Fields
───────────────────────────────────────────────────────────────────────
0x3c      0x3c  ARG_TOKEN         parse_arg_token()    (arg_num: u32, String)
```

**Calling pattern:**
```rust
// Exec handler extracts argv (bounded to 100 args)
let mut argv = Vec::new();
for token in bsm_tokens::scan_tokens(&tokens, bsm_tokens::token_type::ARG_TOKEN)
    .into_iter()
    .take(100)
{
    if let Some((_arg_num, arg)) = bsm_tokens::parse_arg_token(&token) {
        argv.push(arg);
    }
}
```

---

### Return Value Tokens (bsm_tokens.rs)
```
Token ID  Hex   Name               Parser Function         Extracted Fields
────────────────────────────────────────────────────────────────────────────
0x27      0x27  RETURN_TOKEN       parse_return_token()    (errval: i32, retval: i32)
```

**Format:** [token_id:1] [errval:4 BE] [retval:4 BE]

**Calling pattern (not currently used in handlers):**
```rust
if let Some((errval, retval)) = bsm_tokens::parse_return_token(&token) {
    // Handle return values
}
```

---

### Socket Address Tokens (bsm_tokens.rs)
```
Token ID  Hex   Name                   Parser Function        Extracted Fields
──────────────────────────────────────────────────────────────────────────────
0x1e      0x1e  SOCKADDR_IN_TOKEN      parse_sockaddr_ipv4()  (ip: String, port: u16)
0x1f      0x1f  SOCKADDR_IN6_TOKEN     parse_sockaddr_ipv6()  (ip: String, port: u16)
0x1d      0x1d  SOCKADDR_TOKEN         (generic - deprecated)
```

**IPv4 Format:** [token_id:1] [sa_family:2 BE] [port:2 BE] [addr:4]

**IPv6 Format:** [token_id:1] [sa_family:2 BE] [port:2 BE] [flowinfo:4] [addr:16] [scopeid:4]

**Calling pattern (network handlers):**
```rust
let (remote_ip, remote_port) = bsm_tokens::scan_tokens(&tokens, bsm_tokens::token_type::SOCKADDR_IN_TOKEN)
    .into_iter()
    .find_map(|t| bsm_tokens::parse_sockaddr_ipv4(&t))
    .or_else(|| {
        bsm_tokens::scan_tokens(&tokens, bsm_tokens::token_type::SOCKADDR_IN6_TOKEN)
            .into_iter()
            .find_map(|t| bsm_tokens::parse_sockaddr_ipv6(&t))
    })
    .unwrap_or_else(|| ("/unknown".to_string(), 0));
```

---

## Handler → Token Type Mapping

### bsm_exec.rs (AUE_EXECVE = 59)
```
Handler: handle_exec()
Tokens used:
  - SUBJECT_TOKEN_32 (0x24)  → pid, uid, euid
  - PATH_TOKEN (0x14)         → exe
  - TEXT_TOKEN (0x29)         → cwd
  - ARG_TOKEN (0x3c) × N      → argv
```

### bsm_file_ops.rs
```
Handler: handle_write() [AUE_WRITE = 2]
Tokens used:
  - SUBJECT_TOKEN_32 (0x24)  → pid, uid, euid
  - PATH_TOKEN (0x14)         → path
  - Classify via bsm_helpers::classify_path()

Handler: handle_read() [AUE_READ = 3]
Tokens used:
  - SUBJECT_TOKEN_32 (0x24)  → pid, uid
  - PATH_TOKEN (0x14)         → path

Handler: handle_create() [AUE_CREATE = 8]
Tokens used:
  - SUBJECT_TOKEN_32 (0x24)  → pid, uid, euid
  - PATH_TOKEN (0x14)         → path
```

### bsm_network.rs
```
Handler: handle_connect() [AUE_CONNECT = 43]
Tokens used:
  - SUBJECT_TOKEN_32 (0x24)  → pid, uid
  - SOCKADDR_IN_TOKEN (0x1e)  → remote_ip, remote_port (IPv4)
  - SOCKADDR_IN6_TOKEN (0x1f) → remote_ip, remote_port (IPv6)

Handler: handle_bind() [AUE_BIND = 42]
Tokens used:
  - SUBJECT_TOKEN_32 (0x24)  → pid, uid
  - SOCKADDR_IN_TOKEN (0x1e)  → local_ip, local_port (IPv4)
  - SOCKADDR_IN6_TOKEN (0x1f) → local_ip, local_port (IPv6)
```

### bsm_priv_escalation.rs
```
Handler: handle_su() [AUE_SU = 501]
Tokens used:
  - SUBJECT_TOKEN_32 (0x24)  → uid, euid, ruid

Handler: handle_sudo() [AUE_SUDO = 509]
Tokens used:
  - SUBJECT_TOKEN_32 (0x24)  → uid, euid

Handler: handle_chown() [AUE_CHOWN = 7]
Tokens used:
  - SUBJECT_TOKEN_32 (0x24)  → pid, uid, euid
```

---

## Parse Failure Categorization Flow

```
┌─ BSM Record Received ──────────────────────────┐
│                                                │
├─ SIZE CHECK                                   │
│  ├─ bytes_consumed > MAX_BYTES_PER_RECORD?   │
│  │  YES → parse_failed_malformed += 1        │
│  │  NO  → Continue                           │
│  └─ Bounds: 0 < size ≤ 16384 bytes           │
│                                               │
├─ AUE CODE DISPATCH                           │
│  ├─ Unrecognized AUE code?                   │
│  │  YES → parse_failed_unsupported += 1      │
│  │  NO  → Dispatch to handler                │
│  └─ Dispatch table in bsm_helpers.rs         │
│                                               │
├─ TOKEN PARSING (in handler)                  │
│  ├─ Missing required token type?             │
│  │  YES → parse_failed_unsupported += 1      │
│  │  NO  → Parse tokens                       │
│  │                                           │
│  └─ Field validation failure?                │
│     ├─ pid == 0 || pid > 1e7?                │
│     │  YES → parse_failed_validation += 1    │
│     ├─ uid > 1e7?                            │
│     │  YES → parse_failed_validation += 1    │
│     ├─ euid > 1e7?                           │
│     │  YES → parse_failed_validation += 1    │
│     └─ NO  → Event emitted                   │
│                                               │
└─────────────────────────────────────────────────┘
```

---

## Byte Order Convention

**All token parsing uses big-endian (network byte order)** per OpenBSM specification:

```rust
// Helper functions in bsm_tokens.rs
fn read_u32_be(data: &[u8], offset: usize) -> Option<u32> {
    Some(u32::from_be_bytes([...]))  // Big-endian
}

fn read_u16_be(data: &[u8], offset: usize) -> Option<u16> {
    Some(u16::from_be_bytes([...]))  // Big-endian
}
```

---

## Subject Token Structure (Common Fields)

All three variants (0x24, 0x25, 0x26) extract the same logical fields:

```rust
#[derive(Debug, Clone, Default)]
pub struct SubjectToken {
    pub auid: Option<u32>,      // Audit user ID
    pub uid: Option<u32>,       // User ID
    pub gid: Option<u32>,       // Group ID
    pub ruid: Option<u32>,      // Real user ID
    pub rgid: Option<u32>,      // Real group ID
    pub pid: Option<u32>,       // Process ID
    pub sid: Option<u32>,       // Session ID
    pub tid: Option<u32>,       // Terminal ID
    pub euid: Option<u32>,      // Effective user ID
    pub egid: Option<u32>,      // Effective group ID
}
```

**Validation rules:**
- pid: Must be > 0 and < 1e7 (reject 0 and absurdly large pids)
- uid/gid/euid/egid: Must be < 1e7 (0 = root is valid)

---

## Coverage Status Summary

| Token Type | Parser | Status | Used By |
|---|---|---|---|
| SUBJECT_TOKEN_32 | ✅ | Complete | All handlers |
| SUBJECT_TOKEN_64 | ✅ | Complete | All handlers |
| SUBJECT_TOKEN_EX | ✅ NEW | Complete | All handlers |
| PATH_TOKEN | ✅ | Complete | file_ops, exec |
| TEXT_TOKEN | ✅ | Complete | exec, helpers |
| ARG_TOKEN | ✅ | Complete | exec |
| RETURN_TOKEN | ✅ | Implemented | None (reserved) |
| SOCKADDR_IN_TOKEN | ✅ | Complete | network |
| SOCKADDR_IN6_TOKEN | ✅ | Complete | network |

**Missing/Not Used:** ATTR_TOKEN variants, FILE_TOKEN, SOCKADDR_TOKEN (generic)
