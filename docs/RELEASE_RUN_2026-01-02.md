[0;34m============================================================[0m
[0;34m  Release Gates - Tier 2 Verification[0m
[0;34m============================================================[0m

[1;33m[1/5] Checking compilation...[0m
warning: unused import: `super::session::*`
 --> crates/workbench/src/api.rs:6:5
  |
6 | use super::session::*;
  |     ^^^^^^^^^^^^^^^^^
  |
  = note: `#[warn(unused_imports)]` on by default

warning: `workbench` (lib) generated 1 warning (run `cargo fix --lib -p workbench` to apply 1 suggestion)
warning: unused imports: `ClaimEntry` and `TimelineEntry`
  --> crates/server/src/bundle_exchange.rs:16:21
   |
16 | use crate::report::{ClaimEntry, IntegrityNoteEntry, ReportBundle, TimelineEntry};
   |                     ^^^^^^^^^^                                    ^^^^^^^^^^^^^
   |
   = note: `#[warn(unused_imports)]` on by default

warning: unused import: `std::collections::HashMap`
  --> crates/server/src/golden_bundle.rs:26:5
   |
26 | use std::collections::HashMap;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `std::collections::BTreeMap`
  --> crates/server/src/health.rs:15:5
   |
15 | use std::collections::BTreeMap;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused variable: `path_str`
   --> crates/server/src/write_isolation.rs:275:9
    |
275 |     let path_str = path.to_string_lossy();
    |         ^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_path_str`
    |
    = note: `#[warn(unused_variables)]` on by default

warning: field `telemetry_root` is never read
  --> crates/server/src/write_isolation.rs:54:5
   |
52 | pub struct WriteIsolationContext {
   |            --------------------- field in this struct
53 |     /// Root telemetry directory
54 |     telemetry_root: PathBuf,
   |     ^^^^^^^^^^^^^^
   |
   = note: `#[warn(dead_code)]` on by default

warning: `edr-server` (lib) generated 5 warnings (run `cargo fix --lib -p edr-server` to apply 3 suggestions)
warning: unused import: `std::thread`
   --> crates/agent-macos/src/capture_macos_rotating.rs:145:5
    |
145 | use std::thread;
    |     ^^^^^^^^^^^
    |
    = note: `#[warn(unused_imports)]` on by default

warning: unused import: `std::io::Read`
  --> crates/agent-macos/src/platform/bsm_parser.rs:13:5
   |
13 | use std::io::Read;
   |     ^^^^^^^^^^^^^

warning: unused imports: `SystemTime` and `UNIX_EPOCH`
 --> crates/agent-macos/src/platform/facts.rs:4:17
  |
4 | use std::time::{SystemTime, UNIX_EPOCH};
  |                 ^^^^^^^^^^  ^^^^^^^^^^

warning: unused import: `std::collections::HashMap`
 --> crates/agent-macos/src/platform/macos_fact_extractors.rs:4:5
  |
4 | use std::collections::HashMap;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `MacOSSensorMode`
 --> crates/agent-macos/src/platform/macos_fact_extractors_es.rs:5:42
  |
5 | use super::macos_fact_extractors::{Fact, MacOSSensorMode};
  |                                          ^^^^^^^^^^^^^^^

warning: unused import: `super::bsm_tokens`
 --> crates/agent-macos/src/sensors/bsm/bsm_cred_access.rs:5:5
  |
5 | use super::bsm_tokens;
  |     ^^^^^^^^^^^^^^^^^

warning: unused import: `edr_core::Event`
 --> crates/agent-macos/src/sensors/bsm/bsm_exec.rs:7:5
  |
7 | use edr_core::Event;
  |     ^^^^^^^^^^^^^^^

warning: unused import: `EvidencePtr`
 --> crates/agent-macos/src/sensors/bsm/bsm_fs_ops.rs:6:23
  |
6 | use edr_core::{Event, EvidencePtr};
  |                       ^^^^^^^^^^^

warning: unused import: `crate::sensors::hash_keys`
 --> crates/agent-macos/src/sensors/es/file_metadata.rs:4:5
  |
4 | use crate::sensors::hash_keys;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `crate::sensors::hash_keys`
 --> crates/agent-macos/src/sensors/es/file_rename.rs:4:5
  |
4 | use crate::sensors::hash_keys;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `crate::sensors::hash_keys`
 --> crates/agent-macos/src/sensors/es/file_unlink.rs:4:5
  |
4 | use crate::sensors::hash_keys;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `std::collections::HashMap`
 --> crates/agent-macos/src/sensors/es/identity.rs:5:5
  |
5 | use std::collections::HashMap;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^

warning: variable does not need to be mutable
    --> crates/agent-macos/src/capture_macos_rotating.rs:1057:9
     |
1057 |     let mut segment_index = 0u32; // Track raw segment count for canonical indexing
     |         ----^^^^^^^^^^^^^
     |         |
     |         help: remove this `mut`
     |
     = note: `#[warn(unused_mut)]` on by default

warning: variable does not need to be mutable
    --> crates/agent-macos/src/capture_macos_rotating.rs:1059:9
     |
1059 |     let mut parse_failed_total = 0u64; // Count of malformed records
     |         ----^^^^^^^^^^^^^^^^^^
     |         |
     |         help: remove this `mut`

warning: variable does not need to be mutable
    --> crates/agent-macos/src/capture_macos_rotating.rs:1060:9
     |
1060 |     let mut parse_failed_malformed = 0u64; // Records exceeding size limit or corrupted
     |         ----^^^^^^^^^^^^^^^^^^^^^^
     |         |
     |         help: remove this `mut`

warning: variable does not need to be mutable
    --> crates/agent-macos/src/capture_macos_rotating.rs:1061:9
     |
1061 |     let mut parse_failed_unsupported = 0u64; // Unrecognized AUE codes or missing token types
     |         ----^^^^^^^^^^^^^^^^^^^^^^^^
     |         |
     |         help: remove this `mut`

warning: variable does not need to be mutable
    --> crates/agent-macos/src/capture_macos_rotating.rs:1062:9
     |
1062 |     let mut parse_failed_validation = 0u64; // Field validation failures (invalid pid, uid, etc.)
     |         ----^^^^^^^^^^^^^^^^^^^^^^^
     |         |
     |         help: remove this `mut`

warning: variable does not need to be mutable
    --> crates/agent-macos/src/capture_macos_rotating.rs:1128:35
     |
1128 |                         if let Ok(mut index) = load_or_create_index(&index_path) {
     |                                   ----^^^^^
     |                                   |
     |                                   help: remove this `mut`

warning: variable does not need to be mutable
    --> crates/agent-macos/src/capture_macos_rotating.rs:1167:21
     |
1167 |                 let mut parse_errors_batch = 0u64;
     |                     ----^^^^^^^^^^^^^^^^^^
     |                     |
     |                     help: remove this `mut`

warning: variable does not need to be mutable
    --> crates/agent-macos/src/capture_macos_rotating.rs:1304:31
     |
1304 |                     if let Ok(mut index) = load_or_create_index(&index_path) {
     |                               ----^^^^^
     |                               |
     |                               help: remove this `mut`

warning: value assigned to `path_buffer` is never read
   --> crates/agent-macos/src/platform/bsm_parser.rs:268:17
    |
268 |         let mut path_buffer = String::new();
    |                 ^^^^^^^^^^^
    |
    = help: maybe it is overwritten before being read?
    = note: `#[warn(unused_assignments)]` on by default

warning: unused variable: `segment_id`
  --> crates/agent-macos/src/sensors/bsm/bsm_archive_tool_exec.rs:23:5
   |
23 |     segment_id: &str,
   |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`
   |
   = note: `#[warn(unused_variables)]` on by default

warning: unused variable: `record_index`
  --> crates/agent-macos/src/sensors/bsm/bsm_archive_tool_exec.rs:24:5
   |
24 |     record_index: usize,
   |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
  --> crates/agent-macos/src/sensors/bsm/bsm_auth_event.rs:27:5
   |
27 |     segment_id: &str,
   |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
  --> crates/agent-macos/src/sensors/bsm/bsm_auth_event.rs:28:5
   |
28 |     record_index: usize,
   |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
  --> crates/agent-macos/src/sensors/bsm/bsm_auth_event.rs:98:5
   |
98 |     segment_id: &str,
   |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
  --> crates/agent-macos/src/sensors/bsm/bsm_auth_event.rs:99:5
   |
99 |     record_index: usize,
   |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
  --> crates/agent-macos/src/sensors/bsm/bsm_cred_access.rs:26:5
   |
26 |     segment_id: &str,
   |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
  --> crates/agent-macos/src/sensors/bsm/bsm_cred_access.rs:27:5
   |
27 |     record_index: usize,
   |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
  --> crates/agent-macos/src/sensors/bsm/bsm_defense_evasion.rs:52:5
   |
52 |     segment_id: &str,
   |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
  --> crates/agent-macos/src/sensors/bsm/bsm_defense_evasion.rs:53:5
   |
53 |     record_index: usize,
   |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
  --> crates/agent-macos/src/sensors/bsm/bsm_defense_evasion.rs:99:5
   |
99 |     segment_id: &str,
   |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
   --> crates/agent-macos/src/sensors/bsm/bsm_defense_evasion.rs:100:5
    |
100 |     record_index: usize,
    |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
  --> crates/agent-macos/src/sensors/bsm/bsm_discovery_exec.rs:23:5
   |
23 |     segment_id: &str,
   |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
  --> crates/agent-macos/src/sensors/bsm/bsm_discovery_exec.rs:24:5
   |
24 |     record_index: usize,
   |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
  --> crates/agent-macos/src/sensors/bsm/bsm_file_ops.rs:61:5
   |
61 |     segment_id: &str,
   |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
  --> crates/agent-macos/src/sensors/bsm/bsm_file_ops.rs:62:5
   |
62 |     record_index: usize,
   |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
   --> crates/agent-macos/src/sensors/bsm/bsm_file_ops.rs:116:5
    |
116 |     segment_id: &str,
    |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
   --> crates/agent-macos/src/sensors/bsm/bsm_file_ops.rs:117:5
    |
117 |     record_index: usize,
    |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
   --> crates/agent-macos/src/sensors/bsm/bsm_file_ops.rs:167:5
    |
167 |     segment_id: &str,
    |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
   --> crates/agent-macos/src/sensors/bsm/bsm_file_ops.rs:168:5
    |
168 |     record_index: usize,
    |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
   --> crates/agent-macos/src/sensors/bsm/bsm_file_ops.rs:213:5
    |
213 |     segment_id: &str,
    |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
   --> crates/agent-macos/src/sensors/bsm/bsm_file_ops.rs:214:5
    |
214 |     record_index: usize,
    |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
   --> crates/agent-macos/src/sensors/bsm/bsm_file_ops.rs:271:5
    |
271 |     segment_id: &str,
    |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
   --> crates/agent-macos/src/sensors/bsm/bsm_file_ops.rs:272:5
    |
272 |     record_index: usize,
    |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
   --> crates/agent-macos/src/sensors/bsm/bsm_file_ops.rs:327:5
    |
327 |     segment_id: &str,
    |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
   --> crates/agent-macos/src/sensors/bsm/bsm_file_ops.rs:328:5
    |
328 |     record_index: usize,
    |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
   --> crates/agent-macos/src/sensors/bsm/bsm_file_ops.rs:387:5
    |
387 |     segment_id: &str,
    |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
   --> crates/agent-macos/src/sensors/bsm/bsm_file_ops.rs:388:5
    |
388 |     record_index: usize,
    |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `exe_path`
   --> crates/agent-macos/src/sensors/bsm/bsm_file_ops.rs:474:5
    |
474 |     exe_path: Option<&str>,
    |     ^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_exe_path`

warning: unused variable: `success`
   --> crates/agent-macos/src/sensors/bsm/bsm_file_ops.rs:475:5
    |
475 |     success: bool,
    |     ^^^^^^^ help: if this is intentional, prefix it with an underscore: `_success`

warning: unused variable: `segment_id`
  --> crates/agent-macos/src/sensors/bsm/bsm_fs_ops.rs:14:5
   |
14 |     segment_id: String,
   |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
  --> crates/agent-macos/src/sensors/bsm/bsm_fs_ops.rs:15:5
   |
15 |     record_index: usize,
   |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
  --> crates/agent-macos/src/sensors/bsm/bsm_fs_ops.rs:63:5
   |
63 |     segment_id: String,
   |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
  --> crates/agent-macos/src/sensors/bsm/bsm_fs_ops.rs:64:5
   |
64 |     record_index: usize,
   |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
   --> crates/agent-macos/src/sensors/bsm/bsm_fs_ops.rs:112:5
    |
112 |     segment_id: String,
    |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
   --> crates/agent-macos/src/sensors/bsm/bsm_fs_ops.rs:113:5
    |
113 |     record_index: usize,
    |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
  --> crates/agent-macos/src/sensors/bsm/bsm_net_connect.rs:16:5
   |
16 |     segment_id: &str,
   |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
  --> crates/agent-macos/src/sensors/bsm/bsm_net_connect.rs:17:5
   |
17 |     record_index: usize,
   |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
  --> crates/agent-macos/src/sensors/bsm/bsm_network.rs:75:5
   |
75 |     segment_id: &str,
   |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
  --> crates/agent-macos/src/sensors/bsm/bsm_network.rs:76:5
   |
76 |     record_index: usize,
   |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
   --> crates/agent-macos/src/sensors/bsm/bsm_network.rs:157:5
    |
157 |     segment_id: &str,
    |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
   --> crates/agent-macos/src/sensors/bsm/bsm_network.rs:158:5
    |
158 |     record_index: usize,
    |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
   --> crates/agent-macos/src/sensors/bsm/bsm_network.rs:221:5
    |
221 |     segment_id: &str,
    |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
   --> crates/agent-macos/src/sensors/bsm/bsm_network.rs:222:5
    |
222 |     record_index: usize,
    |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `success`
   --> crates/agent-macos/src/sensors/bsm/bsm_network.rs:419:5
    |
419 |     success: bool,
    |     ^^^^^^^ help: if this is intentional, prefix it with an underscore: `_success`

warning: unused variable: `segment_id`
  --> crates/agent-macos/src/sensors/bsm/bsm_persistence_change.rs:38:5
   |
38 |     segment_id: &str,
   |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
  --> crates/agent-macos/src/sensors/bsm/bsm_persistence_change.rs:39:5
   |
39 |     record_index: usize,
   |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
  --> crates/agent-macos/src/sensors/bsm/bsm_persistence_change.rs:95:5
   |
95 |     segment_id: &str,
   |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
  --> crates/agent-macos/src/sensors/bsm/bsm_persistence_change.rs:96:5
   |
96 |     record_index: usize,
   |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
  --> crates/agent-macos/src/sensors/bsm/bsm_priv_escalation.rs:44:5
   |
44 |     segment_id: &str,
   |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
  --> crates/agent-macos/src/sensors/bsm/bsm_priv_escalation.rs:45:5
   |
45 |     record_index: usize,
   |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
   --> crates/agent-macos/src/sensors/bsm/bsm_priv_escalation.rs:124:5
    |
124 |     segment_id: &str,
    |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
   --> crates/agent-macos/src/sensors/bsm/bsm_priv_escalation.rs:125:5
    |
125 |     record_index: usize,
    |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
   --> crates/agent-macos/src/sensors/bsm/bsm_priv_escalation.rs:205:5
    |
205 |     segment_id: &str,
    |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
   --> crates/agent-macos/src/sensors/bsm/bsm_priv_escalation.rs:206:5
    |
206 |     record_index: usize,
    |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
   --> crates/agent-macos/src/sensors/bsm/bsm_priv_escalation.rs:285:5
    |
285 |     segment_id: &str,
    |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
   --> crates/agent-macos/src/sensors/bsm/bsm_priv_escalation.rs:286:5
    |
286 |     record_index: usize,
    |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
   --> crates/agent-macos/src/sensors/bsm/bsm_priv_escalation.rs:358:5
    |
358 |     segment_id: &str,
    |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
   --> crates/agent-macos/src/sensors/bsm/bsm_priv_escalation.rs:359:5
    |
359 |     record_index: usize,
    |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
   --> crates/agent-macos/src/sensors/bsm/bsm_priv_escalation.rs:448:5
    |
448 |     segment_id: &str,
    |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
   --> crates/agent-macos/src/sensors/bsm/bsm_priv_escalation.rs:449:5
    |
449 |     record_index: usize,
    |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
   --> crates/agent-macos/src/sensors/bsm/bsm_priv_escalation.rs:530:5
    |
530 |     segment_id: &str,
    |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
   --> crates/agent-macos/src/sensors/bsm/bsm_priv_escalation.rs:531:5
    |
531 |     record_index: usize,
    |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
  --> crates/agent-macos/src/sensors/bsm/bsm_process_injection.rs:35:5
   |
35 |     segment_id: &str,
   |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
  --> crates/agent-macos/src/sensors/bsm/bsm_process_injection.rs:36:5
   |
36 |     record_index: usize,
   |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
   --> crates/agent-macos/src/sensors/bsm/bsm_process_injection.rs:142:5
    |
142 |     segment_id: &str,
    |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
   --> crates/agent-macos/src/sensors/bsm/bsm_process_injection.rs:143:5
    |
143 |     record_index: usize,
    |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `host`
  --> crates/agent-macos/src/sensors/bsm/bsm_reader.rs:30:9
   |
30 |         host: &str,
   |         ^^^^ help: if this is intentional, prefix it with an underscore: `_host`

warning: unused variable: `stream_id`
  --> crates/agent-macos/src/sensors/bsm/bsm_reader.rs:31:9
   |
31 |         stream_id: &str,
   |         ^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_stream_id`

warning: unused variable: `segment_id`
  --> crates/agent-macos/src/sensors/bsm/bsm_reader.rs:32:9
   |
32 |         segment_id: &str,
   |         ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
  --> crates/agent-macos/src/sensors/bsm/bsm_reader.rs:33:9
   |
33 |         record_index: usize,
   |         ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
  --> crates/agent-macos/src/sensors/bsm/bsm_script_exec.rs:63:5
   |
63 |     segment_id: &str,
   |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
  --> crates/agent-macos/src/sensors/bsm/bsm_script_exec.rs:64:5
   |
64 |     record_index: usize,
   |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
   --> crates/agent-macos/src/sensors/bsm/bsm_script_exec.rs:201:5
    |
201 |     segment_id: &str,
    |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
   --> crates/agent-macos/src/sensors/bsm/bsm_script_exec.rs:202:5
    |
202 |     record_index: usize,
    |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
  --> crates/agent-macos/src/sensors/bsm/bsm_staging_write.rs:19:5
   |
19 |     segment_id: &str,
   |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
  --> crates/agent-macos/src/sensors/bsm/bsm_staging_write.rs:20:5
   |
20 |     record_index: usize,
   |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `host`
  --> crates/agent-macos/src/sensors/bsm/mod.rs:30:5
   |
30 |     host: String,
   |     ^^^^ help: if this is intentional, prefix it with an underscore: `_host`

warning: unused variable: `stream_id`
  --> crates/agent-macos/src/sensors/bsm/mod.rs:31:5
   |
31 |     stream_id: String,
   |     ^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_stream_id`

warning: unused variable: `segment_id`
  --> crates/agent-macos/src/sensors/bsm/mod.rs:32:5
   |
32 |     segment_id: String,
   |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
  --> crates/agent-macos/src/sensors/bsm/mod.rs:33:5
   |
33 |     record_index: usize,
   |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `record_data`
  --> crates/agent-macos/src/sensors/bsm/mod.rs:34:5
   |
34 |     record_data: Vec<u8>,
   |     ^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_data`

warning: unused variable: `ts_millis`
  --> crates/agent-macos/src/sensors/bsm/mod.rs:35:5
   |
35 |     ts_millis: u64,
   |     ^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_ts_millis`

warning: unused variable: `event_data`
  --> crates/agent-macos/src/sensors/es/file_create.rs:14:5
   |
14 |     event_data: Vec<u8>,
   |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_event_data`

warning: unused variable: `event_data`
  --> crates/agent-macos/src/sensors/es/file_metadata.rs:14:5
   |
14 |     event_data: Vec<u8>,
   |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_event_data`

warning: unused variable: `event_data`
  --> crates/agent-macos/src/sensors/es/file_open.rs:14:5
   |
14 |     event_data: Vec<u8>,
   |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_event_data`

warning: unused variable: `event_data`
  --> crates/agent-macos/src/sensors/es/file_rename.rs:14:5
   |
14 |     event_data: Vec<u8>,
   |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_event_data`

warning: unused variable: `event_data`
  --> crates/agent-macos/src/sensors/es/file_unlink.rs:14:5
   |
14 |     event_data: Vec<u8>,
   |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_event_data`

warning: unused variable: `event_data`
  --> crates/agent-macos/src/sensors/es/file_write.rs:14:5
   |
14 |     event_data: Vec<u8>,
   |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_event_data`

warning: unused variable: `es_event_data`
  --> crates/agent-macos/src/sensors/es/identity.rs:63:5
   |
63 |     es_event_data: &[u8],
   |     ^^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_es_event_data`

warning: unused variable: `stream_id`
  --> crates/agent-macos/src/sensors/es/identity.rs:64:5
   |
64 |     stream_id: String,
   |     ^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_stream_id`

warning: struct `BSMParser` is never constructed
  --> crates/agent-macos/src/capture_macos_rotating.rs:10:8
   |
10 | struct BSMParser {
   |        ^^^^^^^^^
   |
   = note: `#[warn(dead_code)]` on by default

warning: constant `BSM_HEADER_TOKEN` is never used
  --> crates/agent-macos/src/capture_macos_rotating.rs:15:7
   |
15 | const BSM_HEADER_TOKEN: u8 = 0x14; // AUT_HEADER32
   |       ^^^^^^^^^^^^^^^^

warning: constant `BSM_HEADER64_TOKEN` is never used
  --> crates/agent-macos/src/capture_macos_rotating.rs:16:7
   |
16 | const BSM_HEADER64_TOKEN: u8 = 0x74; // AUT_HEADER64
   |       ^^^^^^^^^^^^^^^^^^

warning: constant `BSM_TRAILER_TOKEN` is never used
  --> crates/agent-macos/src/capture_macos_rotating.rs:17:7
   |
17 | const BSM_TRAILER_TOKEN: u8 = 0x13; // AUT_TRAILER
   |       ^^^^^^^^^^^^^^^^^

warning: associated items `new`, `feed`, and `try_parse_record` are never used
  --> crates/agent-macos/src/capture_macos_rotating.rs:20:8
   |
19 | impl BSMParser {
   | -------------- associated items in this implementation
20 |     fn new() -> Self {
   |        ^^^
...
27 |     fn feed(&mut self, data: &[u8]) {
   |        ^^^^
...
40 |     fn try_parse_record(&mut self) -> Option<(i32, Vec<u8>, u64, usize)> {
   |        ^^^^^^^^^^^^^^^^

warning: constant `MAX_RECORDS_PER_POLL` is never used
   --> crates/agent-macos/src/capture_macos_rotating.rs:150:7
    |
150 | const MAX_RECORDS_PER_POLL: usize = 1000; // Max records to read in one poll
    |       ^^^^^^^^^^^^^^^^^^^^

warning: constant `MAX_BYTES_PER_RECORD` is never used
   --> crates/agent-macos/src/capture_macos_rotating.rs:151:7
    |
151 | const MAX_BYTES_PER_RECORD: usize = 16384; // Max 16KB per record
    |       ^^^^^^^^^^^^^^^^^^^^

warning: constant `MAX_BATCH_BYTES` is never used
   --> crates/agent-macos/src/capture_macos_rotating.rs:152:7
    |
152 | const MAX_BATCH_BYTES: usize = 10 * 1024 * 1024; // Max 10MB per batch
    |       ^^^^^^^^^^^^^^^

warning: function `get_telemetry_root` is never used
   --> crates/agent-macos/src/capture_macos_rotating.rs:154:4
    |
154 | fn get_telemetry_root() -> Result<PathBuf, String> {
    |    ^^^^^^^^^^^^^^^^^^

warning: function `get_max_segment_bytes` is never used
   --> crates/agent-macos/src/capture_macos_rotating.rs:174:4
    |
174 | fn get_max_segment_bytes() -> u64 {
    |    ^^^^^^^^^^^^^^^^^^^^^

warning: function `hostname` is never used
   --> crates/agent-macos/src/capture_macos_rotating.rs:242:4
    |
242 | fn hostname() -> String {
    |    ^^^^^^^^

warning: function `apply_evidence_ptr_ownership` is never used
   --> crates/agent-macos/src/capture_macos_rotating.rs:252:4
    |
252 | fn apply_evidence_ptr_ownership(
    |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `event_to_normalized` is never used
   --> crates/agent-macos/src/capture_macos_rotating.rs:274:4
    |
274 | fn event_to_normalized(host: &str, event: &Event) -> NormalizedEvent {
    |    ^^^^^^^^^^^^^^^^^^^

warning: function `setup_signal_handler` is never used
   --> crates/agent-macos/src/capture_macos_rotating.rs:369:4
    |
369 | fn setup_signal_handler(shutdown_flag: Arc<AtomicBool>) {
    |    ^^^^^^^^^^^^^^^^^^^^

warning: function `now_ms` is never used
   --> crates/agent-macos/src/capture_macos_rotating.rs:379:4
    |
379 | fn now_ms() -> u64 {
    |    ^^^^^^

warning: function `load_or_create_index` is never used
   --> crates/agent-macos/src/capture_macos_rotating.rs:386:4
    |
386 | fn load_or_create_index(index_path: &PathBuf) -> Result<SegmentIndex, String> {
    |    ^^^^^^^^^^^^^^^^^^^^

warning: function `validate_index` is never used
   --> crates/agent-macos/src/capture_macos_rotating.rs:404:4
    |
404 | fn validate_index(index: &SegmentIndex, segments_dir: &PathBuf) -> Result<(), String> {
    |    ^^^^^^^^^^^^^^

warning: function `write_segment_atomic` is never used
   --> crates/agent-macos/src/capture_macos_rotating.rs:429:4
    |
429 | fn write_segment_atomic(
    |    ^^^^^^^^^^^^^^^^^^^^

warning: function `compute_sha256_file` is never used
   --> crates/agent-macos/src/capture_macos_rotating.rs:507:4
    |
507 | fn compute_sha256_file(path: &PathBuf) -> Result<String, String> {
    |    ^^^^^^^^^^^^^^^^^^^

warning: function `compute_sha256_string` is never used
   --> crates/agent-macos/src/capture_macos_rotating.rs:524:4
    |
524 | fn compute_sha256_string(data: &str) -> String {
    |    ^^^^^^^^^^^^^^^^^^^^^

warning: function `update_index_atomic` is never used
   --> crates/agent-macos/src/capture_macos_rotating.rs:530:4
    |
530 | fn update_index_atomic(
    |    ^^^^^^^^^^^^^^^^^^^

warning: function `validate_root_permissions` is never used
   --> crates/agent-macos/src/capture_macos_rotating.rs:601:4
    |
601 | fn validate_root_permissions() -> Result<(), String> {
    |    ^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `run_capture_with_mock` is never used
   --> crates/agent-macos/src/capture_macos_rotating.rs:608:4
    |
608 | fn run_capture_with_mock(fixture_path: &PathBuf) {
    |    ^^^^^^^^^^^^^^^^^^^^^

warning: function `cleanup_stale_temp_files` is never used
   --> crates/agent-macos/src/capture_macos_rotating.rs:697:4
    |
697 | fn cleanup_stale_temp_files(segments_dir: &PathBuf) {
    |    ^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `get_build_id` is never used
   --> crates/agent-macos/src/capture_macos_rotating.rs:725:4
    |
725 | fn get_build_id() -> String {
    |    ^^^^^^^^^^^^

warning: function `write_heartbeat` is never used
   --> crates/agent-macos/src/capture_macos_rotating.rs:730:4
    |
730 | fn write_heartbeat(
    |    ^^^^^^^^^^^^^^^

warning: function `macos_dedup_key` is never used
   --> crates/agent-macos/src/capture_macos_rotating.rs:805:4
    |
805 | fn macos_dedup_key(event: &edr_core::Event) -> Option<String> {
    |    ^^^^^^^^^^^^^^^

warning: function `main` is never used
   --> crates/agent-macos/src/capture_macos_rotating.rs:951:4
    |
951 | fn main() {
    |    ^^^^

warning: field `pos` is never read
   --> crates/agent-macos/src/platform/bsm_parser.rs:177:5
    |
175 | pub struct BSMParser {
    |            --------- field in this struct
176 |     buffer: Vec<u8>,
177 |     pos: usize,
    |     ^^^

warning: constant `PRIVILEGED_UIDS` is never used
  --> crates/agent-macos/src/sensors/bsm/bsm_priv_escalation.rs:26:7
   |
26 | const PRIVILEGED_UIDS: &[u32] = &[
   |       ^^^^^^^^^^^^^^^

warning: function `read_u16_le` is never used
  --> crates/agent-macos/src/sensors/bsm/bsm_tokens.rs:92:4
   |
92 | fn read_u16_le(data: &[u8], offset: usize) -> Option<u16> {
   |    ^^^^^^^^^^^

warning: function `estimate_token_len_with_data` is never used
   --> crates/agent-macos/src/sensors/bsm/bsm_tokens.rs:397:4
    |
397 | fn estimate_token_len_with_data(data: &[u8]) -> Option<usize> {
    |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `PathBuf`
 --> crates/agent-windows/src/evtlog_state.rs:9:23
  |
9 | use std::path::{Path, PathBuf};
  |                       ^^^^^^^
  |
  = note: `#[warn(unused_imports)]` on by default

warning: unused import: `DateTime`
 --> crates/agent-windows/src/sensors/sysmon_adapter.rs:4:14
  |
4 | use chrono::{DateTime, Utc};
  |              ^^^^^^^^

warning: unexpected `cfg` condition value: `windows_etw`
   --> crates/agent-windows/src/sensors/mod.rs:122:30
    |
122 |             enable_etw: cfg!(feature = "windows_etw"),
    |                              ^^^^^^^^^^^^^^^^^^^^^^^ help: remove the condition
    |
    = note: no expected values for `feature`
    = help: consider adding `windows_etw` as a feature in `Cargo.toml`
    = note: see <https://doc.rust-lang.org/nightly/rustc/check-cfg/cargo-specifics.html> for more information about checking conditional configuration
    = note: `#[warn(unexpected_cfgs)]` on by default

warning: unused variable: `host`
  --> crates/agent-windows/src/sensors/collect.rs:24:24
   |
24 | fn collect_evtx_events(host: &HostCtx) -> Vec<Event> {
   |                        ^^^^ help: if this is intentional, prefix it with an underscore: `_host`
   |
   = note: `#[warn(unused_variables)]` on by default

warning: variable does not need to be mutable
  --> crates/agent-windows/src/sensors/collect.rs:25:9
   |
25 |     let mut events = Vec::new();
   |         ----^^^^^^
   |         |
   |         help: remove this `mut`
   |
   = note: `#[warn(unused_mut)]` on by default

warning: variable does not need to be mutable
  --> crates/agent-windows/src/sensors/lateral_movement_monitor.rs:46:13
   |
46 |         let mut fields = event.fields.clone();
   |             ----^^^^^^
   |             |
   |             help: remove this `mut`

warning: variable does not need to be mutable
  --> crates/agent-windows/src/sensors/log_tamper_monitor.rs:43:13
   |
43 |         let mut fields = event.fields.clone();
   |             ----^^^^^^
   |             |
   |             help: remove this `mut`

warning: variable does not need to be mutable
  --> crates/agent-windows/src/sensors/powershell_monitor.rs:44:13
   |
44 |         let mut fields = event.fields.clone();
   |             ----^^^^^^
   |             |
   |             help: remove this `mut`

warning: variable does not need to be mutable
  --> crates/agent-windows/src/sensors/registry_monitor.rs:47:13
   |
47 |         let mut fields = event.fields.clone();
   |             ----^^^^^^
   |             |
   |             help: remove this `mut`

warning: variable does not need to be mutable
  --> crates/agent-windows/src/sensors/service_monitor.rs:43:13
   |
43 |         let mut fields = event.fields.clone();
   |             ----^^^^^^
   |             |
   |             help: remove this `mut`

warning: variable does not need to be mutable
  --> crates/agent-windows/src/sensors/task_scheduler_monitor.rs:44:13
   |
44 |         let mut fields = event.fields.clone();
   |             ----^^^^^^
   |             |
   |             help: remove this `mut`

warning: variable does not need to be mutable
  --> crates/agent-windows/src/sensors/wmi_monitor.rs:44:13
   |
44 |         let mut fields = event.fields.clone();
   |             ----^^^^^^
   |             |
   |             help: remove this `mut`

warning: fields `channels` and `dedup` are never read
   --> crates/agent-windows/src/wevt_reader.rs:137:5
    |
136 | pub struct WevtReader {
    |            ---------- fields in this struct
137 |     channels: Vec<ChannelConfig>,
    |     ^^^^^^^^
...
141 |     dedup: std::sync::Mutex<BTreeMap<String, (HashSet<u64>, VecDeque<u64>)>>,
    |     ^^^^^
    |
    = note: `#[warn(dead_code)]` on by default

warning: methods `is_duplicate`, `record_event`, `record_event_values_failure`, `record_xml_truncated`, and `record_render_failure` are never used
   --> crates/agent-windows/src/wevt_reader.rs:219:8
    |
147 | impl WevtReader {
    | --------------- methods in this implementation
...
219 |     fn is_duplicate(&self, channel: &str, record_id: u64) -> bool {
    |        ^^^^^^^^^^^^
...
242 |     fn record_event(&self, channel: &str, ts: DateTime<Utc>) {
    |        ^^^^^^^^^^^^
...
256 |     fn record_event_values_failure(&self, channel: &str) {
    |        ^^^^^^^^^^^^^^^^^^^^^^^^^^^
...
267 |     fn record_xml_truncated(&self) {
    |        ^^^^^^^^^^^^^^^^^^^^
...
273 |     fn record_render_failure(&self, _channel: &str) {
    |        ^^^^^^^^^^^^^^^^^^^^^

warning: function `poll_windows` is never used
   --> crates/agent-windows/src/wevt_reader.rs:886:12
    |
886 |     pub fn poll_windows(_reader: &mut WevtReader) -> Result<Vec<WevtRecord>, String> {
    |            ^^^^^^^^^^^^

warning: unused variable: `sensors_count`
   --> crates/agent-linux/src/capture_linux_rotating.rs:133:13
    |
133 |         let sensors_count = sensor_events.len();
    |             ^^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_sensors_count`
    |
    = note: `#[warn(unused_variables)]` on by default

warning: unused variable: `ebpf_read`
   --> crates/agent-linux/src/capture_linux_rotating.rs:383:9
    |
383 |         ebpf_read: u64,
    |         ^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_ebpf_read`

warning: unused variable: `sensors_read`
   --> crates/agent-linux/src/capture_linux_rotating.rs:384:9
    |
384 |         sensors_read: u64,
    |         ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_sensors_read`

warning: unused variable: `decode_failed`
   --> crates/agent-linux/src/capture_linux_rotating.rs:385:9
    |
385 |         decode_failed: u64,
    |         ^^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_decode_failed`

warning: unused variable: `stream_id`
  --> crates/agent-linux/src/sensors/ebpf_primitives/composite_detectors.rs:57:9
   |
57 |     let stream_id = base_event
   |         ^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_stream_id`

warning: unreachable pattern
  --> crates/agent-linux/src/sensors/ebpf_primitives/net_connect.rs:77:15
   |
76 |         53 | 5353 |  // DNS (tunneling)
   |         -- matches all the relevant values
77 |         123 | 53 |  // NTP, DNS (timing channels)
   |               ^^ no value can reach this
   |
   = note: `#[warn(unreachable_patterns)]` on by default

warning: unused variable: `argv_joined`
  --> crates/agent-linux/src/sensors/ebpf_primitives/script_exec.rs:66:9
   |
66 |     let argv_joined = argv.join(" ");
   |         ^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_argv_joined`

warning: unused variable: `sensor_name`
   --> crates/agent-linux/src/sensors_v2/config.rs:210:20
    |
210 | pub fn record_poll(sensor_name: &str) {
    |                    ^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_sensor_name`

warning: constant `EVASION_TOOLS` is never used
  --> crates/agent-linux/src/sensors/ebpf_primitives/defense_evasion.rs:44:7
   |
44 | const EVASION_TOOLS: &[&str] = &[
   |       ^^^^^^^^^^^^^
   |
   = note: `#[warn(dead_code)]` on by default

warning: constant `EPISODE_KIND_THRESHOLD` is never used
  --> crates/agent-linux/src/sensors_v2/alert_engine.rs:22:7
   |
22 | const EPISODE_KIND_THRESHOLD: usize = 3;
   |       ^^^^^^^^^^^^^^^^^^^^^^

warning: constant `EPISODE_WINDOW_MS` is never used
  --> crates/agent-linux/src/sensors_v2/alert_engine.rs:23:7
   |
23 | const EPISODE_WINDOW_MS: u64 = 600000; // 10 minutes
   |       ^^^^^^^^^^^^^^^^^

warning: function `episode_id` is never used
   --> crates/agent-linux/src/sensors_v2/alert_engine.rs:124:4
    |
124 | fn episode_id(event_keys: &[String]) -> String {
    |    ^^^^^^^^^^

warning: constant `PERSISTENCE_PATHS` is never used
  --> crates/agent-linux/src/sensors_v2/persistence_watch.rs:15:7
   |
15 | const PERSISTENCE_PATHS: &[&str] = &[
   |       ^^^^^^^^^^^^^^^^^

warning: unused imports: `ImportedBundleRecord`, `ImportedBundleStore`, and `recompute_from_bundle`
  --> crates/server/src/main.rs:22:23
   |
22 |     mark_as_imported, recompute_from_bundle, validate_bundle, ExportBundleRequest,
   |                       ^^^^^^^^^^^^^^^^^^^^^
23 |     ExportBundleResponse, ImportBundleResponse, ImportedBundleRecord, ImportedBundleStore,
   |                                                 ^^^^^^^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^^^^^^

warning: unused imports: `TelemetryThrottledSummary` and `ThrottleVisibilityState`
  --> crates/server/src/main.rs:27:36
   |
27 |     CaptureProfile, ProfileConfig, TelemetryThrottledSummary, ThrottleController, ThrottleDecision,
   |                                    ^^^^^^^^^^^^^^^^^^^^^^^^^
28 |     ThrottleVisibilityState,
   |     ^^^^^^^^^^^^^^^^^^^^^^^

warning: unused imports: `ActionDetails`, `SelfCheckResponse`, `SelfCheckVerdict`, and `StreamStats`
  --> crates/server/src/main.rs:32:5
   |
32 |     ActionDetails, DiagnosticEngine, SelfCheckResponse, SelfCheckVerdict, StreamStats,
   |     ^^^^^^^^^^^^^                    ^^^^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^^^  ^^^^^^^^^^^

warning: unused import: `ProbeResult`
  --> crates/server/src/main.rs:34:13
   |
34 | use probe::{ProbeResult, ProbeRunner, ProbeSpec};
   |             ^^^^^^^^^^^

warning: unused variable: `response`
   --> crates/server/src/main.rs:966:17
    |
966 |             let response = SupportBundleResponse {
    |                 ^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_response`
    |
    = note: `#[warn(unused_variables)]` on by default

warning: unused variable: `stream_id`
   --> crates/server/src/capture_control.rs:909:34
    |
909 |     fn maybe_emit_summary(&self, stream_id: &str, stream: &StreamState) {
    |                                  ^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_stream_id`

warning: unused variable: `stream`
   --> crates/server/src/capture_control.rs:909:51
    |
909 |     fn maybe_emit_summary(&self, stream_id: &str, stream: &StreamState) {
    |                                                   ^^^^^^ help: if this is intentional, prefix it with an underscore: `_stream`

warning: unused variable: `s`
   --> crates/server/src/diagnostics.rs:537:68
    |
537 |                 let expected_running = enabled_sensors.iter().any(|s| {
    |                                                                    ^ help: if this is intentional, prefix it with an underscore: `_s`

warning: field `start_time` is never read
  --> crates/server/src/main.rs:74:5
   |
56 | struct AppState {
   |        -------- field in this struct
...
74 |     start_time: chrono::DateTime<chrono::Utc>,
   |     ^^^^^^^^^^
   |
   = note: `#[warn(dead_code)]` on by default

warning: field `note` is never read
   --> crates/server/src/main.rs:124:5
    |
122 | struct SessionControlRequest {
    |        --------------------- field in this struct
123 |     action: String, // "start", "stop", "pause", "resume"
124 |     note: Option<String>,
    |     ^^^^
    |
    = note: `SessionControlRequest` has a derived impl for the trait `Debug`, but this is intentionally ignored during dead code analysis

warning: constant `IMPORTED_NAMESPACE` is never used
  --> crates/server/src/bundle_exchange.rs:35:11
   |
35 | pub const IMPORTED_NAMESPACE: &str = "imported_bundle";
   |           ^^^^^^^^^^^^^^^^^^

warning: field `allowed_extensions` is never read
  --> crates/server/src/bundle_exchange.rs:48:9
   |
42 | pub struct ZipSafetyPolicy {
   |            --------------- field in this struct
...
48 |     pub allowed_extensions: Vec<&'static str>,
   |         ^^^^^^^^^^^^^^^^^^
   |
   = note: `ZipSafetyPolicy` has derived impls for the traits `Clone` and `Debug`, but these are intentionally ignored during dead code analysis

warning: field `bundles` is never read
   --> crates/server/src/bundle_exchange.rs:319:5
    |
318 | pub struct ImportedBundleStore {
    |            ------------------- field in this struct
319 |     bundles: std::sync::RwLock<HashMap<String, ImportedBundleRecord>>,
    |     ^^^^^^^

warning: associated items `new`, `store`, `get`, `list`, `remove`, and `is_imported_namespace` are never used
   --> crates/server/src/bundle_exchange.rs:323:12
    |
322 | impl ImportedBundleStore {
    | ------------------------ associated items in this implementation
323 |     pub fn new() -> Self {
    |            ^^^
...
327 |     pub fn store(&self, record: ImportedBundleRecord) {
    |            ^^^^^
...
332 |     pub fn get(&self, bundle_id: &str) -> Option<ImportedBundleRecord> {
    |            ^^^
...
337 |     pub fn list(&self) -> Vec<String> {
    |            ^^^^
...
342 |     pub fn remove(&self, bundle_id: &str) -> bool {
    |            ^^^^^^
...
348 |     pub fn is_imported_namespace(namespace: &str) -> bool {
    |            ^^^^^^^^^^^^^^^^^^^^^

warning: function `export_to_json` is never used
   --> crates/server/src/bundle_exchange.rs:640:8
    |
640 | pub fn export_to_json(bundle: &IncidentBundle) -> Result<Vec<u8>, String> {
    |        ^^^^^^^^^^^^^^

warning: function `recompute_from_bundle` is never used
    --> crates/server/src/bundle_exchange.rs:1052:8
     |
1052 | pub fn recompute_from_bundle(
     |        ^^^^^^^^^^^^^^^^^^^^^

warning: method `as_str` is never used
   --> crates/server/src/capture_control.rs:343:12
    |
342 | impl DropReason {
    | --------------- method in this implementation
343 |     pub fn as_str(&self) -> &'static str {
    |            ^^^^^^

warning: methods `record_sample` and `record_queue` are never used
   --> crates/server/src/capture_control.rs:498:12
    |
477 | impl StreamCounters {
    | ------------------- methods in this implementation
...
498 |     pub fn record_sample(&self) {
    |            ^^^^^^^^^^^^^
...
502 |     pub fn record_queue(&self) {
    |            ^^^^^^^^^^^^

warning: multiple methods are never used
   --> crates/server/src/capture_control.rs:964:12
    |
592 | impl ThrottleController {
    | ----------------------- methods in this implementation
...
964 |     pub fn take_pending_summaries(&self) -> Vec<TelemetryThrottledSummary> {
    |            ^^^^^^^^^^^^^^^^^^^^^^
...
970 |     pub fn has_critical_gap(&self) -> bool {
    |            ^^^^^^^^^^^^^^^^
...
975 |     pub fn get_audit_log(&self) -> Vec<ThrottleAuditEntry> {
    |            ^^^^^^^^^^^^^
...
980 |     pub fn clear_audit_log(&self) {
    |            ^^^^^^^^^^^^^^^
...
985 |     pub fn is_sensor_enabled(&self, sensor_id: &str) -> bool {
    |            ^^^^^^^^^^^^^^^^^
...
991 |     pub fn is_collector_enabled(&self, collector_id: &str) -> bool {
    |            ^^^^^^^^^^^^^^^^^^^^
...
999 |     pub fn is_heavy_feature_enabled(&self, feature_id: &str) -> bool {
    |            ^^^^^^^^^^^^^^^^^^^^^^^^

warning: methods `validate_config_match` and `validate_config_best_effort` are never used
    --> crates/server/src/capture_control.rs:1099:12
     |
1065 | impl ThrottleController {
     | ----------------------- methods in this implementation
...
1099 |     pub fn validate_config_match(
     |            ^^^^^^^^^^^^^^^^^^^^^
...
1178 |     pub fn validate_config_best_effort(&self, snapshot: &ThrottleConfigSnapshot) -> Vec<String> {
     |            ^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: associated items `event_type`, `ordering_key`, and `tier0_warning` are never used
    --> crates/server/src/capture_control.rs:1214:12
     |
1213 | impl TelemetryThrottledSummary {
     | ------------------------------ associated items in this implementation
1214 |     pub fn event_type() -> &'static str {
     |            ^^^^^^^^^^
...
1219 |     pub fn ordering_key(&self) -> (String, i64, u32) {
     |            ^^^^^^^^^^^^
...
1228 |     pub fn tier0_warning(&self) -> Option<&'static str> {
     |            ^^^^^^^^^^^^^

warning: associated function `open_in_memory` is never used
  --> crates/server/src/db.rs:40:12
   |
30 | impl Database {
   | ------------- associated function in this implementation
...
40 |     pub fn open_in_memory() -> Result<Self, rusqlite::Error> {
   |            ^^^^^^^^^^^^^^

warning: methods `as_str` and `emoji` are never used
   --> crates/server/src/diagnostics.rs:104:12
    |
103 | impl SelfCheckVerdict {
    | --------------------- methods in this implementation
104 |     pub fn as_str(&self) -> &'static str {
    |            ^^^^^^
...
112 |     pub fn emoji(&self) -> &'static str {
    |            ^^^^^

warning: field `event_observer` is never read
   --> crates/server/src/probe.rs:194:5
    |
192 | pub struct ProbeRunner {
    |            ----------- field in this struct
193 |     /// Event sink for observing results (optional, for testing)
194 |     event_observer: Option<mpsc::Sender<ObservedEventSummary>>,
    |     ^^^^^^^^^^^^^^

warning: associated items `with_observer` and `notify_event` are never used
   --> crates/server/src/probe.rs:205:12
    |
197 | impl ProbeRunner {
    | ---------------- associated items in this implementation
...
205 |     pub fn with_observer(observer: mpsc::Sender<ObservedEventSummary>) -> Self {
    |            ^^^^^^^^^^^^^
...
552 |     pub async fn notify_event(&self, event: ObservedEventSummary) {
    |                  ^^^^^^^^^^^^

warning: function `correlate_events` is never used
   --> crates/server/src/probe.rs:570:8
    |
570 | pub fn correlate_events(
    |        ^^^^^^^^^^^^^^^^

warning: function `correlate_events_robust` is never used
   --> crates/server/src/probe.rs:606:8
    |
606 | pub fn correlate_events_robust(
    |        ^^^^^^^^^^^^^^^^^^^^^^^

warning: struct `EventCorrelationDetail` is never constructed
   --> crates/server/src/probe.rs:786:12
    |
786 | pub struct EventCorrelationDetail {
    |            ^^^^^^^^^^^^^^^^^^^^^^
    |
    = note: `EventCorrelationDetail` has derived impls for the traits `Clone` and `Debug`, but these are intentionally ignored during dead code analysis

warning: methods `with_generated_at` and `with_time_window` are never used
   --> crates/server/src/report.rs:824:12
    |
774 | impl ReportBundleBuilder {
    | ------------------------ methods in this implementation
...
824 |     pub fn with_generated_at(mut self, ts: DateTime<Utc>) -> Self {
    |            ^^^^^^^^^^^^^^^^^
...
840 |     pub fn with_time_window(mut self, t_min: DateTime<Utc>, t_max: DateTime<Utc>) -> Self {
    |            ^^^^^^^^^^^^^^^^

warning: function `compute_sha256` is never used
   --> crates/server/src/support_bundle.rs:161:8
    |
161 | pub fn compute_sha256(data: &[u8]) -> String {
    |        ^^^^^^^^^^^^^^

warning: method `with_incident` is never used
   --> crates/server/src/support_bundle.rs:286:12
    |
247 | impl SupportBundleBuilder {
    | ------------------------- method in this implementation
...
286 |     pub fn with_incident(mut self, incident_json: String) -> Self {
    |            ^^^^^^^^^^^^^

warning: associated function `new` is never used
   --> crates/server/src/verification_pack.rs:350:12
    |
349 | impl VerificationState {
    | ---------------------- associated function in this implementation
350 |     pub fn new() -> Self {
    |            ^^^

warning: `agent-macos` (lib) generated 144 warnings (run `cargo fix --lib -p agent-macos` to apply 20 suggestions)
warning: `agent-windows` (lib) generated 15 warnings (run `cargo fix --lib -p agent-windows` to apply 10 suggestions)
warning: `agent-linux` (lib) generated 13 warnings
warning: `edr-server` (bin "edr-server") generated 33 warnings (1 duplicate) (run `cargo fix --bin "edr-server"` to apply 4 suggestions)
warning: unused import: `Path`
 --> crates/locald/src/baseline/baseline_store.rs:7:17
  |
7 | use std::path::{Path, PathBuf};
  |                 ^^^^
  |
  = note: `#[warn(unused_imports)]` on by default

warning: unused imports: `DMatrix` and `DVector`
 --> crates/locald/src/scoring/mahalanobis.rs:4:16
  |
4 | use nalgebra::{DMatrix, DVector};
  |                ^^^^^^^  ^^^^^^^

warning: unused import: `std::path::Path`
 --> crates/locald/src/signal_persistence.rs:7:5
  |
7 | use std::path::Path;
  |     ^^^^^^^^^^^^^^^

warning: unused import: `StoredRecord`
  --> crates/locald/src/evidence/deref.rs:12:58
   |
12 | use super::evidence_store::{EvidenceStore, RecordSource, StoredRecord};
   |                                                          ^^^^^^^^^^^^

warning: unused imports: `SeekFrom` and `Seek`
  --> crates/locald/src/evidence/deref.rs:16:35
   |
16 | use std::io::{BufRead, BufReader, Seek, SeekFrom};
   |                                   ^^^^  ^^^^^^^^

warning: unused import: `EvidenceIntegrityError`
 --> crates/locald/src/evidence/evidence_store.rs:8:27
  |
8 | use super::evidence_ptr::{EvidenceIntegrityError, EvidencePtr};
  |                           ^^^^^^^^^^^^^^^^^^^^^^

warning: unused imports: `AnalystAction`, `AssertionType`, `Assertion`, `FocusWindow`, and `Session`
  --> crates/locald/src/../../../locald/hypothesis/api.rs:13:22
   |
13 | use super::session::{AnalystAction, Assertion, AssertionType, FocusWindow, Session, SessionMode};
   |                      ^^^^^^^^^^^^^  ^^^^^^^^^  ^^^^^^^^^^^^^  ^^^^^^^^^^^  ^^^^^^^

warning: unused import: `calculate_maturity`
 --> crates/locald/src/../../../locald/hypothesis/arbitration.rs:7:46
  |
7 | use super::promotion::{calculate_confidence, calculate_maturity, Severity};
  |                                              ^^^^^^^^^^^^^^^^^^

warning: unused import: `HashMap`
  --> crates/locald/src/../../../locald/hypothesis/arbitration.rs:11:24
   |
11 | use std::collections::{HashMap, HashSet};
   |                        ^^^^^^^

warning: unused import: `super::arbitration::ArbitrationResponse`
 --> crates/locald/src/../../../locald/hypothesis/copilot.rs:7:5
  |
7 | use super::arbitration::ArbitrationResponse;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused imports: `ExplanationVisibilityState`, `MissingEvidence`, `QueryContext`, and `SlotStatusSummary`
  --> crates/locald/src/../../../locald/hypothesis/copilot.rs:11:49
   |
11 |     Claim, ClaimCertainty, ExplanationResponse, ExplanationVisibilityState, MissingEvidence,
   |                                                 ^^^^^^^^^^^^^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^^
12 |     MissingReason, QueryContext, SlotStatusSummary,
   |                    ^^^^^^^^^^^^  ^^^^^^^^^^^^^^^^^

warning: unused import: `super::hypothesis_state::HypothesisStatus`
  --> crates/locald/src/../../../locald/hypothesis/copilot.rs:14:5
   |
14 | use super::hypothesis_state::HypothesisStatus;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `super::session::SessionMode`
  --> crates/locald/src/../../../locald/hypothesis/copilot.rs:16:5
   |
16 | use super::session::SessionMode;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `super::incident::Incident`
 --> crates/locald/src/../../../locald/hypothesis/explanation.rs:9:5
  |
9 | use super::incident::Incident;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `Severity`
  --> crates/locald/src/../../../locald/hypothesis/explanation.rs:11:53
   |
11 | use super::promotion::{ConfidenceSeverityBreakdown, Severity};
   |                                                     ^^^^^^^^

warning: unused import: `EvidencePtr`
 --> crates/locald/src/../../../locald/hypothesis/storage.rs:5:46
  |
5 | use super::canonical_event::{CanonicalEvent, EvidencePtr};
  |                                              ^^^^^^^^^^^

warning: unused imports: `Deserialize` and `Serialize`
  --> crates/locald/src/../../../locald/hypothesis/storage.rs:11:13
   |
11 | use serde::{Deserialize, Serialize};
   |             ^^^^^^^^^^^  ^^^^^^^^^

warning: unused import: `Duration`
 --> crates/locald/src/../../../locald/hypothesis/changes.rs:6:24
  |
6 | use chrono::{DateTime, Duration, Utc};
  |                        ^^^^^^^^

warning: unused import: `HashSet`
 --> crates/locald/src/../../../locald/hypothesis/changes.rs:8:33
  |
8 | use std::collections::{HashMap, HashSet};
  |                                 ^^^^^^^

warning: unused import: `Duration`
 --> crates/locald/src/../../../locald/hypothesis/visibility.rs:7:24
  |
7 | use chrono::{DateTime, Duration, Utc};
  |                        ^^^^^^^^

warning: unused import: `ScopeKey`
  --> crates/locald/src/hypothesis_controller.rs:13:38
   |
13 |     LateArrivalPolicy, QueryContext, ScopeKey, SessionMode, StreamWatermark,
   |                                      ^^^^^^^^

warning: unused import: `DateTime`
  --> crates/locald/src/hypothesis_controller.rs:15:14
   |
15 | use chrono::{DateTime, Utc};
   |              ^^^^^^^^

warning: unused variable: `event`
  --> crates/locald/src/baseline/baseline_update.rs:89:54
   |
89 |     pub fn update_rates(baseline: &mut HostBaseline, event: &Event, metric_name: &str, rate: f64) {
   |                                                      ^^^^^ help: if this is intentional, prefix it with an underscore: `_event`
   |
   = note: `#[warn(unused_variables)]` on by default

warning: unused variable: `our_unique`
   --> crates/locald/src/../../../locald/hypothesis/disambiguator.rs:406:13
    |
406 |         let our_unique: Vec<_> = our_slots.difference(&their_slots).collect();
    |             ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_our_unique`

warning: unused variable: `fill_count`
   --> crates/locald/src/../../../locald/hypothesis/hypothesis_state.rs:581:29
    |
581 |         let (should_update, fill_count, fill_first_ts, fill_last_ts, time_ok) = {
    |                             ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_fill_count`

warning: unused variable: `fill_last_ts`
   --> crates/locald/src/../../../locald/hypothesis/hypothesis_state.rs:627:9
    |
627 |         fill_last_ts: DateTime<Utc>,
    |         ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_fill_last_ts`

warning: unused variable: `host_id`
   --> crates/locald/src/../../../locald/hypothesis/hypothesis_state.rs:841:9
    |
841 |         host_id: &str,
    |         ^^^^^^^ help: if this is intentional, prefix it with an underscore: `_host_id`

warning: associated items `from_results`, `resolved_contents`, `missing_ptrs`, and `has_integrity_errors` are never used
   --> crates/locald/src/evidence/deref.rs:270:12
    |
269 | impl BulkDerefResult {
    | -------------------- associated items in this implementation
270 |     pub fn from_results(results: Vec<DerefResult>) -> Self {
    |            ^^^^^^^^^^^^
...
317 |     pub fn resolved_contents(&self) -> Vec<(&EvidencePtr, &str)> {
    |            ^^^^^^^^^^^^^^^^^
...
326 |     pub fn missing_ptrs(&self) -> Vec<&EvidencePtr> {
    |            ^^^^^^^^^^^^
...
340 |     pub fn has_integrity_errors(&self) -> bool {
    |            ^^^^^^^^^^^^^^^^^^^^
    |
    = note: `#[warn(dead_code)]` on by default

warning: struct `EvidencePtrBuilder` is never constructed
   --> crates/locald/src/evidence/evidence_ptr.rs:214:12
    |
214 | pub struct EvidencePtrBuilder {
    |            ^^^^^^^^^^^^^^^^^^

warning: associated items `new`, `with_sha256`, `with_sha256_from_bytes`, `with_ts`, and `build` are never used
   --> crates/locald/src/evidence/evidence_ptr.rs:223:12
    |
222 | impl EvidencePtrBuilder {
    | ----------------------- associated items in this implementation
223 |     pub fn new(
    |            ^^^
...
237 |     pub fn with_sha256(mut self, sha256: impl Into<String>) -> Self {
    |            ^^^^^^^^^^^
...
242 |     pub fn with_sha256_from_bytes(mut self, bytes: &[u8]) -> Self {
    |            ^^^^^^^^^^^^^^^^^^^^^^
...
247 |     pub fn with_ts(mut self, ts: i64) -> Self {
    |            ^^^^^^^
...
252 |     pub fn build(self) -> EvidencePtr {
    |            ^^^^^

warning: field `db_path` is never read
   --> crates/locald/src/../../../locald/hypothesis/canonical_event.rs:509:5
    |
507 | pub struct EvidenceDerefEngine {
    |            ------------------- field in this struct
508 |     /// Database connection for canonical table lookups
509 |     db_path: String,
    |     ^^^^^^^

warning: field `citation_counter` is never read
   --> crates/locald/src/../../../locald/hypothesis/copilot.rs:416:5
    |
414 | pub struct TemplatedRenderer {
    |            ----------------- field in this struct
415 |     /// Citation counter for generating tokens
416 |     citation_counter: usize,
    |     ^^^^^^^^^^^^^^^^

warning: method `next_citation` is never used
   --> crates/locald/src/../../../locald/hypothesis/copilot.rs:430:8
    |
421 | impl TemplatedRenderer {
    | ---------------------- method in this implementation
...
430 |     fn next_citation(&mut self, claim: &Claim) -> String {
    |        ^^^^^^^^^^^^^

warning: method `check_ordering_constraints` is never used
   --> crates/locald/src/../../../locald/hypothesis/hypothesis_state.rs:649:8
    |
447 | impl HypothesisState {
    | -------------------- method in this implementation
...
649 |     fn check_ordering_constraints(&self, slot: &Slot, fill: &SlotFill) -> bool {
    |        ^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: field `track_non_incident` is never read
   --> crates/locald/src/../../../locald/hypothesis/changes.rs:251:5
    |
247 | pub struct DiffEngine {
    |            ---------- field in this struct
...
251 |     track_non_incident: bool,
    |     ^^^^^^^^^^^^^^^^^^

warning: `edr-locald` (lib) generated 35 warnings (run `cargo fix --lib -p edr-locald` to apply 22 suggestions)
warning: fields `id` and `tx` are never read
   --> crates/integration/src/realtime_sync.rs:100:5
    |
99  | struct ClientSession {
    |        ------------- fields in this struct
100 |     id: String,
    |     ^^
101 |     subscription: Subscription,
102 |     tx: broadcast::Sender<SyncMessage>,
    |     ^^
    |
    = note: `#[warn(dead_code)]` on by default

warning: `integration` (lib) generated 1 warning
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.61s
[0;32m✓ Gate 1: Compilation OK[0m

[1;33m[2/5] Running credibility lock tests...[0m
warning: unused import: `super::session::*`
 --> crates/workbench/src/api.rs:6:5
  |
6 | use super::session::*;
  |     ^^^^^^^^^^^^^^^^^
  |
  = note: `#[warn(unused_imports)]` on by default

warning: `workbench` (lib) generated 1 warning (run `cargo fix --lib -p workbench` to apply 1 suggestion)
warning: unused imports: `ClaimEntry` and `TimelineEntry`
  --> crates/server/src/bundle_exchange.rs:16:21
   |
16 | use crate::report::{ClaimEntry, IntegrityNoteEntry, ReportBundle, TimelineEntry};
   |                     ^^^^^^^^^^                                    ^^^^^^^^^^^^^
   |
   = note: `#[warn(unused_imports)]` on by default

warning: unused import: `std::collections::HashMap`
  --> crates/server/src/golden_bundle.rs:26:5
   |
26 | use std::collections::HashMap;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `std::collections::BTreeMap`
  --> crates/server/src/health.rs:15:5
   |
15 | use std::collections::BTreeMap;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused variable: `path_str`
   --> crates/server/src/write_isolation.rs:275:9
    |
275 |     let path_str = path.to_string_lossy();
    |         ^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_path_str`
    |
    = note: `#[warn(unused_variables)]` on by default

warning: field `telemetry_root` is never read
  --> crates/server/src/write_isolation.rs:54:5
   |
52 | pub struct WriteIsolationContext {
   |            --------------------- field in this struct
53 |     /// Root telemetry directory
54 |     telemetry_root: PathBuf,
   |     ^^^^^^^^^^^^^^
   |
   = note: `#[warn(dead_code)]` on by default

warning: `edr-server` (lib) generated 5 warnings (run `cargo fix --lib -p edr-server` to apply 3 suggestions)
warning: unused imports: `ImportedBundleRecord`, `ImportedBundleStore`, and `recompute_from_bundle`
  --> crates/server/src/main.rs:22:23
   |
22 |     mark_as_imported, recompute_from_bundle, validate_bundle, ExportBundleRequest,
   |                       ^^^^^^^^^^^^^^^^^^^^^
23 |     ExportBundleResponse, ImportBundleResponse, ImportedBundleRecord, ImportedBundleStore,
   |                                                 ^^^^^^^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^^^^^^

warning: unused imports: `TelemetryThrottledSummary` and `ThrottleVisibilityState`
  --> crates/server/src/main.rs:27:36
   |
27 |     CaptureProfile, ProfileConfig, TelemetryThrottledSummary, ThrottleController, ThrottleDecision,
   |                                    ^^^^^^^^^^^^^^^^^^^^^^^^^
28 |     ThrottleVisibilityState,
   |     ^^^^^^^^^^^^^^^^^^^^^^^

warning: unused imports: `ActionDetails`, `SelfCheckResponse`, `SelfCheckVerdict`, and `StreamStats`
  --> crates/server/src/main.rs:32:5
   |
32 |     ActionDetails, DiagnosticEngine, SelfCheckResponse, SelfCheckVerdict, StreamStats,
   |     ^^^^^^^^^^^^^                    ^^^^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^^^  ^^^^^^^^^^^

warning: unused import: `ProbeResult`
  --> crates/server/src/main.rs:34:13
   |
34 | use probe::{ProbeResult, ProbeRunner, ProbeSpec};
   |             ^^^^^^^^^^^

warning: unused variable: `response`
   --> crates/server/src/main.rs:966:17
    |
966 |             let response = SupportBundleResponse {
    |                 ^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_response`
    |
    = note: `#[warn(unused_variables)]` on by default

warning: unused variable: `stream_id`
   --> crates/server/src/capture_control.rs:909:34
    |
909 |     fn maybe_emit_summary(&self, stream_id: &str, stream: &StreamState) {
    |                                  ^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_stream_id`

warning: unused variable: `stream`
   --> crates/server/src/capture_control.rs:909:51
    |
909 |     fn maybe_emit_summary(&self, stream_id: &str, stream: &StreamState) {
    |                                                   ^^^^^^ help: if this is intentional, prefix it with an underscore: `_stream`

warning: unused variable: `s`
   --> crates/server/src/diagnostics.rs:537:68
    |
537 |                 let expected_running = enabled_sensors.iter().any(|s| {
    |                                                                    ^ help: if this is intentional, prefix it with an underscore: `_s`

warning: field `start_time` is never read
  --> crates/server/src/main.rs:74:5
   |
56 | struct AppState {
   |        -------- field in this struct
...
74 |     start_time: chrono::DateTime<chrono::Utc>,
   |     ^^^^^^^^^^
   |
   = note: `#[warn(dead_code)]` on by default

warning: field `note` is never read
   --> crates/server/src/main.rs:124:5
    |
122 | struct SessionControlRequest {
    |        --------------------- field in this struct
123 |     action: String, // "start", "stop", "pause", "resume"
124 |     note: Option<String>,
    |     ^^^^
    |
    = note: `SessionControlRequest` has a derived impl for the trait `Debug`, but this is intentionally ignored during dead code analysis

warning: constant `IMPORTED_NAMESPACE` is never used
  --> crates/server/src/bundle_exchange.rs:35:11
   |
35 | pub const IMPORTED_NAMESPACE: &str = "imported_bundle";
   |           ^^^^^^^^^^^^^^^^^^

warning: field `allowed_extensions` is never read
  --> crates/server/src/bundle_exchange.rs:48:9
   |
42 | pub struct ZipSafetyPolicy {
   |            --------------- field in this struct
...
48 |     pub allowed_extensions: Vec<&'static str>,
   |         ^^^^^^^^^^^^^^^^^^
   |
   = note: `ZipSafetyPolicy` has derived impls for the traits `Clone` and `Debug`, but these are intentionally ignored during dead code analysis

warning: field `bundles` is never read
   --> crates/server/src/bundle_exchange.rs:319:5
    |
318 | pub struct ImportedBundleStore {
    |            ------------------- field in this struct
319 |     bundles: std::sync::RwLock<HashMap<String, ImportedBundleRecord>>,
    |     ^^^^^^^

warning: associated items `new`, `store`, `get`, `list`, `remove`, and `is_imported_namespace` are never used
   --> crates/server/src/bundle_exchange.rs:323:12
    |
322 | impl ImportedBundleStore {
    | ------------------------ associated items in this implementation
323 |     pub fn new() -> Self {
    |            ^^^
...
327 |     pub fn store(&self, record: ImportedBundleRecord) {
    |            ^^^^^
...
332 |     pub fn get(&self, bundle_id: &str) -> Option<ImportedBundleRecord> {
    |            ^^^
...
337 |     pub fn list(&self) -> Vec<String> {
    |            ^^^^
...
342 |     pub fn remove(&self, bundle_id: &str) -> bool {
    |            ^^^^^^
...
348 |     pub fn is_imported_namespace(namespace: &str) -> bool {
    |            ^^^^^^^^^^^^^^^^^^^^^

warning: function `export_to_json` is never used
   --> crates/server/src/bundle_exchange.rs:640:8
    |
640 | pub fn export_to_json(bundle: &IncidentBundle) -> Result<Vec<u8>, String> {
    |        ^^^^^^^^^^^^^^

warning: function `recompute_from_bundle` is never used
    --> crates/server/src/bundle_exchange.rs:1052:8
     |
1052 | pub fn recompute_from_bundle(
     |        ^^^^^^^^^^^^^^^^^^^^^

warning: method `as_str` is never used
   --> crates/server/src/capture_control.rs:343:12
    |
342 | impl DropReason {
    | --------------- method in this implementation
343 |     pub fn as_str(&self) -> &'static str {
    |            ^^^^^^

warning: methods `record_sample` and `record_queue` are never used
   --> crates/server/src/capture_control.rs:498:12
    |
477 | impl StreamCounters {
    | ------------------- methods in this implementation
...
498 |     pub fn record_sample(&self) {
    |            ^^^^^^^^^^^^^
...
502 |     pub fn record_queue(&self) {
    |            ^^^^^^^^^^^^

warning: multiple methods are never used
   --> crates/server/src/capture_control.rs:964:12
    |
592 | impl ThrottleController {
    | ----------------------- methods in this implementation
...
964 |     pub fn take_pending_summaries(&self) -> Vec<TelemetryThrottledSummary> {
    |            ^^^^^^^^^^^^^^^^^^^^^^
...
970 |     pub fn has_critical_gap(&self) -> bool {
    |            ^^^^^^^^^^^^^^^^
...
975 |     pub fn get_audit_log(&self) -> Vec<ThrottleAuditEntry> {
    |            ^^^^^^^^^^^^^
...
980 |     pub fn clear_audit_log(&self) {
    |            ^^^^^^^^^^^^^^^
...
985 |     pub fn is_sensor_enabled(&self, sensor_id: &str) -> bool {
    |            ^^^^^^^^^^^^^^^^^
...
991 |     pub fn is_collector_enabled(&self, collector_id: &str) -> bool {
    |            ^^^^^^^^^^^^^^^^^^^^
...
999 |     pub fn is_heavy_feature_enabled(&self, feature_id: &str) -> bool {
    |            ^^^^^^^^^^^^^^^^^^^^^^^^

warning: methods `validate_config_match` and `validate_config_best_effort` are never used
    --> crates/server/src/capture_control.rs:1099:12
     |
1065 | impl ThrottleController {
     | ----------------------- methods in this implementation
...
1099 |     pub fn validate_config_match(
     |            ^^^^^^^^^^^^^^^^^^^^^
...
1178 |     pub fn validate_config_best_effort(&self, snapshot: &ThrottleConfigSnapshot) -> Vec<String> {
     |            ^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: associated items `event_type`, `ordering_key`, and `tier0_warning` are never used
    --> crates/server/src/capture_control.rs:1214:12
     |
1213 | impl TelemetryThrottledSummary {
     | ------------------------------ associated items in this implementation
1214 |     pub fn event_type() -> &'static str {
     |            ^^^^^^^^^^
...
1219 |     pub fn ordering_key(&self) -> (String, i64, u32) {
     |            ^^^^^^^^^^^^
...
1228 |     pub fn tier0_warning(&self) -> Option<&'static str> {
     |            ^^^^^^^^^^^^^

warning: associated function `open_in_memory` is never used
  --> crates/server/src/db.rs:40:12
   |
30 | impl Database {
   | ------------- associated function in this implementation
...
40 |     pub fn open_in_memory() -> Result<Self, rusqlite::Error> {
   |            ^^^^^^^^^^^^^^

warning: methods `as_str` and `emoji` are never used
   --> crates/server/src/diagnostics.rs:104:12
    |
103 | impl SelfCheckVerdict {
    | --------------------- methods in this implementation
104 |     pub fn as_str(&self) -> &'static str {
    |            ^^^^^^
...
112 |     pub fn emoji(&self) -> &'static str {
    |            ^^^^^

warning: field `event_observer` is never read
   --> crates/server/src/probe.rs:194:5
    |
192 | pub struct ProbeRunner {
    |            ----------- field in this struct
193 |     /// Event sink for observing results (optional, for testing)
194 |     event_observer: Option<mpsc::Sender<ObservedEventSummary>>,
    |     ^^^^^^^^^^^^^^

warning: associated items `with_observer` and `notify_event` are never used
   --> crates/server/src/probe.rs:205:12
    |
197 | impl ProbeRunner {
    | ---------------- associated items in this implementation
...
205 |     pub fn with_observer(observer: mpsc::Sender<ObservedEventSummary>) -> Self {
    |            ^^^^^^^^^^^^^
...
552 |     pub async fn notify_event(&self, event: ObservedEventSummary) {
    |                  ^^^^^^^^^^^^

warning: function `correlate_events` is never used
   --> crates/server/src/probe.rs:570:8
    |
570 | pub fn correlate_events(
    |        ^^^^^^^^^^^^^^^^

warning: function `correlate_events_robust` is never used
   --> crates/server/src/probe.rs:606:8
    |
606 | pub fn correlate_events_robust(
    |        ^^^^^^^^^^^^^^^^^^^^^^^

warning: struct `EventCorrelationDetail` is never constructed
   --> crates/server/src/probe.rs:786:12
    |
786 | pub struct EventCorrelationDetail {
    |            ^^^^^^^^^^^^^^^^^^^^^^
    |
    = note: `EventCorrelationDetail` has derived impls for the traits `Clone` and `Debug`, but these are intentionally ignored during dead code analysis

warning: methods `with_generated_at` and `with_time_window` are never used
   --> crates/server/src/report.rs:824:12
    |
774 | impl ReportBundleBuilder {
    | ------------------------ methods in this implementation
...
824 |     pub fn with_generated_at(mut self, ts: DateTime<Utc>) -> Self {
    |            ^^^^^^^^^^^^^^^^^
...
840 |     pub fn with_time_window(mut self, t_min: DateTime<Utc>, t_max: DateTime<Utc>) -> Self {
    |            ^^^^^^^^^^^^^^^^

warning: function `compute_sha256` is never used
   --> crates/server/src/support_bundle.rs:161:8
    |
161 | pub fn compute_sha256(data: &[u8]) -> String {
    |        ^^^^^^^^^^^^^^

warning: method `with_incident` is never used
   --> crates/server/src/support_bundle.rs:286:12
    |
247 | impl SupportBundleBuilder {
    | ------------------------- method in this implementation
...
286 |     pub fn with_incident(mut self, incident_json: String) -> Self {
    |            ^^^^^^^^^^^^^

warning: associated function `new` is never used
   --> crates/server/src/verification_pack.rs:350:12
    |
349 | impl VerificationState {
    | ---------------------- associated function in this implementation
350 |     pub fn new() -> Self {
    |            ^^^

warning: `edr-server` (bin "edr-server") generated 33 warnings (1 duplicate) (run `cargo fix --bin "edr-server"` to apply 4 suggestions)
warning: unused import: `std::collections::HashMap`
 --> crates/server/tests/credibility_lock_tests.rs:9:5
  |
9 | use std::collections::HashMap;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^
  |
  = note: `#[warn(unused_imports)]` on by default

warning: unused import: `tempfile::TempDir`
  --> crates/server/tests/credibility_lock_tests.rs:11:5
   |
11 | use tempfile::TempDir;
   |     ^^^^^^^^^^^^^^^^^

warning: field `stream_id` is never read
   --> crates/server/tests/credibility_lock_tests.rs:200:5
    |
199 | struct StreamDiagnostic {
    |        ---------------- field in this struct
200 |     stream_id: String,
    |     ^^^^^^^^^
    |
    = note: `StreamDiagnostic` has derived impls for the traits `Clone` and `Debug`, but these are intentionally ignored during dead code analysis
    = note: `#[warn(dead_code)]` on by default

warning: variants `Info` and `Warning` are never constructed
   --> crates/server/tests/credibility_lock_tests.rs:207:5
    |
206 | enum IssueSeverity {
    |      ------------- variants in this enum
207 |     Info,
    |     ^^^^
208 |     Warning,
    |     ^^^^^^^
    |
    = note: `IssueSeverity` has derived impls for the traits `Clone` and `Debug`, but these are intentionally ignored during dead code analysis

warning: `edr-server` (test "credibility_lock_tests") generated 4 warnings (run `cargo fix --test "credibility_lock_tests"` to apply 2 suggestions)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.52s
     Running tests/credibility_lock_tests.rs (target/debug/deps/credibility_lock_tests-2851a1cb031d4116)

running 22 tests
test test_e2e_broken_storage_blocked_verdict ... ok
test test_path_safety_blocks_empty ... ok
test test_path_safety_blocks_absolute_injection ... ok
test test_namespace_isolation_filters_imported_by_default ... ok
test test_path_safety_errors_are_safe ... ok
test test_e2e_imported_bundle_isolation ... ok
test test_namespace_isolation_blocks_direct_imported_access ... ok
test test_path_safety_blocks_traversal ... ok
test test_e2e_evidence_deref_traversal_blocked ... ok
test test_namespace_isolation_error_message_is_actionable ... ok
test test_path_safety_allows_valid_paths ... ok
test test_selfcheck_blocked_on_critical_issue ... ok
test test_selfcheck_blocked_on_db_failure ... ok
test test_selfcheck_blocked_on_no_critical_events ... ok
test test_selfcheck_blocked_on_segment_dir_failure ... ok
test test_selfcheck_degraded_on_error_issue ... ok
test test_selfcheck_healthy_when_all_ok ... ok
test test_redaction_macos_home_paths ... ok
test test_redaction_multiple_paths ... ok
test test_redaction_preserves_system_paths ... ok
test test_redaction_windows_paths ... ok
test test_redaction_linux_paths ... ok

test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s

[0;32m✓ Gate 2: Credibility locks verified (22 tests)[0m

[1;33m[3/5] Running core unit tests...[0m
warning: unused import: `super::session::*`
 --> crates/workbench/src/api.rs:6:5
  |
6 | use super::session::*;
  |     ^^^^^^^^^^^^^^^^^
  |
  = note: `#[warn(unused_imports)]` on by default

warning: `workbench` (lib) generated 1 warning (run `cargo fix --lib -p workbench` to apply 1 suggestion)
warning: unused imports: `ClaimEntry` and `TimelineEntry`
  --> crates/server/src/bundle_exchange.rs:16:21
   |
16 | use crate::report::{ClaimEntry, IntegrityNoteEntry, ReportBundle, TimelineEntry};
   |                     ^^^^^^^^^^                                    ^^^^^^^^^^^^^
   |
   = note: `#[warn(unused_imports)]` on by default

warning: unused import: `std::collections::HashMap`
  --> crates/server/src/golden_bundle.rs:26:5
   |
26 | use std::collections::HashMap;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `std::collections::BTreeMap`
  --> crates/server/src/health.rs:15:5
   |
15 | use std::collections::BTreeMap;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused variable: `path_str`
   --> crates/server/src/write_isolation.rs:275:9
    |
275 |     let path_str = path.to_string_lossy();
    |         ^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_path_str`
    |
    = note: `#[warn(unused_variables)]` on by default

warning: field `telemetry_root` is never read
  --> crates/server/src/write_isolation.rs:54:5
   |
52 | pub struct WriteIsolationContext {
   |            --------------------- field in this struct
53 |     /// Root telemetry directory
54 |     telemetry_root: PathBuf,
   |     ^^^^^^^^^^^^^^
   |
   = note: `#[warn(dead_code)]` on by default

warning: `edr-server` (lib) generated 5 warnings (run `cargo fix --lib -p edr-server` to apply 3 suggestions)
warning: unused import: `Path`
 --> crates/locald/src/baseline/baseline_store.rs:7:17
  |
7 | use std::path::{Path, PathBuf};
  |                 ^^^^
  |
  = note: `#[warn(unused_imports)]` on by default

warning: unused imports: `DMatrix` and `DVector`
 --> crates/locald/src/scoring/mahalanobis.rs:4:16
  |
4 | use nalgebra::{DMatrix, DVector};
  |                ^^^^^^^  ^^^^^^^

warning: unused import: `std::path::Path`
 --> crates/locald/src/signal_persistence.rs:7:5
  |
7 | use std::path::Path;
  |     ^^^^^^^^^^^^^^^

warning: unused import: `StoredRecord`
  --> crates/locald/src/evidence/deref.rs:12:58
   |
12 | use super::evidence_store::{EvidenceStore, RecordSource, StoredRecord};
   |                                                          ^^^^^^^^^^^^

warning: unused imports: `SeekFrom` and `Seek`
  --> crates/locald/src/evidence/deref.rs:16:35
   |
16 | use std::io::{BufRead, BufReader, Seek, SeekFrom};
   |                                   ^^^^  ^^^^^^^^

warning: unused import: `EvidenceIntegrityError`
 --> crates/locald/src/evidence/evidence_store.rs:8:27
  |
8 | use super::evidence_ptr::{EvidenceIntegrityError, EvidencePtr};
  |                           ^^^^^^^^^^^^^^^^^^^^^^

warning: unused imports: `AnalystAction`, `AssertionType`, `Assertion`, `FocusWindow`, and `Session`
  --> crates/locald/src/../../../locald/hypothesis/api.rs:13:22
   |
13 | use super::session::{AnalystAction, Assertion, AssertionType, FocusWindow, Session, SessionMode};
   |                      ^^^^^^^^^^^^^  ^^^^^^^^^  ^^^^^^^^^^^^^  ^^^^^^^^^^^  ^^^^^^^

warning: unused import: `calculate_maturity`
 --> crates/locald/src/../../../locald/hypothesis/arbitration.rs:7:46
  |
7 | use super::promotion::{calculate_confidence, calculate_maturity, Severity};
  |                                              ^^^^^^^^^^^^^^^^^^

warning: unused import: `HashMap`
  --> crates/locald/src/../../../locald/hypothesis/arbitration.rs:11:24
   |
11 | use std::collections::{HashMap, HashSet};
   |                        ^^^^^^^

warning: unused import: `super::arbitration::ArbitrationResponse`
 --> crates/locald/src/../../../locald/hypothesis/copilot.rs:7:5
  |
7 | use super::arbitration::ArbitrationResponse;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused imports: `ExplanationVisibilityState`, `MissingEvidence`, `QueryContext`, and `SlotStatusSummary`
  --> crates/locald/src/../../../locald/hypothesis/copilot.rs:11:49
   |
11 |     Claim, ClaimCertainty, ExplanationResponse, ExplanationVisibilityState, MissingEvidence,
   |                                                 ^^^^^^^^^^^^^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^^
12 |     MissingReason, QueryContext, SlotStatusSummary,
   |                    ^^^^^^^^^^^^  ^^^^^^^^^^^^^^^^^

warning: unused import: `super::hypothesis_state::HypothesisStatus`
  --> crates/locald/src/../../../locald/hypothesis/copilot.rs:14:5
   |
14 | use super::hypothesis_state::HypothesisStatus;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `super::session::SessionMode`
  --> crates/locald/src/../../../locald/hypothesis/copilot.rs:16:5
   |
16 | use super::session::SessionMode;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `super::incident::Incident`
 --> crates/locald/src/../../../locald/hypothesis/explanation.rs:9:5
  |
9 | use super::incident::Incident;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `Severity`
  --> crates/locald/src/../../../locald/hypothesis/explanation.rs:11:53
   |
11 | use super::promotion::{ConfidenceSeverityBreakdown, Severity};
   |                                                     ^^^^^^^^

warning: unused import: `EvidencePtr`
 --> crates/locald/src/../../../locald/hypothesis/storage.rs:5:46
  |
5 | use super::canonical_event::{CanonicalEvent, EvidencePtr};
  |                                              ^^^^^^^^^^^

warning: unused imports: `Deserialize` and `Serialize`
  --> crates/locald/src/../../../locald/hypothesis/storage.rs:11:13
   |
11 | use serde::{Deserialize, Serialize};
   |             ^^^^^^^^^^^  ^^^^^^^^^

warning: unused import: `Duration`
 --> crates/locald/src/../../../locald/hypothesis/changes.rs:6:24
  |
6 | use chrono::{DateTime, Duration, Utc};
  |                        ^^^^^^^^

warning: unused import: `HashSet`
 --> crates/locald/src/../../../locald/hypothesis/changes.rs:8:33
  |
8 | use std::collections::{HashMap, HashSet};
  |                                 ^^^^^^^

warning: unused import: `Duration`
 --> crates/locald/src/../../../locald/hypothesis/visibility.rs:7:24
  |
7 | use chrono::{DateTime, Duration, Utc};
  |                        ^^^^^^^^

warning: unused import: `ScopeKey`
  --> crates/locald/src/hypothesis_controller.rs:13:38
   |
13 |     LateArrivalPolicy, QueryContext, ScopeKey, SessionMode, StreamWatermark,
   |                                      ^^^^^^^^

warning: unused import: `DateTime`
  --> crates/locald/src/hypothesis_controller.rs:15:14
   |
15 | use chrono::{DateTime, Utc};
   |              ^^^^^^^^

warning: unused variable: `event`
  --> crates/locald/src/baseline/baseline_update.rs:89:54
   |
89 |     pub fn update_rates(baseline: &mut HostBaseline, event: &Event, metric_name: &str, rate: f64) {
   |                                                      ^^^^^ help: if this is intentional, prefix it with an underscore: `_event`
   |
   = note: `#[warn(unused_variables)]` on by default

warning: unused variable: `our_unique`
   --> crates/locald/src/../../../locald/hypothesis/disambiguator.rs:406:13
    |
406 |         let our_unique: Vec<_> = our_slots.difference(&their_slots).collect();
    |             ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_our_unique`

warning: unused variable: `fill_count`
   --> crates/locald/src/../../../locald/hypothesis/hypothesis_state.rs:581:29
    |
581 |         let (should_update, fill_count, fill_first_ts, fill_last_ts, time_ok) = {
    |                             ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_fill_count`

warning: unused variable: `fill_last_ts`
   --> crates/locald/src/../../../locald/hypothesis/hypothesis_state.rs:627:9
    |
627 |         fill_last_ts: DateTime<Utc>,
    |         ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_fill_last_ts`

warning: unused variable: `host_id`
   --> crates/locald/src/../../../locald/hypothesis/hypothesis_state.rs:841:9
    |
841 |         host_id: &str,
    |         ^^^^^^^ help: if this is intentional, prefix it with an underscore: `_host_id`

warning: associated items `from_results`, `resolved_contents`, `missing_ptrs`, and `has_integrity_errors` are never used
   --> crates/locald/src/evidence/deref.rs:270:12
    |
269 | impl BulkDerefResult {
    | -------------------- associated items in this implementation
270 |     pub fn from_results(results: Vec<DerefResult>) -> Self {
    |            ^^^^^^^^^^^^
...
317 |     pub fn resolved_contents(&self) -> Vec<(&EvidencePtr, &str)> {
    |            ^^^^^^^^^^^^^^^^^
...
326 |     pub fn missing_ptrs(&self) -> Vec<&EvidencePtr> {
    |            ^^^^^^^^^^^^
...
340 |     pub fn has_integrity_errors(&self) -> bool {
    |            ^^^^^^^^^^^^^^^^^^^^
    |
    = note: `#[warn(dead_code)]` on by default

warning: struct `EvidencePtrBuilder` is never constructed
   --> crates/locald/src/evidence/evidence_ptr.rs:214:12
    |
214 | pub struct EvidencePtrBuilder {
    |            ^^^^^^^^^^^^^^^^^^

warning: associated items `new`, `with_sha256`, `with_sha256_from_bytes`, `with_ts`, and `build` are never used
   --> crates/locald/src/evidence/evidence_ptr.rs:223:12
    |
222 | impl EvidencePtrBuilder {
    | ----------------------- associated items in this implementation
223 |     pub fn new(
    |            ^^^
...
237 |     pub fn with_sha256(mut self, sha256: impl Into<String>) -> Self {
    |            ^^^^^^^^^^^
...
242 |     pub fn with_sha256_from_bytes(mut self, bytes: &[u8]) -> Self {
    |            ^^^^^^^^^^^^^^^^^^^^^^
...
247 |     pub fn with_ts(mut self, ts: i64) -> Self {
    |            ^^^^^^^
...
252 |     pub fn build(self) -> EvidencePtr {
    |            ^^^^^

warning: field `db_path` is never read
   --> crates/locald/src/../../../locald/hypothesis/canonical_event.rs:509:5
    |
507 | pub struct EvidenceDerefEngine {
    |            ------------------- field in this struct
508 |     /// Database connection for canonical table lookups
509 |     db_path: String,
    |     ^^^^^^^

warning: field `citation_counter` is never read
   --> crates/locald/src/../../../locald/hypothesis/copilot.rs:416:5
    |
414 | pub struct TemplatedRenderer {
    |            ----------------- field in this struct
415 |     /// Citation counter for generating tokens
416 |     citation_counter: usize,
    |     ^^^^^^^^^^^^^^^^

warning: method `next_citation` is never used
   --> crates/locald/src/../../../locald/hypothesis/copilot.rs:430:8
    |
421 | impl TemplatedRenderer {
    | ---------------------- method in this implementation
...
430 |     fn next_citation(&mut self, claim: &Claim) -> String {
    |        ^^^^^^^^^^^^^

warning: method `check_ordering_constraints` is never used
   --> crates/locald/src/../../../locald/hypothesis/hypothesis_state.rs:649:8
    |
447 | impl HypothesisState {
    | -------------------- method in this implementation
...
649 |     fn check_ordering_constraints(&self, slot: &Slot, fill: &SlotFill) -> bool {
    |        ^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: field `track_non_incident` is never read
   --> crates/locald/src/../../../locald/hypothesis/changes.rs:251:5
    |
247 | pub struct DiffEngine {
    |            ---------- field in this struct
...
251 |     track_non_incident: bool,
    |     ^^^^^^^^^^^^^^^^^^

warning: `edr-locald` (lib) generated 35 warnings (run `cargo fix --lib -p edr-locald` to apply 22 suggestions)
warning: unused variable: `sensors_count`
   --> crates/agent-linux/src/capture_linux_rotating.rs:133:13
    |
133 |         let sensors_count = sensor_events.len();
    |             ^^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_sensors_count`
    |
    = note: `#[warn(unused_variables)]` on by default

warning: unused variable: `ebpf_read`
   --> crates/agent-linux/src/capture_linux_rotating.rs:383:9
    |
383 |         ebpf_read: u64,
    |         ^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_ebpf_read`

warning: unused variable: `sensors_read`
   --> crates/agent-linux/src/capture_linux_rotating.rs:384:9
    |
384 |         sensors_read: u64,
    |         ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_sensors_read`

warning: unused variable: `decode_failed`
   --> crates/agent-linux/src/capture_linux_rotating.rs:385:9
    |
385 |         decode_failed: u64,
    |         ^^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_decode_failed`

warning: unused variable: `stream_id`
  --> crates/agent-linux/src/sensors/ebpf_primitives/composite_detectors.rs:57:9
   |
57 |     let stream_id = base_event
   |         ^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_stream_id`

warning: unreachable pattern
  --> crates/agent-linux/src/sensors/ebpf_primitives/net_connect.rs:77:15
   |
76 |         53 | 5353 |  // DNS (tunneling)
   |         -- matches all the relevant values
77 |         123 | 53 |  // NTP, DNS (timing channels)
   |               ^^ no value can reach this
   |
   = note: `#[warn(unreachable_patterns)]` on by default

warning: unused variable: `argv_joined`
  --> crates/agent-linux/src/sensors/ebpf_primitives/script_exec.rs:66:9
   |
66 |     let argv_joined = argv.join(" ");
   |         ^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_argv_joined`

warning: unused variable: `sensor_name`
   --> crates/agent-linux/src/sensors_v2/config.rs:210:20
    |
210 | pub fn record_poll(sensor_name: &str) {
    |                    ^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_sensor_name`

warning: constant `EVASION_TOOLS` is never used
  --> crates/agent-linux/src/sensors/ebpf_primitives/defense_evasion.rs:44:7
   |
44 | const EVASION_TOOLS: &[&str] = &[
   |       ^^^^^^^^^^^^^
   |
   = note: `#[warn(dead_code)]` on by default

warning: constant `EPISODE_KIND_THRESHOLD` is never used
  --> crates/agent-linux/src/sensors_v2/alert_engine.rs:22:7
   |
22 | const EPISODE_KIND_THRESHOLD: usize = 3;
   |       ^^^^^^^^^^^^^^^^^^^^^^

warning: constant `EPISODE_WINDOW_MS` is never used
  --> crates/agent-linux/src/sensors_v2/alert_engine.rs:23:7
   |
23 | const EPISODE_WINDOW_MS: u64 = 600000; // 10 minutes
   |       ^^^^^^^^^^^^^^^^^

warning: function `episode_id` is never used
   --> crates/agent-linux/src/sensors_v2/alert_engine.rs:124:4
    |
124 | fn episode_id(event_keys: &[String]) -> String {
    |    ^^^^^^^^^^

warning: constant `PERSISTENCE_PATHS` is never used
  --> crates/agent-linux/src/sensors_v2/persistence_watch.rs:15:7
   |
15 | const PERSISTENCE_PATHS: &[&str] = &[
   |       ^^^^^^^^^^^^^^^^^

warning: unused import: `PathBuf`
 --> crates/agent-windows/src/evtlog_state.rs:9:23
  |
9 | use std::path::{Path, PathBuf};
  |                       ^^^^^^^
  |
  = note: `#[warn(unused_imports)]` on by default

warning: unused import: `DateTime`
 --> crates/agent-windows/src/sensors/sysmon_adapter.rs:4:14
  |
4 | use chrono::{DateTime, Utc};
  |              ^^^^^^^^

warning: unexpected `cfg` condition value: `windows_etw`
   --> crates/agent-windows/src/sensors/mod.rs:122:30
    |
122 |             enable_etw: cfg!(feature = "windows_etw"),
    |                              ^^^^^^^^^^^^^^^^^^^^^^^ help: remove the condition
    |
    = note: no expected values for `feature`
    = help: consider adding `windows_etw` as a feature in `Cargo.toml`
    = note: see <https://doc.rust-lang.org/nightly/rustc/check-cfg/cargo-specifics.html> for more information about checking conditional configuration
    = note: `#[warn(unexpected_cfgs)]` on by default

warning: unused variable: `host`
  --> crates/agent-windows/src/sensors/collect.rs:24:24
   |
24 | fn collect_evtx_events(host: &HostCtx) -> Vec<Event> {
   |                        ^^^^ help: if this is intentional, prefix it with an underscore: `_host`
   |
   = note: `#[warn(unused_variables)]` on by default

warning: variable does not need to be mutable
  --> crates/agent-windows/src/sensors/collect.rs:25:9
   |
25 |     let mut events = Vec::new();
   |         ----^^^^^^
   |         |
   |         help: remove this `mut`
   |
   = note: `#[warn(unused_mut)]` on by default

warning: variable does not need to be mutable
  --> crates/agent-windows/src/sensors/lateral_movement_monitor.rs:46:13
   |
46 |         let mut fields = event.fields.clone();
   |             ----^^^^^^
   |             |
   |             help: remove this `mut`

warning: variable does not need to be mutable
  --> crates/agent-windows/src/sensors/log_tamper_monitor.rs:43:13
   |
43 |         let mut fields = event.fields.clone();
   |             ----^^^^^^
   |             |
   |             help: remove this `mut`

warning: variable does not need to be mutable
  --> crates/agent-windows/src/sensors/powershell_monitor.rs:44:13
   |
44 |         let mut fields = event.fields.clone();
   |             ----^^^^^^
   |             |
   |             help: remove this `mut`

warning: variable does not need to be mutable
  --> crates/agent-windows/src/sensors/registry_monitor.rs:47:13
   |
47 |         let mut fields = event.fields.clone();
   |             ----^^^^^^
   |             |
   |             help: remove this `mut`

warning: variable does not need to be mutable
  --> crates/agent-windows/src/sensors/service_monitor.rs:43:13
   |
43 |         let mut fields = event.fields.clone();
   |             ----^^^^^^
   |             |
   |             help: remove this `mut`

warning: variable does not need to be mutable
  --> crates/agent-windows/src/sensors/task_scheduler_monitor.rs:44:13
   |
44 |         let mut fields = event.fields.clone();
   |             ----^^^^^^
   |             |
   |             help: remove this `mut`

warning: variable does not need to be mutable
  --> crates/agent-windows/src/sensors/wmi_monitor.rs:44:13
   |
44 |         let mut fields = event.fields.clone();
   |             ----^^^^^^
   |             |
   |             help: remove this `mut`

warning: fields `channels` and `dedup` are never read
   --> crates/agent-windows/src/wevt_reader.rs:137:5
    |
136 | pub struct WevtReader {
    |            ---------- fields in this struct
137 |     channels: Vec<ChannelConfig>,
    |     ^^^^^^^^
...
141 |     dedup: std::sync::Mutex<BTreeMap<String, (HashSet<u64>, VecDeque<u64>)>>,
    |     ^^^^^
    |
    = note: `#[warn(dead_code)]` on by default

warning: methods `is_duplicate`, `record_event`, `record_event_values_failure`, `record_xml_truncated`, and `record_render_failure` are never used
   --> crates/agent-windows/src/wevt_reader.rs:219:8
    |
147 | impl WevtReader {
    | --------------- methods in this implementation
...
219 |     fn is_duplicate(&self, channel: &str, record_id: u64) -> bool {
    |        ^^^^^^^^^^^^
...
242 |     fn record_event(&self, channel: &str, ts: DateTime<Utc>) {
    |        ^^^^^^^^^^^^
...
256 |     fn record_event_values_failure(&self, channel: &str) {
    |        ^^^^^^^^^^^^^^^^^^^^^^^^^^^
...
267 |     fn record_xml_truncated(&self) {
    |        ^^^^^^^^^^^^^^^^^^^^
...
273 |     fn record_render_failure(&self, _channel: &str) {
    |        ^^^^^^^^^^^^^^^^^^^^^

warning: function `poll_windows` is never used
   --> crates/agent-windows/src/wevt_reader.rs:886:12
    |
886 |     pub fn poll_windows(_reader: &mut WevtReader) -> Result<Vec<WevtRecord>, String> {
    |            ^^^^^^^^^^^^

warning: `agent-linux` (lib) generated 13 warnings
warning: `agent-windows` (lib) generated 15 warnings (run `cargo fix --lib -p agent-windows` to apply 10 suggestions)
warning: unused import: `std::thread`
   --> crates/agent-macos/src/capture_macos_rotating.rs:145:5
    |
145 | use std::thread;
    |     ^^^^^^^^^^^
    |
    = note: `#[warn(unused_imports)]` on by default

warning: unused import: `std::io::Read`
  --> crates/agent-macos/src/platform/bsm_parser.rs:13:5
   |
13 | use std::io::Read;
   |     ^^^^^^^^^^^^^

warning: unused imports: `SystemTime` and `UNIX_EPOCH`
 --> crates/agent-macos/src/platform/facts.rs:4:17
  |
4 | use std::time::{SystemTime, UNIX_EPOCH};
  |                 ^^^^^^^^^^  ^^^^^^^^^^

warning: unused import: `std::collections::HashMap`
 --> crates/agent-macos/src/platform/macos_fact_extractors.rs:4:5
  |
4 | use std::collections::HashMap;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `MacOSSensorMode`
 --> crates/agent-macos/src/platform/macos_fact_extractors_es.rs:5:42
  |
5 | use super::macos_fact_extractors::{Fact, MacOSSensorMode};
  |                                          ^^^^^^^^^^^^^^^

warning: unused import: `super::bsm_tokens`
 --> crates/agent-macos/src/sensors/bsm/bsm_cred_access.rs:5:5
  |
5 | use super::bsm_tokens;
  |     ^^^^^^^^^^^^^^^^^

warning: unused import: `edr_core::Event`
 --> crates/agent-macos/src/sensors/bsm/bsm_exec.rs:7:5
  |
7 | use edr_core::Event;
  |     ^^^^^^^^^^^^^^^

warning: unused import: `EvidencePtr`
 --> crates/agent-macos/src/sensors/bsm/bsm_fs_ops.rs:6:23
  |
6 | use edr_core::{Event, EvidencePtr};
  |                       ^^^^^^^^^^^

warning: unused import: `crate::sensors::hash_keys`
 --> crates/agent-macos/src/sensors/es/file_metadata.rs:4:5
  |
4 | use crate::sensors::hash_keys;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `crate::sensors::hash_keys`
 --> crates/agent-macos/src/sensors/es/file_rename.rs:4:5
  |
4 | use crate::sensors::hash_keys;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `crate::sensors::hash_keys`
 --> crates/agent-macos/src/sensors/es/file_unlink.rs:4:5
  |
4 | use crate::sensors::hash_keys;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `std::collections::HashMap`
 --> crates/agent-macos/src/sensors/es/identity.rs:5:5
  |
5 | use std::collections::HashMap;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^

warning: variable does not need to be mutable
    --> crates/agent-macos/src/capture_macos_rotating.rs:1057:9
     |
1057 |     let mut segment_index = 0u32; // Track raw segment count for canonical indexing
     |         ----^^^^^^^^^^^^^
     |         |
     |         help: remove this `mut`
     |
     = note: `#[warn(unused_mut)]` on by default

warning: variable does not need to be mutable
    --> crates/agent-macos/src/capture_macos_rotating.rs:1059:9
     |
1059 |     let mut parse_failed_total = 0u64; // Count of malformed records
     |         ----^^^^^^^^^^^^^^^^^^
     |         |
     |         help: remove this `mut`

warning: variable does not need to be mutable
    --> crates/agent-macos/src/capture_macos_rotating.rs:1060:9
     |
1060 |     let mut parse_failed_malformed = 0u64; // Records exceeding size limit or corrupted
     |         ----^^^^^^^^^^^^^^^^^^^^^^
     |         |
     |         help: remove this `mut`

warning: variable does not need to be mutable
    --> crates/agent-macos/src/capture_macos_rotating.rs:1061:9
     |
1061 |     let mut parse_failed_unsupported = 0u64; // Unrecognized AUE codes or missing token types
     |         ----^^^^^^^^^^^^^^^^^^^^^^^^
     |         |
     |         help: remove this `mut`

warning: variable does not need to be mutable
    --> crates/agent-macos/src/capture_macos_rotating.rs:1062:9
     |
1062 |     let mut parse_failed_validation = 0u64; // Field validation failures (invalid pid, uid, etc.)
     |         ----^^^^^^^^^^^^^^^^^^^^^^^
     |         |
     |         help: remove this `mut`

warning: variable does not need to be mutable
    --> crates/agent-macos/src/capture_macos_rotating.rs:1128:35
     |
1128 |                         if let Ok(mut index) = load_or_create_index(&index_path) {
     |                                   ----^^^^^
     |                                   |
     |                                   help: remove this `mut`

warning: variable does not need to be mutable
    --> crates/agent-macos/src/capture_macos_rotating.rs:1167:21
     |
1167 |                 let mut parse_errors_batch = 0u64;
     |                     ----^^^^^^^^^^^^^^^^^^
     |                     |
     |                     help: remove this `mut`

warning: variable does not need to be mutable
    --> crates/agent-macos/src/capture_macos_rotating.rs:1304:31
     |
1304 |                     if let Ok(mut index) = load_or_create_index(&index_path) {
     |                               ----^^^^^
     |                               |
     |                               help: remove this `mut`

warning: value assigned to `path_buffer` is never read
   --> crates/agent-macos/src/platform/bsm_parser.rs:268:17
    |
268 |         let mut path_buffer = String::new();
    |                 ^^^^^^^^^^^
    |
    = help: maybe it is overwritten before being read?
    = note: `#[warn(unused_assignments)]` on by default

warning: unused variable: `segment_id`
  --> crates/agent-macos/src/sensors/bsm/bsm_archive_tool_exec.rs:23:5
   |
23 |     segment_id: &str,
   |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`
   |
   = note: `#[warn(unused_variables)]` on by default

warning: unused variable: `record_index`
  --> crates/agent-macos/src/sensors/bsm/bsm_archive_tool_exec.rs:24:5
   |
24 |     record_index: usize,
   |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
  --> crates/agent-macos/src/sensors/bsm/bsm_auth_event.rs:27:5
   |
27 |     segment_id: &str,
   |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
  --> crates/agent-macos/src/sensors/bsm/bsm_auth_event.rs:28:5
   |
28 |     record_index: usize,
   |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
  --> crates/agent-macos/src/sensors/bsm/bsm_auth_event.rs:98:5
   |
98 |     segment_id: &str,
   |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
  --> crates/agent-macos/src/sensors/bsm/bsm_auth_event.rs:99:5
   |
99 |     record_index: usize,
   |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
  --> crates/agent-macos/src/sensors/bsm/bsm_cred_access.rs:26:5
   |
26 |     segment_id: &str,
   |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
  --> crates/agent-macos/src/sensors/bsm/bsm_cred_access.rs:27:5
   |
27 |     record_index: usize,
   |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
  --> crates/agent-macos/src/sensors/bsm/bsm_defense_evasion.rs:52:5
   |
52 |     segment_id: &str,
   |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
  --> crates/agent-macos/src/sensors/bsm/bsm_defense_evasion.rs:53:5
   |
53 |     record_index: usize,
   |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
  --> crates/agent-macos/src/sensors/bsm/bsm_defense_evasion.rs:99:5
   |
99 |     segment_id: &str,
   |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
   --> crates/agent-macos/src/sensors/bsm/bsm_defense_evasion.rs:100:5
    |
100 |     record_index: usize,
    |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
  --> crates/agent-macos/src/sensors/bsm/bsm_discovery_exec.rs:23:5
   |
23 |     segment_id: &str,
   |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
  --> crates/agent-macos/src/sensors/bsm/bsm_discovery_exec.rs:24:5
   |
24 |     record_index: usize,
   |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
  --> crates/agent-macos/src/sensors/bsm/bsm_file_ops.rs:61:5
   |
61 |     segment_id: &str,
   |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
  --> crates/agent-macos/src/sensors/bsm/bsm_file_ops.rs:62:5
   |
62 |     record_index: usize,
   |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
   --> crates/agent-macos/src/sensors/bsm/bsm_file_ops.rs:116:5
    |
116 |     segment_id: &str,
    |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
   --> crates/agent-macos/src/sensors/bsm/bsm_file_ops.rs:117:5
    |
117 |     record_index: usize,
    |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
   --> crates/agent-macos/src/sensors/bsm/bsm_file_ops.rs:167:5
    |
167 |     segment_id: &str,
    |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
   --> crates/agent-macos/src/sensors/bsm/bsm_file_ops.rs:168:5
    |
168 |     record_index: usize,
    |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
   --> crates/agent-macos/src/sensors/bsm/bsm_file_ops.rs:213:5
    |
213 |     segment_id: &str,
    |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
   --> crates/agent-macos/src/sensors/bsm/bsm_file_ops.rs:214:5
    |
214 |     record_index: usize,
    |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
   --> crates/agent-macos/src/sensors/bsm/bsm_file_ops.rs:271:5
    |
271 |     segment_id: &str,
    |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
   --> crates/agent-macos/src/sensors/bsm/bsm_file_ops.rs:272:5
    |
272 |     record_index: usize,
    |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
   --> crates/agent-macos/src/sensors/bsm/bsm_file_ops.rs:327:5
    |
327 |     segment_id: &str,
    |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
   --> crates/agent-macos/src/sensors/bsm/bsm_file_ops.rs:328:5
    |
328 |     record_index: usize,
    |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
   --> crates/agent-macos/src/sensors/bsm/bsm_file_ops.rs:387:5
    |
387 |     segment_id: &str,
    |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
   --> crates/agent-macos/src/sensors/bsm/bsm_file_ops.rs:388:5
    |
388 |     record_index: usize,
    |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `exe_path`
   --> crates/agent-macos/src/sensors/bsm/bsm_file_ops.rs:474:5
    |
474 |     exe_path: Option<&str>,
    |     ^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_exe_path`

warning: unused variable: `success`
   --> crates/agent-macos/src/sensors/bsm/bsm_file_ops.rs:475:5
    |
475 |     success: bool,
    |     ^^^^^^^ help: if this is intentional, prefix it with an underscore: `_success`

warning: unused variable: `segment_id`
  --> crates/agent-macos/src/sensors/bsm/bsm_fs_ops.rs:14:5
   |
14 |     segment_id: String,
   |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
  --> crates/agent-macos/src/sensors/bsm/bsm_fs_ops.rs:15:5
   |
15 |     record_index: usize,
   |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
  --> crates/agent-macos/src/sensors/bsm/bsm_fs_ops.rs:63:5
   |
63 |     segment_id: String,
   |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
  --> crates/agent-macos/src/sensors/bsm/bsm_fs_ops.rs:64:5
   |
64 |     record_index: usize,
   |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
   --> crates/agent-macos/src/sensors/bsm/bsm_fs_ops.rs:112:5
    |
112 |     segment_id: String,
    |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
   --> crates/agent-macos/src/sensors/bsm/bsm_fs_ops.rs:113:5
    |
113 |     record_index: usize,
    |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
  --> crates/agent-macos/src/sensors/bsm/bsm_net_connect.rs:16:5
   |
16 |     segment_id: &str,
   |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
  --> crates/agent-macos/src/sensors/bsm/bsm_net_connect.rs:17:5
   |
17 |     record_index: usize,
   |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
  --> crates/agent-macos/src/sensors/bsm/bsm_network.rs:75:5
   |
75 |     segment_id: &str,
   |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
  --> crates/agent-macos/src/sensors/bsm/bsm_network.rs:76:5
   |
76 |     record_index: usize,
   |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
   --> crates/agent-macos/src/sensors/bsm/bsm_network.rs:157:5
    |
157 |     segment_id: &str,
    |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
   --> crates/agent-macos/src/sensors/bsm/bsm_network.rs:158:5
    |
158 |     record_index: usize,
    |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
   --> crates/agent-macos/src/sensors/bsm/bsm_network.rs:221:5
    |
221 |     segment_id: &str,
    |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
   --> crates/agent-macos/src/sensors/bsm/bsm_network.rs:222:5
    |
222 |     record_index: usize,
    |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `success`
   --> crates/agent-macos/src/sensors/bsm/bsm_network.rs:419:5
    |
419 |     success: bool,
    |     ^^^^^^^ help: if this is intentional, prefix it with an underscore: `_success`

warning: unused variable: `segment_id`
  --> crates/agent-macos/src/sensors/bsm/bsm_persistence_change.rs:38:5
   |
38 |     segment_id: &str,
   |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
  --> crates/agent-macos/src/sensors/bsm/bsm_persistence_change.rs:39:5
   |
39 |     record_index: usize,
   |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
  --> crates/agent-macos/src/sensors/bsm/bsm_persistence_change.rs:95:5
   |
95 |     segment_id: &str,
   |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
  --> crates/agent-macos/src/sensors/bsm/bsm_persistence_change.rs:96:5
   |
96 |     record_index: usize,
   |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
  --> crates/agent-macos/src/sensors/bsm/bsm_priv_escalation.rs:44:5
   |
44 |     segment_id: &str,
   |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
  --> crates/agent-macos/src/sensors/bsm/bsm_priv_escalation.rs:45:5
   |
45 |     record_index: usize,
   |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
   --> crates/agent-macos/src/sensors/bsm/bsm_priv_escalation.rs:124:5
    |
124 |     segment_id: &str,
    |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
   --> crates/agent-macos/src/sensors/bsm/bsm_priv_escalation.rs:125:5
    |
125 |     record_index: usize,
    |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
   --> crates/agent-macos/src/sensors/bsm/bsm_priv_escalation.rs:205:5
    |
205 |     segment_id: &str,
    |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
   --> crates/agent-macos/src/sensors/bsm/bsm_priv_escalation.rs:206:5
    |
206 |     record_index: usize,
    |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
   --> crates/agent-macos/src/sensors/bsm/bsm_priv_escalation.rs:285:5
    |
285 |     segment_id: &str,
    |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
   --> crates/agent-macos/src/sensors/bsm/bsm_priv_escalation.rs:286:5
    |
286 |     record_index: usize,
    |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
   --> crates/agent-macos/src/sensors/bsm/bsm_priv_escalation.rs:358:5
    |
358 |     segment_id: &str,
    |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
   --> crates/agent-macos/src/sensors/bsm/bsm_priv_escalation.rs:359:5
    |
359 |     record_index: usize,
    |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
   --> crates/agent-macos/src/sensors/bsm/bsm_priv_escalation.rs:448:5
    |
448 |     segment_id: &str,
    |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
   --> crates/agent-macos/src/sensors/bsm/bsm_priv_escalation.rs:449:5
    |
449 |     record_index: usize,
    |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
   --> crates/agent-macos/src/sensors/bsm/bsm_priv_escalation.rs:530:5
    |
530 |     segment_id: &str,
    |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
   --> crates/agent-macos/src/sensors/bsm/bsm_priv_escalation.rs:531:5
    |
531 |     record_index: usize,
    |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
  --> crates/agent-macos/src/sensors/bsm/bsm_process_injection.rs:35:5
   |
35 |     segment_id: &str,
   |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
  --> crates/agent-macos/src/sensors/bsm/bsm_process_injection.rs:36:5
   |
36 |     record_index: usize,
   |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
   --> crates/agent-macos/src/sensors/bsm/bsm_process_injection.rs:142:5
    |
142 |     segment_id: &str,
    |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
   --> crates/agent-macos/src/sensors/bsm/bsm_process_injection.rs:143:5
    |
143 |     record_index: usize,
    |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `host`
  --> crates/agent-macos/src/sensors/bsm/bsm_reader.rs:30:9
   |
30 |         host: &str,
   |         ^^^^ help: if this is intentional, prefix it with an underscore: `_host`

warning: unused variable: `stream_id`
  --> crates/agent-macos/src/sensors/bsm/bsm_reader.rs:31:9
   |
31 |         stream_id: &str,
   |         ^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_stream_id`

warning: unused variable: `segment_id`
  --> crates/agent-macos/src/sensors/bsm/bsm_reader.rs:32:9
   |
32 |         segment_id: &str,
   |         ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
  --> crates/agent-macos/src/sensors/bsm/bsm_reader.rs:33:9
   |
33 |         record_index: usize,
   |         ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
  --> crates/agent-macos/src/sensors/bsm/bsm_script_exec.rs:63:5
   |
63 |     segment_id: &str,
   |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
  --> crates/agent-macos/src/sensors/bsm/bsm_script_exec.rs:64:5
   |
64 |     record_index: usize,
   |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
   --> crates/agent-macos/src/sensors/bsm/bsm_script_exec.rs:201:5
    |
201 |     segment_id: &str,
    |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
   --> crates/agent-macos/src/sensors/bsm/bsm_script_exec.rs:202:5
    |
202 |     record_index: usize,
    |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
  --> crates/agent-macos/src/sensors/bsm/bsm_staging_write.rs:19:5
   |
19 |     segment_id: &str,
   |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
  --> crates/agent-macos/src/sensors/bsm/bsm_staging_write.rs:20:5
   |
20 |     record_index: usize,
   |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `host`
  --> crates/agent-macos/src/sensors/bsm/mod.rs:30:5
   |
30 |     host: String,
   |     ^^^^ help: if this is intentional, prefix it with an underscore: `_host`

warning: unused variable: `stream_id`
  --> crates/agent-macos/src/sensors/bsm/mod.rs:31:5
   |
31 |     stream_id: String,
   |     ^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_stream_id`

warning: unused variable: `segment_id`
  --> crates/agent-macos/src/sensors/bsm/mod.rs:32:5
   |
32 |     segment_id: String,
   |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
  --> crates/agent-macos/src/sensors/bsm/mod.rs:33:5
   |
33 |     record_index: usize,
   |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `record_data`
  --> crates/agent-macos/src/sensors/bsm/mod.rs:34:5
   |
34 |     record_data: Vec<u8>,
   |     ^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_data`

warning: unused variable: `ts_millis`
  --> crates/agent-macos/src/sensors/bsm/mod.rs:35:5
   |
35 |     ts_millis: u64,
   |     ^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_ts_millis`

warning: unused variable: `event_data`
  --> crates/agent-macos/src/sensors/es/file_create.rs:14:5
   |
14 |     event_data: Vec<u8>,
   |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_event_data`

warning: unused variable: `event_data`
  --> crates/agent-macos/src/sensors/es/file_metadata.rs:14:5
   |
14 |     event_data: Vec<u8>,
   |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_event_data`

warning: unused variable: `event_data`
  --> crates/agent-macos/src/sensors/es/file_open.rs:14:5
   |
14 |     event_data: Vec<u8>,
   |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_event_data`

warning: unused variable: `event_data`
  --> crates/agent-macos/src/sensors/es/file_rename.rs:14:5
   |
14 |     event_data: Vec<u8>,
   |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_event_data`

warning: unused variable: `event_data`
  --> crates/agent-macos/src/sensors/es/file_unlink.rs:14:5
   |
14 |     event_data: Vec<u8>,
   |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_event_data`

warning: unused variable: `event_data`
  --> crates/agent-macos/src/sensors/es/file_write.rs:14:5
   |
14 |     event_data: Vec<u8>,
   |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_event_data`

warning: unused variable: `es_event_data`
  --> crates/agent-macos/src/sensors/es/identity.rs:63:5
   |
63 |     es_event_data: &[u8],
   |     ^^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_es_event_data`

warning: unused variable: `stream_id`
  --> crates/agent-macos/src/sensors/es/identity.rs:64:5
   |
64 |     stream_id: String,
   |     ^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_stream_id`

warning: struct `BSMParser` is never constructed
  --> crates/agent-macos/src/capture_macos_rotating.rs:10:8
   |
10 | struct BSMParser {
   |        ^^^^^^^^^
   |
   = note: `#[warn(dead_code)]` on by default

warning: constant `BSM_HEADER_TOKEN` is never used
  --> crates/agent-macos/src/capture_macos_rotating.rs:15:7
   |
15 | const BSM_HEADER_TOKEN: u8 = 0x14; // AUT_HEADER32
   |       ^^^^^^^^^^^^^^^^

warning: constant `BSM_HEADER64_TOKEN` is never used
  --> crates/agent-macos/src/capture_macos_rotating.rs:16:7
   |
16 | const BSM_HEADER64_TOKEN: u8 = 0x74; // AUT_HEADER64
   |       ^^^^^^^^^^^^^^^^^^

warning: constant `BSM_TRAILER_TOKEN` is never used
  --> crates/agent-macos/src/capture_macos_rotating.rs:17:7
   |
17 | const BSM_TRAILER_TOKEN: u8 = 0x13; // AUT_TRAILER
   |       ^^^^^^^^^^^^^^^^^

warning: associated items `new`, `feed`, and `try_parse_record` are never used
  --> crates/agent-macos/src/capture_macos_rotating.rs:20:8
   |
19 | impl BSMParser {
   | -------------- associated items in this implementation
20 |     fn new() -> Self {
   |        ^^^
...
27 |     fn feed(&mut self, data: &[u8]) {
   |        ^^^^
...
40 |     fn try_parse_record(&mut self) -> Option<(i32, Vec<u8>, u64, usize)> {
   |        ^^^^^^^^^^^^^^^^

warning: constant `MAX_RECORDS_PER_POLL` is never used
   --> crates/agent-macos/src/capture_macos_rotating.rs:150:7
    |
150 | const MAX_RECORDS_PER_POLL: usize = 1000; // Max records to read in one poll
    |       ^^^^^^^^^^^^^^^^^^^^

warning: constant `MAX_BYTES_PER_RECORD` is never used
   --> crates/agent-macos/src/capture_macos_rotating.rs:151:7
    |
151 | const MAX_BYTES_PER_RECORD: usize = 16384; // Max 16KB per record
    |       ^^^^^^^^^^^^^^^^^^^^

warning: constant `MAX_BATCH_BYTES` is never used
   --> crates/agent-macos/src/capture_macos_rotating.rs:152:7
    |
152 | const MAX_BATCH_BYTES: usize = 10 * 1024 * 1024; // Max 10MB per batch
    |       ^^^^^^^^^^^^^^^

warning: function `get_telemetry_root` is never used
   --> crates/agent-macos/src/capture_macos_rotating.rs:154:4
    |
154 | fn get_telemetry_root() -> Result<PathBuf, String> {
    |    ^^^^^^^^^^^^^^^^^^

warning: function `get_max_segment_bytes` is never used
   --> crates/agent-macos/src/capture_macos_rotating.rs:174:4
    |
174 | fn get_max_segment_bytes() -> u64 {
    |    ^^^^^^^^^^^^^^^^^^^^^

warning: function `hostname` is never used
   --> crates/agent-macos/src/capture_macos_rotating.rs:242:4
    |
242 | fn hostname() -> String {
    |    ^^^^^^^^

warning: function `apply_evidence_ptr_ownership` is never used
   --> crates/agent-macos/src/capture_macos_rotating.rs:252:4
    |
252 | fn apply_evidence_ptr_ownership(
    |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `event_to_normalized` is never used
   --> crates/agent-macos/src/capture_macos_rotating.rs:274:4
    |
274 | fn event_to_normalized(host: &str, event: &Event) -> NormalizedEvent {
    |    ^^^^^^^^^^^^^^^^^^^

warning: function `setup_signal_handler` is never used
   --> crates/agent-macos/src/capture_macos_rotating.rs:369:4
    |
369 | fn setup_signal_handler(shutdown_flag: Arc<AtomicBool>) {
    |    ^^^^^^^^^^^^^^^^^^^^

warning: function `now_ms` is never used
   --> crates/agent-macos/src/capture_macos_rotating.rs:379:4
    |
379 | fn now_ms() -> u64 {
    |    ^^^^^^

warning: function `load_or_create_index` is never used
   --> crates/agent-macos/src/capture_macos_rotating.rs:386:4
    |
386 | fn load_or_create_index(index_path: &PathBuf) -> Result<SegmentIndex, String> {
    |    ^^^^^^^^^^^^^^^^^^^^

warning: function `validate_index` is never used
   --> crates/agent-macos/src/capture_macos_rotating.rs:404:4
    |
404 | fn validate_index(index: &SegmentIndex, segments_dir: &PathBuf) -> Result<(), String> {
    |    ^^^^^^^^^^^^^^

warning: function `write_segment_atomic` is never used
   --> crates/agent-macos/src/capture_macos_rotating.rs:429:4
    |
429 | fn write_segment_atomic(
    |    ^^^^^^^^^^^^^^^^^^^^

warning: function `compute_sha256_file` is never used
   --> crates/agent-macos/src/capture_macos_rotating.rs:507:4
    |
507 | fn compute_sha256_file(path: &PathBuf) -> Result<String, String> {
    |    ^^^^^^^^^^^^^^^^^^^

warning: function `compute_sha256_string` is never used
   --> crates/agent-macos/src/capture_macos_rotating.rs:524:4
    |
524 | fn compute_sha256_string(data: &str) -> String {
    |    ^^^^^^^^^^^^^^^^^^^^^

warning: function `update_index_atomic` is never used
   --> crates/agent-macos/src/capture_macos_rotating.rs:530:4
    |
530 | fn update_index_atomic(
    |    ^^^^^^^^^^^^^^^^^^^

warning: function `validate_root_permissions` is never used
   --> crates/agent-macos/src/capture_macos_rotating.rs:601:4
    |
601 | fn validate_root_permissions() -> Result<(), String> {
    |    ^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `run_capture_with_mock` is never used
   --> crates/agent-macos/src/capture_macos_rotating.rs:608:4
    |
608 | fn run_capture_with_mock(fixture_path: &PathBuf) {
    |    ^^^^^^^^^^^^^^^^^^^^^

warning: function `cleanup_stale_temp_files` is never used
   --> crates/agent-macos/src/capture_macos_rotating.rs:697:4
    |
697 | fn cleanup_stale_temp_files(segments_dir: &PathBuf) {
    |    ^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `get_build_id` is never used
   --> crates/agent-macos/src/capture_macos_rotating.rs:725:4
    |
725 | fn get_build_id() -> String {
    |    ^^^^^^^^^^^^

warning: function `write_heartbeat` is never used
   --> crates/agent-macos/src/capture_macos_rotating.rs:730:4
    |
730 | fn write_heartbeat(
    |    ^^^^^^^^^^^^^^^

warning: function `macos_dedup_key` is never used
   --> crates/agent-macos/src/capture_macos_rotating.rs:805:4
    |
805 | fn macos_dedup_key(event: &edr_core::Event) -> Option<String> {
    |    ^^^^^^^^^^^^^^^

warning: function `main` is never used
   --> crates/agent-macos/src/capture_macos_rotating.rs:951:4
    |
951 | fn main() {
    |    ^^^^

warning: field `pos` is never read
   --> crates/agent-macos/src/platform/bsm_parser.rs:177:5
    |
175 | pub struct BSMParser {
    |            --------- field in this struct
176 |     buffer: Vec<u8>,
177 |     pos: usize,
    |     ^^^

warning: constant `PRIVILEGED_UIDS` is never used
  --> crates/agent-macos/src/sensors/bsm/bsm_priv_escalation.rs:26:7
   |
26 | const PRIVILEGED_UIDS: &[u32] = &[
   |       ^^^^^^^^^^^^^^^

warning: function `read_u16_le` is never used
  --> crates/agent-macos/src/sensors/bsm/bsm_tokens.rs:92:4
   |
92 | fn read_u16_le(data: &[u8], offset: usize) -> Option<u16> {
   |    ^^^^^^^^^^^

warning: function `estimate_token_len_with_data` is never used
   --> crates/agent-macos/src/sensors/bsm/bsm_tokens.rs:397:4
    |
397 | fn estimate_token_len_with_data(data: &[u8]) -> Option<usize> {
    |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `edr_core::event_keys`
   --> crates/agent-macos/src/sensors/bsm/bsm_network.rs:447:9
    |
447 |     use edr_core::event_keys;
    |         ^^^^^^^^^^^^^^^^^^^^

warning: unused import: `edr_core::event_keys`
   --> crates/agent-macos/src/sensors/bsm/bsm_priv_escalation.rs:752:9
    |
752 |     use edr_core::event_keys;
    |         ^^^^^^^^^^^^^^^^^^^^

warning: `agent-macos` (lib) generated 144 warnings (run `cargo fix --lib -p agent-macos` to apply 20 suggestions)
warning: `agent-macos` (lib test) generated 146 warnings (144 duplicates) (run `cargo fix --lib -p agent-macos --tests` to apply 2 suggestions)
warning: fields `id` and `tx` are never read
   --> crates/integration/src/realtime_sync.rs:100:5
    |
99  | struct ClientSession {
    |        ------------- fields in this struct
100 |     id: String,
    |     ^^
101 |     subscription: Subscription,
102 |     tx: broadcast::Sender<SyncMessage>,
    |     ^^
    |
    = note: `#[warn(dead_code)]` on by default

warning: unused import: `EvidenceRef`
 --> crates/locald/tests/pipeline_integration.rs:7:52
  |
7 |     BaselineQuery, BaselineStore, BaselineUpdater, EvidenceRef, HostBaseline, LinuxSignalEngine,
  |                                                    ^^^^^^^^^^^
  |
  = note: `#[warn(unused_imports)]` on by default

warning: unused import: `std::path::PathBuf`
   --> crates/locald/tests/pipeline_integration.rs:457:9
    |
457 |     use std::path::PathBuf;
    |         ^^^^^^^^^^^^^^^^^^

warning: comparison is useless due to type limits
  --> crates/locald/tests/pipeline_integration.rs:85:13
   |
85 |     assert!(signals.len() >= 0);
   |             ^^^^^^^^^^^^^^^^^^
   |
   = note: `#[warn(unused_comparisons)]` on by default

warning: comparison is useless due to type limits
   --> crates/locald/tests/pipeline_integration.rs:101:13
    |
101 |     assert!(signals.len() >= 0);
    |             ^^^^^^^^^^^^^^^^^^

warning: comparison is useless due to type limits
   --> crates/locald/tests/pipeline_integration.rs:117:13
    |
117 |     assert!(signals.len() >= 0);
    |             ^^^^^^^^^^^^^^^^^^

warning: comparison is useless due to type limits
   --> crates/locald/tests/pipeline_integration.rs:149:13
    |
149 |     assert!(signals.len() >= 0);
    |             ^^^^^^^^^^^^^^^^^^

warning: comparison is useless due to type limits
   --> crates/locald/tests/pipeline_integration.rs:171:13
    |
171 |     assert!(signals.len() >= 0);
    |             ^^^^^^^^^^^^^^^^^^

warning: comparison is useless due to type limits
   --> crates/locald/tests/pipeline_integration.rs:193:13
    |
193 |     assert!(signals.len() >= 0);
    |             ^^^^^^^^^^^^^^^^^^

warning: comparison is useless due to type limits
   --> crates/locald/tests/pipeline_integration.rs:323:17
    |
323 |         assert!(signal.evidence_ptrs.len() >= 0 || signal.dropped_evidence_count >= 0);
    |                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: comparison is useless due to type limits
   --> crates/locald/tests/pipeline_integration.rs:323:52
    |
323 |         assert!(signal.evidence_ptrs.len() >= 0 || signal.dropped_evidence_count >= 0);
    |                                                    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: comparison is useless due to type limits
   --> crates/locald/tests/pipeline_integration.rs:360:13
    |
360 |     assert!(signals.len() >= 0);
    |             ^^^^^^^^^^^^^^^^^^

warning: unused import: `chrono::Utc`
  --> crates/locald/tests/hypothesis_wiring_test.rs:10:5
   |
10 | use chrono::Utc;
   |     ^^^^^^^^^^^
   |
   = note: `#[warn(unused_imports)]` on by default

warning: unused imports: `GlobalWatermark` and `HypothesisStorage`
  --> crates/locald/tests/hypothesis_wiring_test.rs:30:5
   |
30 |     GlobalWatermark,
   |     ^^^^^^^^^^^^^^^
...
35 |     HypothesisStorage,
   |     ^^^^^^^^^^^^^^^^^

warning: unused import: `TimeZone`
   --> crates/locald/tests/hypothesis_wiring_test.rs:224:28
    |
224 |     use chrono::{Duration, TimeZone, Utc};
    |                            ^^^^^^^^

warning: unused import: `std::collections::HashMap`
   --> crates/locald/src/baseline/baseline_query.rs:124:9
    |
124 |     use std::collections::HashMap;
    |         ^^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = note: `#[warn(unused_imports)]` on by default

warning: unused import: `Path`
 --> crates/locald/src/baseline/baseline_store.rs:7:17
  |
7 | use std::path::{Path, PathBuf};
  |                 ^^^^

warning: unused import: `edr_core::EvidencePtr`
  --> crates/locald/src/scoring/scored_signal.rs:88:13
   |
88 |         use edr_core::EvidencePtr;
   |             ^^^^^^^^^^^^^^^^^^^^^

warning: unused imports: `ExplanationVisibilityState`, `MissingEvidence`, and `SlotStatusSummary`
  --> crates/locald/src/../../../locald/hypothesis/copilot.rs:11:49
   |
11 |     Claim, ClaimCertainty, ExplanationResponse, ExplanationVisibilityState, MissingEvidence,
   |                                                 ^^^^^^^^^^^^^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^^
12 |     MissingReason, QueryContext, SlotStatusSummary,
   |                                  ^^^^^^^^^^^^^^^^^

warning: variable does not need to be mutable
   --> crates/locald/src/../../../locald/hypothesis/incident.rs:711:13
    |
711 |         let mut incident1 = Incident::from_hypothesis(&hyp1, "host1");
    |             ----^^^^^^^^^
    |             |
    |             help: remove this `mut`
    |
    = note: `#[warn(unused_mut)]` on by default

warning: variable does not need to be mutable
   --> crates/locald/src/../../../locald/hypothesis/storage.rs:988:13
    |
988 |         let mut storage = InMemoryStorage::new();
    |             ----^^^^^^^
    |             |
    |             help: remove this `mut`

warning: unused variable: `i`
   --> crates/locald/src/../../../locald/hypothesis/closure.rs:579:13
    |
579 |         for i in 0..2 {
    |             ^ help: if this is intentional, prefix it with an underscore: `_i`

warning: unused variable: `baseline_watermark`
   --> crates/locald/src/../../../locald/hypothesis/ordering.rs:895:13
    |
895 |         let baseline_watermark = gate.watermark.global_high;
    |             ^^^^^^^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_baseline_watermark`

warning: unused variable: `is_late`
   --> crates/locald/src/../../../locald/hypothesis/ordering.rs:922:22
    |
922 |         let (action, is_late) = gate.check_and_update("stream1", "seg1", 2, normal_time);
    |                      ^^^^^^^ help: if this is intentional, prefix it with an underscore: `_is_late`

warning: variable does not need to be mutable
   --> crates/locald/src/../../../locald/hypothesis/versioning.rs:563:13
    |
563 |         let mut config1 = SessionConfigVersion::new();
    |             ----^^^^^^^
    |             |
    |             help: remove this `mut`

warning: methods `resolved_contents`, `missing_ptrs`, and `has_integrity_errors` are never used
   --> crates/locald/src/evidence/deref.rs:317:12
    |
269 | impl BulkDerefResult {
    | -------------------- methods in this implementation
...
317 |     pub fn resolved_contents(&self) -> Vec<(&EvidencePtr, &str)> {
    |            ^^^^^^^^^^^^^^^^^
...
326 |     pub fn missing_ptrs(&self) -> Vec<&EvidencePtr> {
    |            ^^^^^^^^^^^^
...
340 |     pub fn has_integrity_errors(&self) -> bool {
    |            ^^^^^^^^^^^^^^^^^^^^
    |
    = note: `#[warn(dead_code)]` on by default

warning: methods `with_sha256_from_bytes` and `with_ts` are never used
   --> crates/locald/src/evidence/evidence_ptr.rs:242:12
    |
222 | impl EvidencePtrBuilder {
    | ----------------------- methods in this implementation
...
242 |     pub fn with_sha256_from_bytes(mut self, bytes: &[u8]) -> Self {
    |            ^^^^^^^^^^^^^^^^^^^^^^
...
247 |     pub fn with_ts(mut self, ts: i64) -> Self {
    |            ^^^^^^^

warning: `integration` (lib) generated 1 warning
warning: `edr-locald` (test "pipeline_integration") generated 11 warnings (run `cargo fix --test "pipeline_integration"` to apply 2 suggestions)
warning: `edr-locald` (test "hypothesis_wiring_test") generated 3 warnings (run `cargo fix --test "hypothesis_wiring_test"` to apply 3 suggestions)
warning: `edr-locald` (lib test) generated 38 warnings (26 duplicates) (run `cargo fix --lib -p edr-locald --tests` to apply 7 suggestions)
warning: `agent-windows` (lib test) generated 15 warnings (15 duplicates)
warning: unused import: `std::collections::HashMap`
 --> crates/server/tests/credibility_lock_tests.rs:9:5
  |
9 | use std::collections::HashMap;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^
  |
  = note: `#[warn(unused_imports)]` on by default

warning: unused import: `tempfile::TempDir`
  --> crates/server/tests/credibility_lock_tests.rs:11:5
   |
11 | use tempfile::TempDir;
   |     ^^^^^^^^^^^^^^^^^

warning: field `stream_id` is never read
   --> crates/server/tests/credibility_lock_tests.rs:200:5
    |
199 | struct StreamDiagnostic {
    |        ---------------- field in this struct
200 |     stream_id: String,
    |     ^^^^^^^^^
    |
    = note: `StreamDiagnostic` has derived impls for the traits `Clone` and `Debug`, but these are intentionally ignored during dead code analysis
    = note: `#[warn(dead_code)]` on by default

warning: variants `Info` and `Warning` are never constructed
   --> crates/server/tests/credibility_lock_tests.rs:207:5
    |
206 | enum IssueSeverity {
    |      ------------- variants in this enum
207 |     Info,
    |     ^^^^
208 |     Warning,
    |     ^^^^^^^
    |
    = note: `IssueSeverity` has derived impls for the traits `Clone` and `Debug`, but these are intentionally ignored during dead code analysis

warning: unused import: `std::collections::HashMap`
  --> crates/server/src/golden_bundle.rs:26:5
   |
26 | use std::collections::HashMap;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: `#[warn(unused_imports)]` on by default

warning: unused import: `std::time::Duration`
 --> crates/server/tests/diagnostics_probe_tests.rs:4:5
  |
4 | use std::time::Duration;
  |     ^^^^^^^^^^^^^^^^^^^
  |
  = note: `#[warn(unused_imports)]` on by default

warning: unused variable: `severities`
  --> crates/server/tests/diagnostics_probe_tests.rs:93:9
   |
93 |     let severities = ["critical", "error", "warning", "info"];
   |         ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_severities`
   |
   = note: `#[warn(unused_variables)]` on by default

warning: unused variable: `default_json`
   --> crates/server/tests/diagnostics_probe_tests.rs:160:9
    |
160 |     let default_json = r#"{}"#;
    |         ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_default_json`

warning: unused variable: `universal_actions`
   --> crates/server/tests/diagnostics_probe_tests.rs:341:9
    |
341 |     let universal_actions = ["run_probe", "adjust_throttle", "check_storage"];
    |         ^^^^^^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_universal_actions`

warning: `integration` (lib test) generated 1 warning (1 duplicate)
warning: `agent-linux` (lib test) generated 13 warnings (13 duplicates)
warning: `edr-server` (test "credibility_lock_tests") generated 4 warnings (run `cargo fix --test "credibility_lock_tests"` to apply 2 suggestions)
warning: `edr-server` (lib test) generated 4 warnings (3 duplicates) (run `cargo fix --lib -p edr-server --tests` to apply 1 suggestion)
warning: `edr-server` (test "diagnostics_probe_tests") generated 4 warnings (run `cargo fix --test "diagnostics_probe_tests"` to apply 1 suggestion)
warning: unused import: `std::time::Duration`
 --> crates/server/tests/capture_control_tests.rs:6:5
  |
6 | use std::time::Duration;
  |     ^^^^^^^^^^^^^^^^^^^
  |
  = note: `#[warn(unused_imports)]` on by default

warning: unused variable: `rate_per_sec`
  --> crates/server/tests/capture_control_tests.rs:15:9
   |
15 |     let rate_per_sec = 10u32;
   |         ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_rate_per_sec`
   |
   = note: `#[warn(unused_variables)]` on by default

warning: unused variable: `window_events`
   --> crates/server/tests/capture_control_tests.rs:336:9
    |
336 |     let window_events = 0;
    |         ^^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_window_events`

warning: field `global_byte_rate` is never read
  --> crates/server/tests/capture_control_tests.rs:58:9
   |
56 |     struct ProfileConfig {
   |            ------------- field in this struct
57 |         global_event_rate: u32,
58 |         global_byte_rate: u64,
   |         ^^^^^^^^^^^^^^^^
   |
   = note: `#[warn(dead_code)]` on by default

warning: constant `WINDOW_MS` is never used
   --> crates/server/tests/capture_control_tests.rs:170:11
    |
170 |     const WINDOW_MS: u64 = 5000;
    |           ^^^^^^^^^

warning: field `enabled_collectors` is never read
   --> crates/server/tests/capture_control_tests.rs:203:9
    |
197 |     struct ConfigSnapshot {
    |            -------------- field in this struct
...
203 |         enabled_collectors: Vec<String>,
    |         ^^^^^^^^^^^^^^^^^^

warning: fields `priority`, `rate_per_sec`, `burst`, and `max_queue` are never read
   --> crates/server/tests/capture_control_tests.rs:207:9
    |
206 |     struct StreamConfig {
    |            ------------ fields in this struct
207 |         priority: String,
    |         ^^^^^^^^
208 |         rate_per_sec: u32,
    |         ^^^^^^^^^^^^
209 |         burst: u32,
    |         ^^^^^
210 |         max_queue: u32,
    |         ^^^^^^^^^

warning: field `ts` is never read
   --> crates/server/tests/capture_control_tests.rs:249:9
    |
245 |     struct AuditEntry {
    |            ---------- field in this struct
...
249 |         ts: std::time::SystemTime,
    |         ^^

warning: fields `window_start`, `window_end`, `events_sampled`, and `reason` are never read
   --> crates/server/tests/capture_control_tests.rs:286:9
    |
285 |     struct TelemetryThrottledSummary {
    |            ------------------------- fields in this struct
286 |         window_start: std::time::SystemTime,
    |         ^^^^^^^^^^^^
287 |         window_end: std::time::SystemTime,
    |         ^^^^^^^^^^
...
290 |         events_sampled: u64,
    |         ^^^^^^^^^^^^^^
291 |         reason: String,
    |         ^^^^^^

warning: fields `profile` and `burst` are never read
   --> crates/server/tests/capture_control_tests.rs:538:9
    |
537 |     struct ThrottleState {
    |            ------------- fields in this struct
538 |         profile: String,
    |         ^^^^^^^
539 |         tokens: u32,
540 |         burst: u32,
    |         ^^^^^

warning: unused import: `std::path::Path`
   --> crates/server/tests/verification_tests.rs:242:9
    |
242 |     use std::path::Path;
    |         ^^^^^^^^^^^^^^^
    |
    = note: `#[warn(unused_imports)]` on by default

warning: unused imports: `ImportedBundleRecord`, `ImportedBundleStore`, and `recompute_from_bundle`
  --> crates/server/src/main.rs:22:23
   |
22 |     mark_as_imported, recompute_from_bundle, validate_bundle, ExportBundleRequest,
   |                       ^^^^^^^^^^^^^^^^^^^^^
23 |     ExportBundleResponse, ImportBundleResponse, ImportedBundleRecord, ImportedBundleStore,
   |                                                 ^^^^^^^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^^^^^^
   |
   = note: `#[warn(unused_imports)]` on by default

warning: unused imports: `TelemetryThrottledSummary` and `ThrottleVisibilityState`
  --> crates/server/src/main.rs:27:36
   |
27 |     CaptureProfile, ProfileConfig, TelemetryThrottledSummary, ThrottleController, ThrottleDecision,
   |                                    ^^^^^^^^^^^^^^^^^^^^^^^^^
28 |     ThrottleVisibilityState,
   |     ^^^^^^^^^^^^^^^^^^^^^^^

warning: unused imports: `ActionDetails`, `SelfCheckResponse`, `SelfCheckVerdict`, and `StreamStats`
  --> crates/server/src/main.rs:32:5
   |
32 |     ActionDetails, DiagnosticEngine, SelfCheckResponse, SelfCheckVerdict, StreamStats,
   |     ^^^^^^^^^^^^^                    ^^^^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^^^  ^^^^^^^^^^^

warning: unused import: `ProbeResult`
  --> crates/server/src/main.rs:34:13
   |
34 | use probe::{ProbeResult, ProbeRunner, ProbeSpec};
   |             ^^^^^^^^^^^

warning: unused variable: `response`
   --> crates/server/src/main.rs:966:17
    |
966 |             let response = SupportBundleResponse {
    |                 ^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_response`
    |
    = note: `#[warn(unused_variables)]` on by default

warning: unused variable: `stream_id`
   --> crates/server/src/capture_control.rs:909:34
    |
909 |     fn maybe_emit_summary(&self, stream_id: &str, stream: &StreamState) {
    |                                  ^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_stream_id`

warning: unused variable: `stream`
   --> crates/server/src/capture_control.rs:909:51
    |
909 |     fn maybe_emit_summary(&self, stream_id: &str, stream: &StreamState) {
    |                                                   ^^^^^^ help: if this is intentional, prefix it with an underscore: `_stream`

warning: unused variable: `j`
    --> crates/server/src/capture_control.rs:1523:25
     |
1523 |                     for j in 0..100 {
     |                         ^ help: if this is intentional, prefix it with an underscore: `_j`

warning: unused variable: `s`
   --> crates/server/src/diagnostics.rs:537:68
    |
537 |                 let expected_running = enabled_sensors.iter().any(|s| {
    |                                                                    ^ help: if this is intentional, prefix it with an underscore: `_s`

warning: field `start_time` is never read
  --> crates/server/src/main.rs:74:5
   |
56 | struct AppState {
   |        -------- field in this struct
...
74 |     start_time: chrono::DateTime<chrono::Utc>,
   |     ^^^^^^^^^^
   |
   = note: `#[warn(dead_code)]` on by default

warning: field `note` is never read
   --> crates/server/src/main.rs:124:5
    |
122 | struct SessionControlRequest {
    |        --------------------- field in this struct
123 |     action: String, // "start", "stop", "pause", "resume"
124 |     note: Option<String>,
    |     ^^^^
    |
    = note: `SessionControlRequest` has a derived impl for the trait `Debug`, but this is intentionally ignored during dead code analysis

warning: field `allowed_extensions` is never read
  --> crates/server/src/bundle_exchange.rs:48:9
   |
42 | pub struct ZipSafetyPolicy {
   |            --------------- field in this struct
...
48 |     pub allowed_extensions: Vec<&'static str>,
   |         ^^^^^^^^^^^^^^^^^^
   |
   = note: `ZipSafetyPolicy` has derived impls for the traits `Clone` and `Debug`, but these are intentionally ignored during dead code analysis

warning: field `bundles` is never read
   --> crates/server/src/bundle_exchange.rs:319:5
    |
318 | pub struct ImportedBundleStore {
    |            ------------------- field in this struct
319 |     bundles: std::sync::RwLock<HashMap<String, ImportedBundleRecord>>,
    |     ^^^^^^^

warning: associated items `new`, `store`, `get`, `list`, and `remove` are never used
   --> crates/server/src/bundle_exchange.rs:323:12
    |
322 | impl ImportedBundleStore {
    | ------------------------ associated items in this implementation
323 |     pub fn new() -> Self {
    |            ^^^
...
327 |     pub fn store(&self, record: ImportedBundleRecord) {
    |            ^^^^^
...
332 |     pub fn get(&self, bundle_id: &str) -> Option<ImportedBundleRecord> {
    |            ^^^
...
337 |     pub fn list(&self) -> Vec<String> {
    |            ^^^^
...
342 |     pub fn remove(&self, bundle_id: &str) -> bool {
    |            ^^^^^^

warning: method `as_str` is never used
   --> crates/server/src/capture_control.rs:343:12
    |
342 | impl DropReason {
    | --------------- method in this implementation
343 |     pub fn as_str(&self) -> &'static str {
    |            ^^^^^^

warning: methods `record_sample` and `record_queue` are never used
   --> crates/server/src/capture_control.rs:498:12
    |
477 | impl StreamCounters {
    | ------------------- methods in this implementation
...
498 |     pub fn record_sample(&self) {
    |            ^^^^^^^^^^^^^
...
502 |     pub fn record_queue(&self) {
    |            ^^^^^^^^^^^^

warning: methods `take_pending_summaries`, `get_audit_log`, `clear_audit_log`, `is_sensor_enabled`, `is_collector_enabled`, and `is_heavy_feature_enabled` are never used
   --> crates/server/src/capture_control.rs:964:12
    |
592 | impl ThrottleController {
    | ----------------------- methods in this implementation
...
964 |     pub fn take_pending_summaries(&self) -> Vec<TelemetryThrottledSummary> {
    |            ^^^^^^^^^^^^^^^^^^^^^^
...
975 |     pub fn get_audit_log(&self) -> Vec<ThrottleAuditEntry> {
    |            ^^^^^^^^^^^^^
...
980 |     pub fn clear_audit_log(&self) {
    |            ^^^^^^^^^^^^^^^
...
985 |     pub fn is_sensor_enabled(&self, sensor_id: &str) -> bool {
    |            ^^^^^^^^^^^^^^^^^
...
991 |     pub fn is_collector_enabled(&self, collector_id: &str) -> bool {
    |            ^^^^^^^^^^^^^^^^^^^^
...
999 |     pub fn is_heavy_feature_enabled(&self, feature_id: &str) -> bool {
    |            ^^^^^^^^^^^^^^^^^^^^^^^^

warning: method `validate_config_best_effort` is never used
    --> crates/server/src/capture_control.rs:1178:12
     |
1065 | impl ThrottleController {
     | ----------------------- method in this implementation
...
1178 |     pub fn validate_config_best_effort(&self, snapshot: &ThrottleConfigSnapshot) -> Vec<String> {
     |            ^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: associated function `event_type` is never used
    --> crates/server/src/capture_control.rs:1214:12
     |
1213 | impl TelemetryThrottledSummary {
     | ------------------------------ associated function in this implementation
1214 |     pub fn event_type() -> &'static str {
     |            ^^^^^^^^^^

warning: methods `as_str` and `emoji` are never used
   --> crates/server/src/diagnostics.rs:104:12
    |
103 | impl SelfCheckVerdict {
    | --------------------- methods in this implementation
104 |     pub fn as_str(&self) -> &'static str {
    |            ^^^^^^
...
112 |     pub fn emoji(&self) -> &'static str {
    |            ^^^^^

warning: field `event_observer` is never read
   --> crates/server/src/probe.rs:194:5
    |
192 | pub struct ProbeRunner {
    |            ----------- field in this struct
193 |     /// Event sink for observing results (optional, for testing)
194 |     event_observer: Option<mpsc::Sender<ObservedEventSummary>>,
    |     ^^^^^^^^^^^^^^

warning: associated items `with_observer` and `notify_event` are never used
   --> crates/server/src/probe.rs:205:12
    |
197 | impl ProbeRunner {
    | ---------------- associated items in this implementation
...
205 |     pub fn with_observer(observer: mpsc::Sender<ObservedEventSummary>) -> Self {
    |            ^^^^^^^^^^^^^
...
552 |     pub async fn notify_event(&self, event: ObservedEventSummary) {
    |                  ^^^^^^^^^^^^

warning: field `observed_command` is never read
   --> crates/server/src/probe.rs:790:9
    |
786 | pub struct EventCorrelationDetail {
    |            ---------------------- field in this struct
...
790 |     pub observed_command: Option<String>,
    |         ^^^^^^^^^^^^^^^^
    |
    = note: `EventCorrelationDetail` has derived impls for the traits `Clone` and `Debug`, but these are intentionally ignored during dead code analysis

warning: methods `with_generated_at` and `with_time_window` are never used
   --> crates/server/src/report.rs:824:12
    |
774 | impl ReportBundleBuilder {
    | ------------------------ methods in this implementation
...
824 |     pub fn with_generated_at(mut self, ts: DateTime<Utc>) -> Self {
    |            ^^^^^^^^^^^^^^^^^
...
840 |     pub fn with_time_window(mut self, t_min: DateTime<Utc>, t_max: DateTime<Utc>) -> Self {
    |            ^^^^^^^^^^^^^^^^

warning: method `with_incident` is never used
   --> crates/server/src/support_bundle.rs:286:12
    |
247 | impl SupportBundleBuilder {
    | ------------------------- method in this implementation
...
286 |     pub fn with_incident(mut self, incident_json: String) -> Self {
    |            ^^^^^^^^^^^^^

warning: associated function `new` is never used
   --> crates/server/src/verification_pack.rs:350:12
    |
349 | impl VerificationState {
    | ---------------------- associated function in this implementation
350 |     pub fn new() -> Self {
    |            ^^^

warning: unused imports: `ImportedBundleRecord`, `ImportedBundleStore`, and `recompute_from_bundle`
  --> crates/server/src/main.rs:22:23
   |
22 |     mark_as_imported, recompute_from_bundle, validate_bundle, ExportBundleRequest,
   |                       ^^^^^^^^^^^^^^^^^^^^^
23 |     ExportBundleResponse, ImportBundleResponse, ImportedBundleRecord, ImportedBundleStore,
   |                                                 ^^^^^^^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^^^^^^

warning: constant `IMPORTED_NAMESPACE` is never used
  --> crates/server/src/bundle_exchange.rs:35:11
   |
35 | pub const IMPORTED_NAMESPACE: &str = "imported_bundle";
   |           ^^^^^^^^^^^^^^^^^^

warning: associated items `new`, `store`, `get`, `list`, `remove`, and `is_imported_namespace` are never used
   --> crates/server/src/bundle_exchange.rs:323:12
    |
322 | impl ImportedBundleStore {
    | ------------------------ associated items in this implementation
323 |     pub fn new() -> Self {
    |            ^^^
...
327 |     pub fn store(&self, record: ImportedBundleRecord) {
    |            ^^^^^
...
332 |     pub fn get(&self, bundle_id: &str) -> Option<ImportedBundleRecord> {
    |            ^^^
...
337 |     pub fn list(&self) -> Vec<String> {
    |            ^^^^
...
342 |     pub fn remove(&self, bundle_id: &str) -> bool {
    |            ^^^^^^
...
348 |     pub fn is_imported_namespace(namespace: &str) -> bool {
    |            ^^^^^^^^^^^^^^^^^^^^^

warning: function `export_to_json` is never used
   --> crates/server/src/bundle_exchange.rs:640:8
    |
640 | pub fn export_to_json(bundle: &IncidentBundle) -> Result<Vec<u8>, String> {
    |        ^^^^^^^^^^^^^^

warning: function `recompute_from_bundle` is never used
    --> crates/server/src/bundle_exchange.rs:1052:8
     |
1052 | pub fn recompute_from_bundle(
     |        ^^^^^^^^^^^^^^^^^^^^^

warning: multiple methods are never used
   --> crates/server/src/capture_control.rs:964:12
    |
592 | impl ThrottleController {
    | ----------------------- methods in this implementation
...
964 |     pub fn take_pending_summaries(&self) -> Vec<TelemetryThrottledSummary> {
    |            ^^^^^^^^^^^^^^^^^^^^^^
...
970 |     pub fn has_critical_gap(&self) -> bool {
    |            ^^^^^^^^^^^^^^^^
...
975 |     pub fn get_audit_log(&self) -> Vec<ThrottleAuditEntry> {
    |            ^^^^^^^^^^^^^
...
980 |     pub fn clear_audit_log(&self) {
    |            ^^^^^^^^^^^^^^^
...
985 |     pub fn is_sensor_enabled(&self, sensor_id: &str) -> bool {
    |            ^^^^^^^^^^^^^^^^^
...
991 |     pub fn is_collector_enabled(&self, collector_id: &str) -> bool {
    |            ^^^^^^^^^^^^^^^^^^^^
...
999 |     pub fn is_heavy_feature_enabled(&self, feature_id: &str) -> bool {
    |            ^^^^^^^^^^^^^^^^^^^^^^^^

warning: methods `validate_config_match` and `validate_config_best_effort` are never used
    --> crates/server/src/capture_control.rs:1099:12
     |
1065 | impl ThrottleController {
     | ----------------------- methods in this implementation
...
1099 |     pub fn validate_config_match(
     |            ^^^^^^^^^^^^^^^^^^^^^
...
1178 |     pub fn validate_config_best_effort(&self, snapshot: &ThrottleConfigSnapshot) -> Vec<String> {
     |            ^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: associated items `event_type`, `ordering_key`, and `tier0_warning` are never used
    --> crates/server/src/capture_control.rs:1214:12
     |
1213 | impl TelemetryThrottledSummary {
     | ------------------------------ associated items in this implementation
1214 |     pub fn event_type() -> &'static str {
     |            ^^^^^^^^^^
...
1219 |     pub fn ordering_key(&self) -> (String, i64, u32) {
     |            ^^^^^^^^^^^^
...
1228 |     pub fn tier0_warning(&self) -> Option<&'static str> {
     |            ^^^^^^^^^^^^^

warning: associated function `open_in_memory` is never used
  --> crates/server/src/db.rs:40:12
   |
30 | impl Database {
   | ------------- associated function in this implementation
...
40 |     pub fn open_in_memory() -> Result<Self, rusqlite::Error> {
   |            ^^^^^^^^^^^^^^

warning: function `correlate_events` is never used
   --> crates/server/src/probe.rs:570:8
    |
570 | pub fn correlate_events(
    |        ^^^^^^^^^^^^^^^^

warning: function `correlate_events_robust` is never used
   --> crates/server/src/probe.rs:606:8
    |
606 | pub fn correlate_events_robust(
    |        ^^^^^^^^^^^^^^^^^^^^^^^

warning: struct `EventCorrelationDetail` is never constructed
   --> crates/server/src/probe.rs:786:12
    |
786 | pub struct EventCorrelationDetail {
    |            ^^^^^^^^^^^^^^^^^^^^^^
    |
    = note: `EventCorrelationDetail` has derived impls for the traits `Clone` and `Debug`, but these are intentionally ignored during dead code analysis

warning: function `compute_sha256` is never used
   --> crates/server/src/support_bundle.rs:161:8
    |
161 | pub fn compute_sha256(data: &[u8]) -> String {
    |        ^^^^^^^^^^^^^^

warning: `edr-server` (test "capture_control_tests") generated 10 warnings (run `cargo fix --test "capture_control_tests"` to apply 1 suggestion)
warning: `edr-server` (test "verification_tests") generated 1 warning (run `cargo fix --test "verification_tests"` to apply 1 suggestion)
warning: `edr-server` (bin "edr-server" test) generated 26 warnings (run `cargo fix --bin "edr-server" --tests` to apply 4 suggestions)
warning: `edr-server` (bin "edr-server") generated 33 warnings (20 duplicates) (run `cargo fix --bin "edr-server"` to apply 1 suggestion)
warning: `workbench` (lib test) generated 1 warning (1 duplicate)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.65s
     Running unittests src/lib.rs (target/debug/deps/agent_linux-fb262cd9cf009c1d)

running 8 tests
test telemetry::tests::test_risk_score_clamping ... ok
test host::tests::test_host_ctx_creation ... ok
test telemetry::tests::test_telemetry_record_builder ... ok
test event_builders::tests::test_network_event_uses_canonical_field_keys ... ok
test sensors::ebpf_primitives::composite_detectors::tests::test_shadow_file_access_detection ... ok
test telemetry_types::tests::test_confidence_normalization ... ok
test sensors::ebpf_primitives::composite_detectors::tests::test_ptrace_injection_detection ... ok
test telemetry_types::tests::test_telemetry_output_builder ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/main.rs (target/debug/deps/agent_linux-647ba9960c736c7c)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/lib.rs (target/debug/deps/agent_macos-2a437c48042ced95)

running 92 tests
test platform::bsm_parser::tests::test_aue_to_event_type ... ok
test macos_ingest::tests::test_parse_exec_event ... ok
test macos_ingest::tests::test_parse_segment ... ok
test platform::bsm_parser::tests::test_parser_no_panic_on_malformed ... ok
test platform::bsm_parser::tests::test_parsed_record_to_json ... ok
test platform::bsm_parser::tests::test_token_type_from_byte ... ok
test platform::evidence::tests::test_evidence_ptr_creation ... ok
test platform::bsm_parser::tests::test_schema_version_always_one ... ok
test platform::evidence::tests::test_evidence_ptr_dedup ... ok
test platform::facts::tests::test_fact_creation ... ok
test platform::facts::tests::test_fact_path_extraction ... ok
test platform::macos_fact_extractors::tests::test_compute_proc_risk ... ok
test platform::macos_fact_extractors::tests::test_proc_exec_fact_evidence ... ok
test platform::slot_engine::tests::test_slot_engine_basic ... ok
test sensors::bsm::bsm_auth_event::tests::test_detect_bsm_auth_success ... ok
test sensors::bsm::bsm_auth_event::tests::test_detect_bsm_auth_failure ... ok
test sensors::bsm::bsm_auth_event::tests::test_detect_ssh_exec ... ok
test sensors::bsm::bsm_auth_event::tests::test_detect_su_exec ... ok
test sensors::bsm::bsm_auth_event::tests::test_extract_ssh_target ... ok
test sensors::bsm::bsm_auth_event::tests::test_detect_sudo_default_root ... ok
test sensors::bsm::bsm_auth_event::tests::test_extract_sudo_target ... ok
test sensors::bsm::bsm_auth_event::tests::test_detect_sudo_exec ... ok
test sensors::bsm::bsm_auth_event::tests::test_no_detection_non_auth_cmd ... ok
test sensors::bsm::bsm_composite_detectors::tests::test_code_signature_bypass_detection ... ok
test sensors::bsm::bsm_composite_detectors::tests::test_dyld_injection_detection ... ok
test sensors::bsm::bsm_defense_evasion::tests::test_detect_history_clear_cmd ... ok
test sensors::bsm::bsm_composite_detectors::tests::test_keychain_ssh_chain_detection ... ok
test sensors::bsm::bsm_composite_detectors::tests::test_persistence_launchctl_detection ... ok
test sensors::bsm::bsm_defense_evasion::tests::test_detect_log_deletion ... ok
test sensors::bsm::bsm_defense_evasion::tests::test_detect_history_deletion ... ok
test sensors::bsm::bsm_defense_evasion::tests::test_no_detection_normal_file ... ok
test sensors::bsm::bsm_defense_evasion::tests::test_detect_spctl_disable ... ok
test sensors::bsm::bsm_defense_evasion::tests::test_detect_xattr_quarantine_removal ... ok
test sensors::bsm::bsm_file_ops::tests::test_handle_chown_to_root ... ok
test sensors::bsm::bsm_file_ops::tests::test_handle_chmod_setuid ... ok
test sensors::bsm::bsm_file_ops::tests::test_handle_create_staging_dir ... ok
test sensors::bsm::bsm_file_ops::tests::test_handle_open_sensitive_file ... ok
test sensors::bsm::bsm_file_ops::tests::test_handle_rename_hiding_file ... ok
test sensors::bsm::bsm_file_ops::tests::test_handle_rename_to_staging ... ok
test sensors::bsm::bsm_file_ops::tests::test_handle_truncate_log_wipe ... ok
test sensors::bsm::bsm_file_ops::tests::test_rejects_empty_path ... ok
test sensors::bsm::bsm_net_connect::tests::test_net_connect_filters_private_ips ... ok
test sensors::bsm::bsm_file_ops::tests::test_handle_unlink_log_file ... ok
test sensors::bsm::bsm_network::tests::test_classify_port ... ok
test sensors::bsm::bsm_net_connect::tests::test_net_connect_uses_canonical_field_keys ... ok
test sensors::bsm::bsm_network::tests::test_handle_bind_privileged_port ... ok
test sensors::bsm::bsm_network::tests::test_handle_accept_external ... ok
test sensors::bsm::bsm_network::tests::test_handle_bind_suspicious_port ... ok
test sensors::bsm::bsm_network::tests::test_handle_connect_exfil_indicator ... ok
test sensors::bsm::bsm_network::tests::test_is_private_ip ... ok
test sensors::bsm::bsm_network::tests::test_rejects_empty_ip ... ok
test sensors::bsm::bsm_network::tests::test_handle_connect_external_suspicious ... ok
test sensors::bsm::bsm_network::tests::test_handle_connect_internal ... ok
test sensors::bsm::bsm_persistence_change::tests::test_classify_cron ... ok
test sensors::bsm::bsm_persistence_change::tests::test_classify_launchagent ... ok
test sensors::bsm::bsm_persistence_change::tests::test_classify_launchdaemon ... ok
test sensors::bsm::bsm_persistence_change::tests::test_classify_no_match ... ok
test sensors::bsm::bsm_persistence_change::tests::test_classify_profile ... ok
test sensors::bsm::bsm_priv_escalation::tests::test_classify_uid_transition ... ok
test sensors::bsm::bsm_persistence_change::tests::test_detect_launchctl_load ... ok
test sensors::bsm::bsm_persistence_change::tests::test_detect_persistence_launchagent_create ... ok
test sensors::bsm::bsm_priv_escalation::tests::test_failed_escalation ... ok
test sensors::bsm::bsm_priv_escalation::tests::test_handle_seteuid_escalation ... ok
test sensors::bsm::bsm_priv_escalation::tests::test_handle_setgid_to_wheel ... ok
test sensors::bsm::bsm_priv_escalation::tests::test_handle_setgroups_with_privileged ... ok
test sensors::bsm::bsm_priv_escalation::tests::test_handle_setregid_to_wheel ... ok
test sensors::bsm::bsm_priv_escalation::tests::test_handle_setreuid_reescalation ... ok
test sensors::bsm::bsm_priv_escalation::tests::test_handle_setuid_escalation_to_root ... ok
test sensors::bsm::bsm_priv_escalation::tests::test_handle_setuid_privilege_drop ... ok
test sensors::bsm::bsm_process_injection::tests::test_detect_dyld_insert ... ok
test sensors::bsm::bsm_process_injection::tests::test_detect_frida ... ok
test sensors::bsm::bsm_process_injection::tests::test_extract_pid_from_args ... ok
test sensors::bsm::bsm_process_injection::tests::test_detect_lldb_attach ... ok
test sensors::bsm::bsm_process_injection::tests::test_skip_legitimate_caller ... ok
test sensors::bsm::bsm_process_injection::tests::test_detect_task_for_pid ... ok
test sensors::bsm::bsm_process_injection::tests::test_skip_self_access ... ok
test sensors::bsm::bsm_script_exec::tests::test_detect_bash_inline ... ok
test sensors::bsm::bsm_script_exec::tests::test_detect_osascript ... ok
test sensors::bsm::bsm_script_exec::tests::test_detect_osascript_lolbin ... ok
test sensors::bsm::bsm_script_exec::tests::test_detect_python_exec ... ok
test sensors::bsm::bsm_script_exec::tests::test_detect_perl_exec ... ok
test sensors::bsm::bsm_script_exec::tests::test_extract_script_path ... ok
test sensors::bsm::bsm_script_exec::tests::test_detect_python_inline ... ok
test sensors::bsm::bsm_script_exec::tests::test_no_detection_non_script ... ok
test sensors::hash_keys::tests::test_file_key_deterministic ... ok
test telemetry::tests::test_risk_score_clamping ... ok
test sensors::hash_keys::tests::test_identity_key_deterministic ... ok
test sensors::hash_keys::tests::test_proc_key_deterministic ... ok
test telemetry::tests::test_telemetry_record_builder ... ok
test sensors::hash_keys::tests::test_key_format ... ok
test telemetry_types::tests::test_confidence_normalization ... ok
test telemetry_types::tests::test_telemetry_output_builder ... ok

test result: ok. 92 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/main.rs (target/debug/deps/agent_macos-80637f55eca47643)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/lib.rs (target/debug/deps/agent_windows-c06d3b6a9a0fc509)

running 21 tests
test evtlog_state::tests::test_event_log_read_state_new ... ok
test host::tests::test_host_ctx_creation ... ok
test sensors::defender_adapter::tests::test_defender_adapter_non_defender ... ok
test evtlog_state::tests::test_mark_read_error ... ok
test evtlog_state::tests::test_update_channel ... ok
test sensors::defender_adapter::tests::test_defender_adapter_detection ... ok
test sensors::evtx_collector::tests::test_evtx_record_creation ... ok
test sensors::evtx_collector::tests::test_evtx_collector_new ... ok
test sensors::etw_adapter::tests::test_etw_adapter_wmi ... ok
test sensors::evtx_collector::tests::test_bookmark_management ... ok
test sensors::etw_adapter::tests::test_etw_adapter_powershell ... ok
test sensors::evtx_collector::tests::test_poll_returns_empty_on_non_windows ... ok
test sensors::sysmon_adapter::tests::test_sysmon_adapter_non_sysmon ... ok
test sensors::sysmon_adapter_ext::tests::test_ext_adapter_non_extended ... ok
test sensors::sysmon_adapter::tests::test_sysmon_adapter_process_create ... ok
test sensors::sysmon_adapter_ext::tests::test_ext_adapter_create_remote_thread ... ok
test telemetry::tests::test_risk_score_clamping ... ok
test sensors::sysmon_adapter_ext::tests::test_ext_adapter_process_access ... ok
test telemetry::tests::test_telemetry_record_builder ... ok
test telemetry_types::tests::test_confidence_normalization ... ok
test telemetry_types::tests::test_telemetry_output_builder ... ok

test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/main.rs (target/debug/deps/agent_windows-92c22b909b58024d)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/lib.rs (target/debug/deps/edr_core-2c3f33ca04291ac9)

running 6 tests
test event_validation_test::tests::test_episode_candidate_determinism ... ok
test event_validation_test::tests::test_event_validate_empty_fields ... ok
test event_validation_test::tests::test_event_validate_mixed_valid_and_invalid ... ok
test event_validation_test::tests::test_event_validate_unknown_field ... ok
test event_validation_test::tests::test_event_keys_coverage ... ok
test event_validation_test::tests::test_event_validate_all_valid_keys ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/lib.rs (target/debug/deps/edr_locald-21064e524239e784)

running 182 tests
test baseline::baseline_update::tests::test_extract_prefix ... ok
test baseline::types::tests::test_rate_baseline_burst ... ok
test baseline::baseline_query::tests::test_is_new_parent_child ... ok
test baseline::baseline_query::tests::test_is_burst ... ok
test baseline::baseline_update::tests::test_update_first_seen ... ok
test baseline::types::tests::test_feature_vector_baseline ... ok
test baseline::types::tests::test_rate_baseline_ema ... ok
test baseline::baseline_query::tests::test_is_new_remote_ip ... ok
test evidence::evidence_ptr::tests::test_evidence_ptr_determinism ... ok
test evidence::evidence_ptr::tests::test_canonical_key_roundtrip ... ok
test baseline::baseline_store::tests::test_baseline_store_create ... ok
test evidence::evidence_ptr::tests::test_evidence_ptr_different_streams ... ok
test baseline::baseline_store::tests::test_baseline_store_get_or_create ... ok
test evidence::evidence_ptr::tests::test_evidence_ptr_ordering ... ok
test evidence::path_safety::tests::test_safe_join_traversal ... ok
test evidence::path_safety::tests::test_validate_path_component_absolute ... ok
test evidence::evidence_ptr::tests::test_sha256_verification ... ok
test evidence::path_safety::tests::test_safe_join_valid ... ok
test evidence::path_safety::tests::test_validate_path_component_traversal ... ok
test evidence::path_safety::tests::test_validate_path_component_empty ... ok
test evidence::path_safety::tests::test_validate_path_component_valid ... ok
test evidence::path_safety::tests::test_validate_path_component_with_separator ... ok
test evidence::path_safety::tests::test_validate_relative_path_absolute ... ok
test evidence::path_safety::tests::test_validate_relative_path_traversal ... ok
test evidence::path_safety::tests::test_validate_relative_path_valid ... ok
test evidence::path_safety::tests::test_validate_segment_id_invalid ... ok
test evidence::path_safety::tests::test_validate_segment_id_valid ... ok
test hypothesis::api::tests::test_api_error_with_details ... ok
test baseline::baseline_store::tests::test_baseline_store_persistence ... ok
test hypothesis::api::tests::test_api_response_err ... ok
test hypothesis::api::tests::test_api_response_ok ... ok
test hypothesis::api::tests::test_scope_key_request_conversion ... ok
test hypothesis::audit::tests::test_audit_entry_builder ... ok
test hypothesis::arbitration::tests::test_arbitration_empty ... ok
test hypothesis::arbitration::tests::test_subchain_detection ... ok
test hypothesis::audit::tests::test_required_references ... ok
test hypothesis::audit::tests::test_audit_log_record_and_lookup ... ok
test hypothesis::arbitration::tests::test_arbitration_ranking ... ok
test hypothesis::canonical_event::tests::test_canonical_event_ordering ... ok
test hypothesis::audit::tests::test_rollback ... ok
test hypothesis::canonical_event::tests::test_evidence_ptr_canonical_key ... ok
test evidence::path_safety::tests::test_validate_path_within_root_valid ... ok
test hypothesis::canonical_event::tests::test_evidence_ptr_sha256 ... ok
test hypothesis::canonical_fact::tests::test_fact_creation ... ok
test evidence::path_safety::tests::test_validate_path_within_root_escape ... ok
test hypothesis::canonical_fact::tests::test_fact_store ... ok
test hypothesis::changes::tests::test_change_renderer ... ok
test hypothesis::changes::tests::test_change_tracker ... ok
test hypothesis::changes::tests::test_change_window ... ok
test hypothesis::closure::tests::test_lifecycle_creation ... ok
test hypothesis::closure::tests::test_invalid_transition ... ok
test hypothesis::closure::tests::test_lifecycle_manager ... ok
test hypothesis::changes::tests::test_state_diff ... ok
test hypothesis::closure::tests::test_lifecycle_transitions ... ok
test hypothesis::command_alignment::tests::test_discovery_categorization ... ok
test hypothesis::canonical_event::tests::test_deref_cannot_read_outside_root ... ok
test hypothesis::command_alignment::tests::test_development_categorization ... ok
test hypothesis::command_alignment::tests::test_alignment_computation ... ok
test hypothesis::command_alignment::tests::test_ide_context ... ok
test hypothesis::command_alignment::tests::test_credential_access_categorization ... ok
test hypothesis::command_alignment::tests::test_privesc_categorization ... ok
test hypothesis::copilot::tests::test_ctf_context_detection ... ok
test hypothesis::command_alignment::tests::test_sudo_apt_is_not_privesc ... ok
test hypothesis::copilot::tests::test_question_categorization ... ok
test hypothesis::copilot::tests::test_truncate ... ok
test hypothesis::copilot::tests::test_templated_renderer_basic ... ok
test hypothesis::copilot::tests::test_validation_forbidden_content ... ok
test hypothesis::determinism::tests::test_global_order_key_ordering ... ok
test hypothesis::canonical_event::tests::test_deref_with_valid_segment ... ok
test hypothesis::determinism::tests::test_deterministic_merge ... ok
test hypothesis::determinism::tests::test_stable_key_formulas_hypothesis_id ... ok
test hypothesis::determinism::tests::test_stable_key_formulas_proc_key ... ok
test hypothesis::determinism::tests::test_golden_output_verification ... ok
test hypothesis::disambiguator::tests::test_pivot_action_serialization ... ok
test hypothesis::disambiguator::tests::test_disambiguator_generation ... ok
test hypothesis::explanation::tests::test_claim_creation ... ok
test hypothesis::explanation::tests::test_explanation_builder ... ok
test hypothesis::explanation::tests::test_os_lexicon ... ok
test hypothesis::hardening::tests::test_pivot_validation ... ok
test hypothesis::hardening::tests::test_sensitive_redaction ... ok
test hypothesis::hypothesis_state::tests::test_corroboration_vector ... ok
test hypothesis::hardening::tests::test_question_validation ... ok
test hypothesis::hardening::tests::test_string_sanitization ... ok
test hypothesis::hypothesis_state::tests::test_hypothesis_id_determinism ... ok
test hypothesis::hypothesis_state::tests::test_hypothesis_slot_fill ... ok
test hypothesis::incident::tests::test_incident_from_hypothesis ... ok
test hypothesis::incident::tests::test_incident_id_determinism ... ok
test hypothesis::ordering::tests::test_audit_record_generation ... ok
test hypothesis::incident::tests::test_incident_merge ... ok
test hypothesis::ordering::tests::test_clock_skew_non_monotonic_timestamps ... ok
test hypothesis::ordering::tests::test_event_merger ... ok
test hypothesis::ordering::tests::test_duplicate_event_idempotency ... ok
test hypothesis::ordering::tests::test_event_order_key_ordering ... ok
test hypothesis::ordering::tests::test_future_timestamp_does_not_poison_watermark ... ok
test hypothesis::ordering::tests::test_late_arrival_boundary_exact_grace_limit ... ok
test hypothesis::ordering::tests::test_late_arrival_policy ... ok
test hypothesis::ordering::tests::test_late_enrichment_annotation ... ok
test hypothesis::ordering::tests::test_late_enrichment_stored_and_auditable ... ok
test hypothesis::ordering::tests::test_multi_stream_lag_watermark ... ok
test hypothesis::ordering::tests::test_rejected_events_not_stored ... ok
test hypothesis::ordering::tests::test_verify_ordering ... ok
test hypothesis::promotion::tests::test_promotion_decision ... ok
test hypothesis::promotion::tests::test_severity_calculation ... ok
test hypothesis::promotion::tests::test_maturity_calculation ... ok
test hypothesis::scope_keys::tests::test_file_scope_windows_vs_unix ... ok
test hypothesis::scope_keys::tests::test_proc_scope_key_determinism ... ok
test hypothesis::scope_keys::tests::test_proc_scope_key_pid_reuse ... ok
test hypothesis::scope_keys::tests::test_scope_key_parse_roundtrip ... ok
test hypothesis::scope_keys::tests::test_time_bucket ... ok
test hypothesis::session::tests::test_focus_window ... ok
test hypothesis::session::tests::test_checkpoint ... ok
test hypothesis::session::tests::test_session_creation ... ok
test hypothesis::storage::tests::test_event_query_builder ... ok
test hypothesis::storage::tests::test_hypothesis_query_builder ... ok
test hypothesis::storage::tests::test_in_memory_storage ... ok
test hypothesis::tier0::tests::test_reference_required ... ok
test hypothesis::storage::tests::test_incident_query_builder ... ok
test hypothesis::tier0::tests::test_suppression_allowed_without_tier0 ... ok
test hypothesis::tier0::tests::test_suppression_denied_for_tier0 ... ok
test hypothesis::tier0::tests::test_tier0_invariant_detection ... ok
test hypothesis::versioning::tests::test_compatibility_check ... ok
test hypothesis::versioning::tests::test_component_version ... ok
test hypothesis::versioning::tests::test_content_hash_matching ... ok
test hypothesis::versioning::tests::test_session_config_fingerprint ... ok
test hypothesis::visibility::tests::test_can_claim ... ok
test hypothesis::versioning::tests::test_versioned_explanation ... ok
test hypothesis::visibility::tests::test_collector_confidence ... ok
test hypothesis_controller::tests::test_arbitration_empty ... ok
test hypothesis::versioning::tests::test_version_registry ... ok
test hypothesis::visibility::tests::test_visibility_assessment ... ok
test hypothesis_controller::tests::test_controller_creation ... ok
test hypothesis_controller::tests::test_fact_ingestion ... ok
test os::linux::signal_engine::tests::test_engine_creation ... ok
test os::linux::signal_engine::tests::test_critical_file_write ... ok
test os::macos::signal_engine::tests::test_engine_creation ... ok
test os::linux::signal_engine::tests::test_reverse_shell_detection ... ok
test os::linux::signal_engine::tests::test_kernel_module_detection ... ok
test os::windows::signal_engine::tests::test_engine_creation ... ok
test os::windows::signal_engine::tests::test_should_fire_first_time ... ok
test os::macos::signal_engine::tests::test_persistence_detection ... ok
test os::windows::signals_windows::tests::test_baseline_scores ... ok
test os::macos::signal_engine::tests::test_tcc_bypass_detection ... ok
test os::windows::signals_windows::tests::test_escalated_score ... ok
test os::windows::signals_windows::tests::test_lolbin_abuse_technique ... ok
test os::windows::signals_windows::tests::test_process_injection_chain ... ok
test pipeline::tests::test_log_sink ... ok
test pipeline::tests::test_pipeline_creation ... ok
test pipeline::tests::test_telemetry_input_to_event ... ok
test playbook_loader::tests::test_current_platform ... ok
test playbook_loader::tests::test_platform_playbook_dirs ... ok
test pipeline::tests::test_pipeline_batch_processing ... ok
test pipeline::tests::test_pipeline_with_memory_sink ... ok
test pipeline::tests::test_telemetry_from_json ... ok
test safety::tests::test_data_namespace_conversions ... ok
test safety::tests::test_safe_join_rejects_absolute_path ... ok
test safety::tests::test_safe_join_rejects_parent_dir ... ok
test safety::tests::test_safe_join_rejects_traversal ... ok
test safety::tests::test_zip_safety_policy_allowlist ... ok
test safety::tests::test_zip_safety_policy_default ... ok
test scoring::krim_lite::tests::test_krim_score ... ok
test scoring::elliptic_envelope_lite::tests::test_elliptic_envelope_fit_and_score ... ok
test scoring::mahalanobis::tests::test_mahalanobis_distance ... ok
test scoring::scored_signal::tests::test_scored_signal_from_critical ... ok
test scoring::scoring_engine::tests::test_scoring_engine_disabled ... ok
test signal_orchestrator::tests::test_empty_batch_processing ... ok
test scoring::scoring_engine::tests::test_scoring_engine_enabled ... ok
test signal_orchestrator::tests::test_orchestrator_creation ... ok
test signal_orchestrator::tests::test_platform_specific_orchestrator ... ok
test signal_result::tests::test_different_bucket_different_id ... ok
test signal_result::tests::test_evidence_capping ... ok
test signal_result::tests::test_signal_id_deterministic ... ok
test signal_result::tests::test_signal_result_new ... ok
test safety::tests::test_safe_join_rejects_symlink_escape ... ok
test safety::tests::test_safe_join_with_tempdir ... ok
test signal_persistence::tests::test_persist_signal ... ok
test safety::tests::test_safe_join_normal_path ... ok
test evidence::deref::tests::test_deref_missing ... ok
test evidence::deref::tests::test_deref_from_db ... ok
test evidence::evidence_store::tests::test_store_and_retrieve ... ok
test evidence::deref::tests::test_bulk_deref_stats ... ok
test evidence::evidence_store::tests::test_query_by_time ... ok
test evidence::evidence_store::tests::test_batch_store ... ok

test result: ok. 182 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running unittests src/main.rs (target/debug/deps/edr_locald-69a8cce2c4835fd7)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/credibility_locks_tests.rs (target/debug/deps/credibility_locks_tests-72963f9d60265552)

running 9 tests
test test_include_imported_flag_returns_both ... ok
test test_import_path_construction ... ok
test test_default_queries_exclude_imported ... ok
test test_zip_allows_known_filenames ... ok
test test_data_namespace_isolation ... ok
test test_zip_rejects_path_traversal ... ok
test test_selfcheck_response_structure ... ok
test test_zip_rejects_unknown_filenames ... ok
test test_zip_safety_policy_sizes ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/golden_replay_test.rs (target/debug/deps/golden_replay_test-3c3b85bc5ba8b979)

running 6 tests
test test_timeline_canonical_sort ... ok
test test_event_order_key_determinism ... ok
test test_determinism_with_timestamp_collision ... ok
test test_visibility_state_in_explanation ... ok
test test_explanation_response_json_structure ... ok
test test_explanation_json_determinism ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/hypothesis_wiring_test.rs (target/debug/deps/hypothesis_wiring_test-a67cc7ca09b43b5c)

running 12 tests
test test_evidence_ptr_canonical_key ... ok
test test_controller_default ... ok
test test_hypothesis_controller_creation ... ok
test test_arbitration_runs ... ok
test test_scope_key_variants ... ok
test test_storage_access ... ok
test test_type_exports ... ok
test test_fact_ingestion ... ok
test test_end_to_end_smoke ... ok
test test_late_after_grace_does_not_mutate ... ok
test test_late_within_grace_reopens_or_updates ... ok
test test_subchain_absorbed_visible_and_linked ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/integration_pipeline.rs (target/debug/deps/integration_pipeline-675e01dbf06aa8fa)

running 1 test
test test_agent_to_locald_pipeline ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/pipeline_integration.rs (target/debug/deps/pipeline_integration-c8ce0ab951f9c56b)

running 23 tests
test test_core_event_creation ... ok
test test_event_tag_contains ... ok
test test_baseline_query_new_ip ... ok
test test_linux_signal_engine_integration ... ok
test test_baseline_updater_first_seen ... ok
test test_orchestrator_routes_by_platform ... ok
test test_macos_signal_engine_integration ... ok
test test_baseline_store_operations ... ok
test test_cross_platform_signal_consistency ... ok
test test_evidence_ptr_preserved_through_pipeline ... ok
test test_orchestrator_processes_events ... ok
test test_pipeline_processes_linux_telemetry ... ok
test test_pipeline_processes_macos_telemetry ... ok
test test_pipeline_batch_processing ... ok
test test_pipeline_processes_windows_telemetry ... ok
test test_pipeline_stats_tracking ... ok
test test_scoring_engine_creation ... ok
test test_scoring_engine_scores_signals ... ok
test test_scoring_preserves_signal_data ... ok
test test_severity_ordering ... ok
test test_telemetry_input_converts_to_event ... ok
test test_telemetry_input_from_json ... ok
test test_windows_signal_engine_integration ... ok

test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/lib.rs (target/debug/deps/edr_server-2cf4689b7eafe84c)

running 61 tests
test bundle_exchange::tests::test_imported_namespace_prefix ... ok
test bundle_exchange::tests::test_export_import_recompute_roundtrip ... ok
test bundle_exchange::tests::test_mark_as_imported ... ok
test bundle_exchange::tests::test_recompute_without_section_fails ... ok
test bundle_exchange::tests::test_mark_as_imported_adds_namespace ... ok
test bundle_exchange::tests::test_recompute_returns_diff_structure ... ok
test bundle_exchange::tests::test_export_import_roundtrip_json ... ok
test bundle_exchange::tests::test_export_import_preserves_evidence_ordering ... ok
test bundle_exchange::tests::test_bundle_validation ... ok
test bundle_exchange::tests::test_sha256_checksum_computed ... ok
test golden_bundle::tests::test_all_predefined_scenarios ... ok
test golden_bundle::tests::test_canonical_ordering_validation ... ok
test bundle_exchange::tests::test_zip_rejects_path_traversal ... ok
test bundle_exchange::tests::test_tamper_detection_fails_checksum ... ok
test bundle_exchange::tests::test_export_import_roundtrip_zip ... ok
test bundle_exchange::tests::test_zip_rejects_too_many_files ... ok
test golden_bundle::tests::test_late_arrival_bundle ... ok
test golden_bundle::tests::test_verify_generated_bundle_passes ... ok
test health::tests::test_capture_status_no_heartbeat ... ok
test golden_bundle::tests::test_tampered_bundle_fails ... ok
test golden_bundle::tests::test_generate_golden_bundle ... ok
test health::tests::test_capture_status_with_heartbeat ... ok
test health::tests::test_startup_validation_not_writable ... ok
test health::tests::test_health_check_blocked_db ... ok
test query_isolation::tests::test_default_filter_excludes_imported ... ok
test query_isolation::tests::test_explicit_include_imported ... ok
test golden_bundle::tests::test_visibility_degraded_bundle ... ok
test query_isolation::tests::test_extract_bundle_id ... ok
test query_isolation::tests::test_filter_namespaces ... ok
test query_isolation::tests::test_imported_only ... ok
test query_isolation::tests::test_is_imported_namespace ... ok
test query_isolation::tests::test_make_imported_namespace ... ok
test query_isolation::tests::test_query_source_from_filter ... ok
test health::tests::test_storage_health_writable ... ok
test query_isolation::tests::test_specific_bundle ... ok
test report::tests::test_claim_certainty_categorization ... ok
test report::tests::test_visibility_section ... ok
test report::tests::test_report_bundle_builder ... ok
test health::tests::test_startup_validation_success ... ok
test support_bundle::tests::test_support_bundle_contains_selfcheck ... ok
test support_bundle::tests::test_support_bundle_hashes_validate ... ok
test support_bundle::tests::test_support_bundle_redacts_sensitive_tokens ... ok
test support_bundle::tests::test_support_bundle_size_caps_enforced ... ok
test support_bundle::tests::test_support_bundle_zip_structure_allowlist ... ok
test write_isolation::tests::test_extract_bundle_id ... ok
test write_isolation::tests::test_absolute_path_rejected ... ok
test write_isolation::tests::test_path_escape_rejected ... ok
test write_isolation::tests::test_write_isolation_invalid_bundle_id ... ok
test write_isolation::tests::test_write_in_subdir ... ok
test write_isolation::tests::test_list_files ... ok
test write_isolation::tests::test_remove_sandbox ... ok
test write_isolation::tests::test_write_isolation_valid_bundle_id ... ok
test write_isolation::tests::test_write_within_sandbox ... ok
test health::tests::test_health_check_healthy ... ok
test bundle_exchange::tests::test_redaction_preserves_localhost ... ok
test bundle_exchange::tests::test_redaction_replaces_ips ... ok
test bundle_exchange::tests::test_redaction_replaces_hostnames ... ok
test bundle_exchange::tests::test_redaction_is_deterministic ... ok
test bundle_exchange::tests::test_bundle_sanitization ... ok
test bundle_exchange::tests::test_redacted_bundle_contains_no_raw_sensitive_tokens ... ok
test bundle_exchange::tests::test_redacted_bundle_roundtrip ... ok

test result: ok. 61 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.44s

     Running unittests src/main.rs (target/debug/deps/edr_server-8526c73ce182b68a)

running 70 tests
test bundle_exchange::tests::test_imported_namespace_prefix ... ok
test bundle_exchange::tests::test_mark_as_imported ... ok
test bundle_exchange::tests::test_recompute_returns_diff_structure ... ok
test bundle_exchange::tests::test_recompute_without_section_fails ... ok
test bundle_exchange::tests::test_mark_as_imported_adds_namespace ... ok
test bundle_exchange::tests::test_export_import_recompute_roundtrip ... ok
test bundle_exchange::tests::test_export_import_roundtrip_json ... ok
test bundle_exchange::tests::test_export_import_preserves_evidence_ordering ... ok
test bundle_exchange::tests::test_bundle_validation ... ok
test bundle_exchange::tests::test_sha256_checksum_computed ... ok
test bundle_exchange::tests::test_zip_rejects_path_traversal ... ok
test bundle_exchange::tests::test_tamper_detection_fails_checksum ... ok
test capture_control::tests::test_capture_profile_from_str ... ok
test bundle_exchange::tests::test_export_import_roundtrip_zip ... ok
test capture_control::tests::test_config_mismatch_detection ... ok
test capture_control::tests::test_config_snapshot_roundtrip ... ok
test capture_control::tests::test_profile_config_defaults ... ok
test capture_control::tests::test_reset_counters ... ok
test capture_control::tests::test_deterministic_ordering_preserved ... ok
test capture_control::tests::test_sanity_gate_2_summary_ordering_key ... ok
test capture_control::tests::test_sanity_gate_3_tier0_critical_gap ... ok
test capture_control::tests::test_sanity_gate_3_tier0_warning_message ... ok
test capture_control::tests::test_sanity_gate_4_strict_recompute_validation ... ok
test capture_control::tests::test_sanity_gate_5_visibility_endpoint_is_cheap ... ok
test capture_control::tests::test_throttle_controller_accept ... ok
test capture_control::tests::test_throttle_controller_drop_on_burst ... ok
test capture_control::tests::test_throttle_controller_global_limit ... ok
test capture_control::tests::test_sanity_gate_1_thread_safety_atomics ... ok
test capture_control::tests::test_token_bucket_rate_limiting ... ok
test capture_control::tests::test_tier0_throttle_flag ... ok
test capture_control::tests::test_visibility_degraded_state ... ok
test diagnostics::tests::test_action_details_exist ... ok
test diagnostics::tests::test_missing_reason_detection ... ok
test diagnostics::tests::test_severity_ordering ... ok
test diagnostics::tests::test_verdict_healthy ... ok
test bundle_exchange::tests::test_zip_rejects_too_many_files ... ok
test db::tests::test_document_crud ... ok
test db::tests::test_signal_storage ... ok
test capture_control::tests::test_token_bucket_refill ... ok
test probe::tests::test_expected_streams ... ok
test probe::tests::test_correlate_robust_partial_match ... ok
test probe::tests::test_correlate_events ... ok
test probe::tests::test_correlate_robust_full_match ... ok
test probe::tests::test_correlate_robust_no_match ... ok
test probe::tests::test_probe_runner_temp_file ... ok
test probe::tests::test_probe_spec_defaults ... ok
test probe::tests::test_probe_spec_serde ... ok
test report::tests::test_claim_certainty_categorization ... ok
test report::tests::test_report_bundle_builder ... ok
test report::tests::test_visibility_section ... ok
test support_bundle::tests::test_support_bundle_contains_selfcheck ... ok
test support_bundle::tests::test_support_bundle_hashes_validate ... ok
test support_bundle::tests::test_support_bundle_redacts_sensitive_tokens ... ok
test support_bundle::tests::test_support_bundle_size_caps_enforced ... ok
test support_bundle::tests::test_support_bundle_zip_structure_allowlist ... ok
test verification_pack::tests::test_verification_bundle_has_evidence_excerpts ... ok
test verification_pack::tests::test_verification_bundle_has_required_elements ... ok
test verification_pack::tests::test_verification_bundle_is_deterministic ... ok
test probe::tests::test_probe_runner_process_spawn ... ok
test verification_pack::tests::test_verification_bundle_marked_synthetic ... ok
test probe::tests::test_probe_builds_fingerprints ... ok
test probe::tests::test_probe_runner_localhost_connect ... ok
test probe::tests::test_probe_runner_full ... ok
test bundle_exchange::tests::test_redaction_preserves_localhost ... ok
test bundle_exchange::tests::test_redaction_replaces_ips ... ok
test bundle_exchange::tests::test_redaction_replaces_hostnames ... ok
test bundle_exchange::tests::test_redaction_is_deterministic ... ok
test bundle_exchange::tests::test_bundle_sanitization ... ok
test bundle_exchange::tests::test_redacted_bundle_contains_no_raw_sensitive_tokens ... ok
test bundle_exchange::tests::test_redacted_bundle_roundtrip ... ok

test result: ok. 70 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.44s

     Running unittests src/bin/golden_cli.rs (target/debug/deps/golden_cli-5060032a3167a1f7)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/analyst_workflow_tests.rs (target/debug/deps/analyst_workflow_tests-1d12e7b647cda470)

running 17 tests
test test_apply_disambiguator_result ... ok
test test_checkpoint_summary_structure ... ok
test test_checkpoint_create_list_restore ... ok
test test_change_item_significance_levels ... ok
test test_diff_deterministic_output ... ok
test test_diff_stats_aggregation ... ok
test test_checkpoint_roundtrip ... ok
test test_disambiguator_apply_updates_hypothesis ... ok
test test_diff_view_shows_changes ... ok
test test_disambiguator_structure ... ok
test test_focus_window_deterministic_filtering ... ok
test test_focus_window_request_serialization ... ok
test test_focus_window_response_structure ... ok
test test_focus_window_roundtrip_changes_explain ... ok
test test_visibility_degraded_affects_claims ... ok
test test_visibility_gap_detection ... ok
test test_visibility_health_states ... ok

test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/bundle_exchange_tests.rs (target/debug/deps/bundle_exchange_tests-91930fbb94b1156b)

running 14 tests
test test_bundle_checksum_validation ... ok
test test_bundle_version_compatibility ... ok
test test_no_redaction_preserves_originals ... ok
test test_imported_bundle_marked_correctly ... ok
test test_export_bundle_request_structure ... ok
test test_import_bundle_response_structure ... ok
test test_export_bundle_response_structure ... ok
test test_redaction_determinism ... ok
test test_incident_bundle_format ... ok
test test_redaction_preserves_structure ... ok
test test_zip_magic_bytes_detection ... ok
test test_session_metadata_preserved ... ok
test test_roundtrip_preserves_hypothesis_ordering ... ok
test test_roundtrip_preserves_evidence_references ... ok

test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/capture_control_tests.rs (target/debug/deps/capture_control_tests-a1e43ba27ae0de5c)

running 18 tests
test test_audit_log_for_dropped_events ... ok
test test_byte_rate_limits ... ok
test test_config_snapshot_validation ... ok
test test_full_throttle_flow ... ok
test test_config_snapshot_completeness ... ok
test test_profile_configurations ... ok
test test_priority_class_behavior ... ok
test test_global_rate_limits ... ok
test test_stream_counter_reset ... ok
test test_profile_from_string ... ok
test test_telemetry_throttled_summary ... ok
test test_throttle_decision_variants ... ok
test test_token_bucket_basic_rate_limiting ... ok
test test_token_bucket_refill ... ok
test test_concurrent_token_bucket_access ... ok
test test_unknown_stream_defaults ... ok
test test_visibility_degradation_triggers ... ok
test test_visibility_state_serialization ... ok

test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/credibility_lock_tests.rs (target/debug/deps/credibility_lock_tests-39bb557896a2fa82)

running 22 tests
test test_namespace_isolation_blocks_direct_imported_access ... ok
test test_namespace_isolation_error_message_is_actionable ... ok
test test_e2e_imported_bundle_isolation ... ok
test test_e2e_evidence_deref_traversal_blocked ... ok
test test_namespace_isolation_filters_imported_by_default ... ok
test test_path_safety_allows_valid_paths ... ok
test test_path_safety_blocks_absolute_injection ... ok
test test_e2e_broken_storage_blocked_verdict ... ok
test test_path_safety_blocks_empty ... ok
test test_path_safety_blocks_traversal ... ok
test test_path_safety_errors_are_safe ... ok
test test_selfcheck_blocked_on_critical_issue ... ok
test test_selfcheck_blocked_on_db_failure ... ok
test test_selfcheck_blocked_on_no_critical_events ... ok
test test_selfcheck_blocked_on_segment_dir_failure ... ok
test test_selfcheck_degraded_on_error_issue ... ok
test test_selfcheck_healthy_when_all_ok ... ok
test test_redaction_windows_paths ... ok
test test_redaction_macos_home_paths ... ok
test test_redaction_linux_paths ... ok
test test_redaction_preserves_system_paths ... ok
test test_redaction_multiple_paths ... ok

test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running tests/diagnostics_probe_tests.rs (target/debug/deps/diagnostics_probe_tests-bf1d7e9dc7321134)

running 20 tests
test test_first_run_skips_modal_if_healthy ... ok
test test_admin_requirements ... ok
test test_first_run_shows_modal_if_blocked ... ok
test test_diagnostic_reports_capture_profile ... ok
test test_diagnostic_reports_throttle_state ... ok
test test_missing_reason_descriptions ... ok
test test_issue_severity_ordering ... ok
test test_non_first_run_no_auto_modal ... ok
test test_os_specific_actions_defined ... ok
test test_probe_action_expected_streams ... ok
test test_probe_spec_defaults ... ok
test test_probe_timeout_bounded ... ok
test test_probe_result_structure ... ok
test test_response_deterministic_structure ... ok
test test_stream_diagnostic_schema ... ok
test test_verdict_blocked_on_no_critical_streams ... ok
test test_selfcheck_response_schema_stability ... ok
test test_verdict_blocked_on_storage_failure ... ok
test test_verdict_degraded_on_critical_gap ... ok
test test_verdict_healthy_when_all_good ... ok

test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/entity_extraction_tests.rs (target/debug/deps/entity_extraction_tests-c3b6eef5709a4b81)

running 3 tests
test test_entity_response_structure ... ok
test test_entity_extraction_all_types_present ... ok
test test_entity_extraction_stable_ordering ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/pdf_report_tests.rs (target/debug/deps/pdf_report_tests-03433e65945f3023)

running 15 tests
test test_e2e_live_server ... ignored, Optional: requires running server at localhost:3000
test test_fixture_has_claim_with_evidence ... ok
test test_fixture_has_missing_stream ... ok
test test_fixture_has_timeline_with_late_arrival ... ok
test test_fixture_has_disambiguator ... ok
test test_fixture_has_one_hypothesis ... ok
test test_pdf_minimal_request ... ok
test test_pdf_body_size_threshold ... ok
test test_renderer_handles_minimal_bundle ... ok
test test_pdf_magic_header ... ok
test test_pdf_endpoint_content_type ... ok
test test_pdf_endpoint_returns_200 ... ok
test test_pdf_content_disposition ... ok
test test_renderer_produces_valid_pdf ... ok
test test_pdf_stable_output ... ok

test result: ok. 14 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 5.46s

     Running tests/support_bundle_tests.rs (target/debug/deps/support_bundle_tests-c78a911924a2e20e)

running 5 tests
test test_support_bundle_size_caps_enforced ... ok
test test_support_bundle_contains_selfcheck ... ok
test test_support_bundle_redacts_sensitive_tokens ... ok
test test_support_bundle_hashes_validate ... ok
test test_support_bundle_zip_structure_allowlist ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/verification_tests.rs (target/debug/deps/verification_tests-426cb81155db033f)

running 12 tests
test test_self_check_recommends_verification_when_no_events ... ok
test test_self_check_response_structure ... ok
test test_app_state_response_structure ... ok
test test_pdf_verification_pack_watermark ... ok
test test_setup_complete_response_structure ... ok
test test_setup_request_validation ... ok
test test_verification_bundle_deterministic ... ok
test test_report_bundle_pdf_ready ... ok
test test_verification_reset_response_structure ... ok
test test_wizard_flow_sequence ... ok
test test_verification_load_response_structure ... ok
test test_first_run_marker_logic ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/lib.rs (target/debug/deps/integration-ec4388e38fad2092)

running 13 tests
test export::tests::test_bundle_creation ... ok
test adaptive_report::tests::test_report_builder ... ok
test export::tests::test_export_config_default ... ok
test correlation::tests::test_correlation_engine ... ok
test capabilities::tests::test_registry_confidence ... ok
test capabilities::tests::test_manifest_creation ... ok
test fact::tests::test_fact_creation ... ok
test fact::tests::test_incident_confidence_calculation ... ok
test export_pdf::tests::test_strip_markdown ... ok
test fact::tests::test_incident_builder ... ok
test capabilities::tests::test_coverage_calculation ... ok
test mapping::tests::test_severity_mapping ... ok
test mapping::tests::test_primitive_detection ... ok

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

     Running unittests src/lib.rs (target/debug/deps/workbench-6aa44c98d6f3877d)

running 9 tests
test api::tests::test_get_technique ... ok
test document::tests::test_editable_text_user_override ... ok
test document::tests::test_document_creation ... ok
test document::tests::test_timeline_ordering ... ok
test session::tests::test_session_lifecycle ... ok
test export::tests::test_markdown_export ... ok
test session::tests::test_markers ... ok
test api::tests::test_search_techniques ... ok
test export::tests::test_html_export ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests agent_linux

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests agent_macos

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests agent_windows

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests edr_core

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests edr_locald

running 2 tests
test crates/locald/src/lib.rs - (line 27) ... ignored
test crates/locald/src/safety.rs - safety::safe_join_under (line 44) ... ignored

test result: ok. 0 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests edr_server

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests integration

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests workbench

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

[0;32m✓ Gate 3: Unit tests OK[0m

[1;33m[4/5] Running doc tests...[0m
warning: unused import: `super::session::*`
 --> crates/workbench/src/api.rs:6:5
  |
6 | use super::session::*;
  |     ^^^^^^^^^^^^^^^^^
  |
  = note: `#[warn(unused_imports)]` on by default

warning: `workbench` (lib) generated 1 warning (run `cargo fix --lib -p workbench` to apply 1 suggestion)
warning: unused import: `std::thread`
   --> crates/agent-macos/src/capture_macos_rotating.rs:145:5
    |
145 | use std::thread;
    |     ^^^^^^^^^^^
    |
    = note: `#[warn(unused_imports)]` on by default

warning: unused import: `std::io::Read`
  --> crates/agent-macos/src/platform/bsm_parser.rs:13:5
   |
13 | use std::io::Read;
   |     ^^^^^^^^^^^^^

warning: unused imports: `SystemTime` and `UNIX_EPOCH`
 --> crates/agent-macos/src/platform/facts.rs:4:17
  |
4 | use std::time::{SystemTime, UNIX_EPOCH};
  |                 ^^^^^^^^^^  ^^^^^^^^^^

warning: unused import: `std::collections::HashMap`
 --> crates/agent-macos/src/platform/macos_fact_extractors.rs:4:5
  |
4 | use std::collections::HashMap;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `MacOSSensorMode`
 --> crates/agent-macos/src/platform/macos_fact_extractors_es.rs:5:42
  |
5 | use super::macos_fact_extractors::{Fact, MacOSSensorMode};
  |                                          ^^^^^^^^^^^^^^^

warning: unused import: `super::bsm_tokens`
 --> crates/agent-macos/src/sensors/bsm/bsm_cred_access.rs:5:5
  |
5 | use super::bsm_tokens;
  |     ^^^^^^^^^^^^^^^^^

warning: unused import: `edr_core::Event`
 --> crates/agent-macos/src/sensors/bsm/bsm_exec.rs:7:5
  |
7 | use edr_core::Event;
  |     ^^^^^^^^^^^^^^^

warning: unused import: `EvidencePtr`
 --> crates/agent-macos/src/sensors/bsm/bsm_fs_ops.rs:6:23
  |
6 | use edr_core::{Event, EvidencePtr};
  |                       ^^^^^^^^^^^

warning: unused import: `crate::sensors::hash_keys`
 --> crates/agent-macos/src/sensors/es/file_metadata.rs:4:5
  |
4 | use crate::sensors::hash_keys;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `crate::sensors::hash_keys`
 --> crates/agent-macos/src/sensors/es/file_rename.rs:4:5
  |
4 | use crate::sensors::hash_keys;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `crate::sensors::hash_keys`
 --> crates/agent-macos/src/sensors/es/file_unlink.rs:4:5
  |
4 | use crate::sensors::hash_keys;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `std::collections::HashMap`
 --> crates/agent-macos/src/sensors/es/identity.rs:5:5
  |
5 | use std::collections::HashMap;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^

warning: variable does not need to be mutable
    --> crates/agent-macos/src/capture_macos_rotating.rs:1057:9
     |
1057 |     let mut segment_index = 0u32; // Track raw segment count for canonical indexing
     |         ----^^^^^^^^^^^^^
     |         |
     |         help: remove this `mut`
     |
     = note: `#[warn(unused_mut)]` on by default

warning: variable does not need to be mutable
    --> crates/agent-macos/src/capture_macos_rotating.rs:1059:9
     |
1059 |     let mut parse_failed_total = 0u64; // Count of malformed records
     |         ----^^^^^^^^^^^^^^^^^^
     |         |
     |         help: remove this `mut`

warning: variable does not need to be mutable
    --> crates/agent-macos/src/capture_macos_rotating.rs:1060:9
     |
1060 |     let mut parse_failed_malformed = 0u64; // Records exceeding size limit or corrupted
     |         ----^^^^^^^^^^^^^^^^^^^^^^
     |         |
     |         help: remove this `mut`

warning: variable does not need to be mutable
    --> crates/agent-macos/src/capture_macos_rotating.rs:1061:9
     |
1061 |     let mut parse_failed_unsupported = 0u64; // Unrecognized AUE codes or missing token types
     |         ----^^^^^^^^^^^^^^^^^^^^^^^^
     |         |
     |         help: remove this `mut`

warning: variable does not need to be mutable
    --> crates/agent-macos/src/capture_macos_rotating.rs:1062:9
     |
1062 |     let mut parse_failed_validation = 0u64; // Field validation failures (invalid pid, uid, etc.)
     |         ----^^^^^^^^^^^^^^^^^^^^^^^
     |         |
     |         help: remove this `mut`

warning: variable does not need to be mutable
    --> crates/agent-macos/src/capture_macos_rotating.rs:1128:35
     |
1128 |                         if let Ok(mut index) = load_or_create_index(&index_path) {
     |                                   ----^^^^^
     |                                   |
     |                                   help: remove this `mut`

warning: variable does not need to be mutable
    --> crates/agent-macos/src/capture_macos_rotating.rs:1167:21
     |
1167 |                 let mut parse_errors_batch = 0u64;
     |                     ----^^^^^^^^^^^^^^^^^^
     |                     |
     |                     help: remove this `mut`

warning: variable does not need to be mutable
    --> crates/agent-macos/src/capture_macos_rotating.rs:1304:31
     |
1304 |                     if let Ok(mut index) = load_or_create_index(&index_path) {
     |                               ----^^^^^
     |                               |
     |                               help: remove this `mut`

warning: value assigned to `path_buffer` is never read
   --> crates/agent-macos/src/platform/bsm_parser.rs:268:17
    |
268 |         let mut path_buffer = String::new();
    |                 ^^^^^^^^^^^
    |
    = help: maybe it is overwritten before being read?
    = note: `#[warn(unused_assignments)]` on by default

warning: unused variable: `segment_id`
  --> crates/agent-macos/src/sensors/bsm/bsm_archive_tool_exec.rs:23:5
   |
23 |     segment_id: &str,
   |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`
   |
   = note: `#[warn(unused_variables)]` on by default

warning: unused variable: `record_index`
  --> crates/agent-macos/src/sensors/bsm/bsm_archive_tool_exec.rs:24:5
   |
24 |     record_index: usize,
   |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
  --> crates/agent-macos/src/sensors/bsm/bsm_auth_event.rs:27:5
   |
27 |     segment_id: &str,
   |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
  --> crates/agent-macos/src/sensors/bsm/bsm_auth_event.rs:28:5
   |
28 |     record_index: usize,
   |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
  --> crates/agent-macos/src/sensors/bsm/bsm_auth_event.rs:98:5
   |
98 |     segment_id: &str,
   |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
  --> crates/agent-macos/src/sensors/bsm/bsm_auth_event.rs:99:5
   |
99 |     record_index: usize,
   |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
  --> crates/agent-macos/src/sensors/bsm/bsm_cred_access.rs:26:5
   |
26 |     segment_id: &str,
   |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
  --> crates/agent-macos/src/sensors/bsm/bsm_cred_access.rs:27:5
   |
27 |     record_index: usize,
   |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
  --> crates/agent-macos/src/sensors/bsm/bsm_defense_evasion.rs:52:5
   |
52 |     segment_id: &str,
   |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
  --> crates/agent-macos/src/sensors/bsm/bsm_defense_evasion.rs:53:5
   |
53 |     record_index: usize,
   |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
  --> crates/agent-macos/src/sensors/bsm/bsm_defense_evasion.rs:99:5
   |
99 |     segment_id: &str,
   |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
   --> crates/agent-macos/src/sensors/bsm/bsm_defense_evasion.rs:100:5
    |
100 |     record_index: usize,
    |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
  --> crates/agent-macos/src/sensors/bsm/bsm_discovery_exec.rs:23:5
   |
23 |     segment_id: &str,
   |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
  --> crates/agent-macos/src/sensors/bsm/bsm_discovery_exec.rs:24:5
   |
24 |     record_index: usize,
   |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
  --> crates/agent-macos/src/sensors/bsm/bsm_file_ops.rs:61:5
   |
61 |     segment_id: &str,
   |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
  --> crates/agent-macos/src/sensors/bsm/bsm_file_ops.rs:62:5
   |
62 |     record_index: usize,
   |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
   --> crates/agent-macos/src/sensors/bsm/bsm_file_ops.rs:116:5
    |
116 |     segment_id: &str,
    |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
   --> crates/agent-macos/src/sensors/bsm/bsm_file_ops.rs:117:5
    |
117 |     record_index: usize,
    |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
   --> crates/agent-macos/src/sensors/bsm/bsm_file_ops.rs:167:5
    |
167 |     segment_id: &str,
    |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
   --> crates/agent-macos/src/sensors/bsm/bsm_file_ops.rs:168:5
    |
168 |     record_index: usize,
    |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
   --> crates/agent-macos/src/sensors/bsm/bsm_file_ops.rs:213:5
    |
213 |     segment_id: &str,
    |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
   --> crates/agent-macos/src/sensors/bsm/bsm_file_ops.rs:214:5
    |
214 |     record_index: usize,
    |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
   --> crates/agent-macos/src/sensors/bsm/bsm_file_ops.rs:271:5
    |
271 |     segment_id: &str,
    |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
   --> crates/agent-macos/src/sensors/bsm/bsm_file_ops.rs:272:5
    |
272 |     record_index: usize,
    |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
   --> crates/agent-macos/src/sensors/bsm/bsm_file_ops.rs:327:5
    |
327 |     segment_id: &str,
    |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
   --> crates/agent-macos/src/sensors/bsm/bsm_file_ops.rs:328:5
    |
328 |     record_index: usize,
    |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
   --> crates/agent-macos/src/sensors/bsm/bsm_file_ops.rs:387:5
    |
387 |     segment_id: &str,
    |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
   --> crates/agent-macos/src/sensors/bsm/bsm_file_ops.rs:388:5
    |
388 |     record_index: usize,
    |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `exe_path`
   --> crates/agent-macos/src/sensors/bsm/bsm_file_ops.rs:474:5
    |
474 |     exe_path: Option<&str>,
    |     ^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_exe_path`

warning: unused variable: `success`
   --> crates/agent-macos/src/sensors/bsm/bsm_file_ops.rs:475:5
    |
475 |     success: bool,
    |     ^^^^^^^ help: if this is intentional, prefix it with an underscore: `_success`

warning: unused variable: `segment_id`
  --> crates/agent-macos/src/sensors/bsm/bsm_fs_ops.rs:14:5
   |
14 |     segment_id: String,
   |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
  --> crates/agent-macos/src/sensors/bsm/bsm_fs_ops.rs:15:5
   |
15 |     record_index: usize,
   |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
  --> crates/agent-macos/src/sensors/bsm/bsm_fs_ops.rs:63:5
   |
63 |     segment_id: String,
   |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
  --> crates/agent-macos/src/sensors/bsm/bsm_fs_ops.rs:64:5
   |
64 |     record_index: usize,
   |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
   --> crates/agent-macos/src/sensors/bsm/bsm_fs_ops.rs:112:5
    |
112 |     segment_id: String,
    |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
   --> crates/agent-macos/src/sensors/bsm/bsm_fs_ops.rs:113:5
    |
113 |     record_index: usize,
    |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
  --> crates/agent-macos/src/sensors/bsm/bsm_net_connect.rs:16:5
   |
16 |     segment_id: &str,
   |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
  --> crates/agent-macos/src/sensors/bsm/bsm_net_connect.rs:17:5
   |
17 |     record_index: usize,
   |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
  --> crates/agent-macos/src/sensors/bsm/bsm_network.rs:75:5
   |
75 |     segment_id: &str,
   |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
  --> crates/agent-macos/src/sensors/bsm/bsm_network.rs:76:5
   |
76 |     record_index: usize,
   |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
   --> crates/agent-macos/src/sensors/bsm/bsm_network.rs:157:5
    |
157 |     segment_id: &str,
    |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
   --> crates/agent-macos/src/sensors/bsm/bsm_network.rs:158:5
    |
158 |     record_index: usize,
    |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
   --> crates/agent-macos/src/sensors/bsm/bsm_network.rs:221:5
    |
221 |     segment_id: &str,
    |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
   --> crates/agent-macos/src/sensors/bsm/bsm_network.rs:222:5
    |
222 |     record_index: usize,
    |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `success`
   --> crates/agent-macos/src/sensors/bsm/bsm_network.rs:419:5
    |
419 |     success: bool,
    |     ^^^^^^^ help: if this is intentional, prefix it with an underscore: `_success`

warning: unused variable: `segment_id`
  --> crates/agent-macos/src/sensors/bsm/bsm_persistence_change.rs:38:5
   |
38 |     segment_id: &str,
   |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
  --> crates/agent-macos/src/sensors/bsm/bsm_persistence_change.rs:39:5
   |
39 |     record_index: usize,
   |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
  --> crates/agent-macos/src/sensors/bsm/bsm_persistence_change.rs:95:5
   |
95 |     segment_id: &str,
   |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
  --> crates/agent-macos/src/sensors/bsm/bsm_persistence_change.rs:96:5
   |
96 |     record_index: usize,
   |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
  --> crates/agent-macos/src/sensors/bsm/bsm_priv_escalation.rs:44:5
   |
44 |     segment_id: &str,
   |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
  --> crates/agent-macos/src/sensors/bsm/bsm_priv_escalation.rs:45:5
   |
45 |     record_index: usize,
   |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
   --> crates/agent-macos/src/sensors/bsm/bsm_priv_escalation.rs:124:5
    |
124 |     segment_id: &str,
    |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
   --> crates/agent-macos/src/sensors/bsm/bsm_priv_escalation.rs:125:5
    |
125 |     record_index: usize,
    |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
   --> crates/agent-macos/src/sensors/bsm/bsm_priv_escalation.rs:205:5
    |
205 |     segment_id: &str,
    |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
   --> crates/agent-macos/src/sensors/bsm/bsm_priv_escalation.rs:206:5
    |
206 |     record_index: usize,
    |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
   --> crates/agent-macos/src/sensors/bsm/bsm_priv_escalation.rs:285:5
    |
285 |     segment_id: &str,
    |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
   --> crates/agent-macos/src/sensors/bsm/bsm_priv_escalation.rs:286:5
    |
286 |     record_index: usize,
    |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
   --> crates/agent-macos/src/sensors/bsm/bsm_priv_escalation.rs:358:5
    |
358 |     segment_id: &str,
    |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
   --> crates/agent-macos/src/sensors/bsm/bsm_priv_escalation.rs:359:5
    |
359 |     record_index: usize,
    |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
   --> crates/agent-macos/src/sensors/bsm/bsm_priv_escalation.rs:448:5
    |
448 |     segment_id: &str,
    |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
   --> crates/agent-macos/src/sensors/bsm/bsm_priv_escalation.rs:449:5
    |
449 |     record_index: usize,
    |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
   --> crates/agent-macos/src/sensors/bsm/bsm_priv_escalation.rs:530:5
    |
530 |     segment_id: &str,
    |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
   --> crates/agent-macos/src/sensors/bsm/bsm_priv_escalation.rs:531:5
    |
531 |     record_index: usize,
    |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
  --> crates/agent-macos/src/sensors/bsm/bsm_process_injection.rs:35:5
   |
35 |     segment_id: &str,
   |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
  --> crates/agent-macos/src/sensors/bsm/bsm_process_injection.rs:36:5
   |
36 |     record_index: usize,
   |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
   --> crates/agent-macos/src/sensors/bsm/bsm_process_injection.rs:142:5
    |
142 |     segment_id: &str,
    |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
   --> crates/agent-macos/src/sensors/bsm/bsm_process_injection.rs:143:5
    |
143 |     record_index: usize,
    |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `host`
  --> crates/agent-macos/src/sensors/bsm/bsm_reader.rs:30:9
   |
30 |         host: &str,
   |         ^^^^ help: if this is intentional, prefix it with an underscore: `_host`

warning: unused variable: `stream_id`
  --> crates/agent-macos/src/sensors/bsm/bsm_reader.rs:31:9
   |
31 |         stream_id: &str,
   |         ^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_stream_id`

warning: unused variable: `segment_id`
  --> crates/agent-macos/src/sensors/bsm/bsm_reader.rs:32:9
   |
32 |         segment_id: &str,
   |         ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
  --> crates/agent-macos/src/sensors/bsm/bsm_reader.rs:33:9
   |
33 |         record_index: usize,
   |         ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
  --> crates/agent-macos/src/sensors/bsm/bsm_script_exec.rs:63:5
   |
63 |     segment_id: &str,
   |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
  --> crates/agent-macos/src/sensors/bsm/bsm_script_exec.rs:64:5
   |
64 |     record_index: usize,
   |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
   --> crates/agent-macos/src/sensors/bsm/bsm_script_exec.rs:201:5
    |
201 |     segment_id: &str,
    |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
   --> crates/agent-macos/src/sensors/bsm/bsm_script_exec.rs:202:5
    |
202 |     record_index: usize,
    |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `segment_id`
  --> crates/agent-macos/src/sensors/bsm/bsm_staging_write.rs:19:5
   |
19 |     segment_id: &str,
   |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
  --> crates/agent-macos/src/sensors/bsm/bsm_staging_write.rs:20:5
   |
20 |     record_index: usize,
   |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `host`
  --> crates/agent-macos/src/sensors/bsm/mod.rs:30:5
   |
30 |     host: String,
   |     ^^^^ help: if this is intentional, prefix it with an underscore: `_host`

warning: unused variable: `stream_id`
  --> crates/agent-macos/src/sensors/bsm/mod.rs:31:5
   |
31 |     stream_id: String,
   |     ^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_stream_id`

warning: unused variable: `segment_id`
  --> crates/agent-macos/src/sensors/bsm/mod.rs:32:5
   |
32 |     segment_id: String,
   |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_segment_id`

warning: unused variable: `record_index`
  --> crates/agent-macos/src/sensors/bsm/mod.rs:33:5
   |
33 |     record_index: usize,
   |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_index`

warning: unused variable: `record_data`
  --> crates/agent-macos/src/sensors/bsm/mod.rs:34:5
   |
34 |     record_data: Vec<u8>,
   |     ^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_record_data`

warning: unused variable: `ts_millis`
  --> crates/agent-macos/src/sensors/bsm/mod.rs:35:5
   |
35 |     ts_millis: u64,
   |     ^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_ts_millis`

warning: unused variable: `event_data`
  --> crates/agent-macos/src/sensors/es/file_create.rs:14:5
   |
14 |     event_data: Vec<u8>,
   |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_event_data`

warning: unused variable: `event_data`
  --> crates/agent-macos/src/sensors/es/file_metadata.rs:14:5
   |
14 |     event_data: Vec<u8>,
   |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_event_data`

warning: unused variable: `event_data`
  --> crates/agent-macos/src/sensors/es/file_open.rs:14:5
   |
14 |     event_data: Vec<u8>,
   |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_event_data`

warning: unused variable: `event_data`
  --> crates/agent-macos/src/sensors/es/file_rename.rs:14:5
   |
14 |     event_data: Vec<u8>,
   |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_event_data`

warning: unused variable: `event_data`
  --> crates/agent-macos/src/sensors/es/file_unlink.rs:14:5
   |
14 |     event_data: Vec<u8>,
   |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_event_data`

warning: unused variable: `event_data`
  --> crates/agent-macos/src/sensors/es/file_write.rs:14:5
   |
14 |     event_data: Vec<u8>,
   |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_event_data`

warning: unused variable: `es_event_data`
  --> crates/agent-macos/src/sensors/es/identity.rs:63:5
   |
63 |     es_event_data: &[u8],
   |     ^^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_es_event_data`

warning: unused variable: `stream_id`
  --> crates/agent-macos/src/sensors/es/identity.rs:64:5
   |
64 |     stream_id: String,
   |     ^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_stream_id`

warning: struct `BSMParser` is never constructed
  --> crates/agent-macos/src/capture_macos_rotating.rs:10:8
   |
10 | struct BSMParser {
   |        ^^^^^^^^^
   |
   = note: `#[warn(dead_code)]` on by default

warning: constant `BSM_HEADER_TOKEN` is never used
  --> crates/agent-macos/src/capture_macos_rotating.rs:15:7
   |
15 | const BSM_HEADER_TOKEN: u8 = 0x14; // AUT_HEADER32
   |       ^^^^^^^^^^^^^^^^

warning: constant `BSM_HEADER64_TOKEN` is never used
  --> crates/agent-macos/src/capture_macos_rotating.rs:16:7
   |
16 | const BSM_HEADER64_TOKEN: u8 = 0x74; // AUT_HEADER64
   |       ^^^^^^^^^^^^^^^^^^

warning: constant `BSM_TRAILER_TOKEN` is never used
  --> crates/agent-macos/src/capture_macos_rotating.rs:17:7
   |
17 | const BSM_TRAILER_TOKEN: u8 = 0x13; // AUT_TRAILER
   |       ^^^^^^^^^^^^^^^^^

warning: associated items `new`, `feed`, and `try_parse_record` are never used
  --> crates/agent-macos/src/capture_macos_rotating.rs:20:8
   |
19 | impl BSMParser {
   | -------------- associated items in this implementation
20 |     fn new() -> Self {
   |        ^^^
...
27 |     fn feed(&mut self, data: &[u8]) {
   |        ^^^^
...
40 |     fn try_parse_record(&mut self) -> Option<(i32, Vec<u8>, u64, usize)> {
   |        ^^^^^^^^^^^^^^^^

warning: constant `MAX_RECORDS_PER_POLL` is never used
   --> crates/agent-macos/src/capture_macos_rotating.rs:150:7
    |
150 | const MAX_RECORDS_PER_POLL: usize = 1000; // Max records to read in one poll
    |       ^^^^^^^^^^^^^^^^^^^^

warning: constant `MAX_BYTES_PER_RECORD` is never used
   --> crates/agent-macos/src/capture_macos_rotating.rs:151:7
    |
151 | const MAX_BYTES_PER_RECORD: usize = 16384; // Max 16KB per record
    |       ^^^^^^^^^^^^^^^^^^^^

warning: constant `MAX_BATCH_BYTES` is never used
   --> crates/agent-macos/src/capture_macos_rotating.rs:152:7
    |
152 | const MAX_BATCH_BYTES: usize = 10 * 1024 * 1024; // Max 10MB per batch
    |       ^^^^^^^^^^^^^^^

warning: function `get_telemetry_root` is never used
   --> crates/agent-macos/src/capture_macos_rotating.rs:154:4
    |
154 | fn get_telemetry_root() -> Result<PathBuf, String> {
    |    ^^^^^^^^^^^^^^^^^^

warning: function `get_max_segment_bytes` is never used
   --> crates/agent-macos/src/capture_macos_rotating.rs:174:4
    |
174 | fn get_max_segment_bytes() -> u64 {
    |    ^^^^^^^^^^^^^^^^^^^^^

warning: function `hostname` is never used
   --> crates/agent-macos/src/capture_macos_rotating.rs:242:4
    |
242 | fn hostname() -> String {
    |    ^^^^^^^^

warning: function `apply_evidence_ptr_ownership` is never used
   --> crates/agent-macos/src/capture_macos_rotating.rs:252:4
    |
252 | fn apply_evidence_ptr_ownership(
    |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `event_to_normalized` is never used
   --> crates/agent-macos/src/capture_macos_rotating.rs:274:4
    |
274 | fn event_to_normalized(host: &str, event: &Event) -> NormalizedEvent {
    |    ^^^^^^^^^^^^^^^^^^^

warning: function `setup_signal_handler` is never used
   --> crates/agent-macos/src/capture_macos_rotating.rs:369:4
    |
369 | fn setup_signal_handler(shutdown_flag: Arc<AtomicBool>) {
    |    ^^^^^^^^^^^^^^^^^^^^

warning: function `now_ms` is never used
   --> crates/agent-macos/src/capture_macos_rotating.rs:379:4
    |
379 | fn now_ms() -> u64 {
    |    ^^^^^^

warning: function `load_or_create_index` is never used
   --> crates/agent-macos/src/capture_macos_rotating.rs:386:4
    |
386 | fn load_or_create_index(index_path: &PathBuf) -> Result<SegmentIndex, String> {
    |    ^^^^^^^^^^^^^^^^^^^^

warning: function `validate_index` is never used
   --> crates/agent-macos/src/capture_macos_rotating.rs:404:4
    |
404 | fn validate_index(index: &SegmentIndex, segments_dir: &PathBuf) -> Result<(), String> {
    |    ^^^^^^^^^^^^^^

warning: function `write_segment_atomic` is never used
   --> crates/agent-macos/src/capture_macos_rotating.rs:429:4
    |
429 | fn write_segment_atomic(
    |    ^^^^^^^^^^^^^^^^^^^^

warning: function `compute_sha256_file` is never used
   --> crates/agent-macos/src/capture_macos_rotating.rs:507:4
    |
507 | fn compute_sha256_file(path: &PathBuf) -> Result<String, String> {
    |    ^^^^^^^^^^^^^^^^^^^

warning: function `compute_sha256_string` is never used
   --> crates/agent-macos/src/capture_macos_rotating.rs:524:4
    |
524 | fn compute_sha256_string(data: &str) -> String {
    |    ^^^^^^^^^^^^^^^^^^^^^

warning: function `update_index_atomic` is never used
   --> crates/agent-macos/src/capture_macos_rotating.rs:530:4
    |
530 | fn update_index_atomic(
    |    ^^^^^^^^^^^^^^^^^^^

warning: function `validate_root_permissions` is never used
   --> crates/agent-macos/src/capture_macos_rotating.rs:601:4
    |
601 | fn validate_root_permissions() -> Result<(), String> {
    |    ^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `run_capture_with_mock` is never used
   --> crates/agent-macos/src/capture_macos_rotating.rs:608:4
    |
608 | fn run_capture_with_mock(fixture_path: &PathBuf) {
    |    ^^^^^^^^^^^^^^^^^^^^^

warning: function `cleanup_stale_temp_files` is never used
   --> crates/agent-macos/src/capture_macos_rotating.rs:697:4
    |
697 | fn cleanup_stale_temp_files(segments_dir: &PathBuf) {
    |    ^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `get_build_id` is never used
   --> crates/agent-macos/src/capture_macos_rotating.rs:725:4
    |
725 | fn get_build_id() -> String {
    |    ^^^^^^^^^^^^

warning: function `write_heartbeat` is never used
   --> crates/agent-macos/src/capture_macos_rotating.rs:730:4
    |
730 | fn write_heartbeat(
    |    ^^^^^^^^^^^^^^^

warning: function `macos_dedup_key` is never used
   --> crates/agent-macos/src/capture_macos_rotating.rs:805:4
    |
805 | fn macos_dedup_key(event: &edr_core::Event) -> Option<String> {
    |    ^^^^^^^^^^^^^^^

warning: function `main` is never used
   --> crates/agent-macos/src/capture_macos_rotating.rs:951:4
    |
951 | fn main() {
    |    ^^^^

warning: field `pos` is never read
   --> crates/agent-macos/src/platform/bsm_parser.rs:177:5
    |
175 | pub struct BSMParser {
    |            --------- field in this struct
176 |     buffer: Vec<u8>,
177 |     pos: usize,
    |     ^^^

warning: constant `PRIVILEGED_UIDS` is never used
  --> crates/agent-macos/src/sensors/bsm/bsm_priv_escalation.rs:26:7
   |
26 | const PRIVILEGED_UIDS: &[u32] = &[
   |       ^^^^^^^^^^^^^^^

warning: function `read_u16_le` is never used
  --> crates/agent-macos/src/sensors/bsm/bsm_tokens.rs:92:4
   |
92 | fn read_u16_le(data: &[u8], offset: usize) -> Option<u16> {
   |    ^^^^^^^^^^^

warning: function `estimate_token_len_with_data` is never used
   --> crates/agent-macos/src/sensors/bsm/bsm_tokens.rs:397:4
    |
397 | fn estimate_token_len_with_data(data: &[u8]) -> Option<usize> {
    |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: `agent-macos` (lib) generated 144 warnings (run `cargo fix --lib -p agent-macos` to apply 20 suggestions)
warning: unused import: `Path`
 --> crates/locald/src/baseline/baseline_store.rs:7:17
  |
7 | use std::path::{Path, PathBuf};
  |                 ^^^^
  |
  = note: `#[warn(unused_imports)]` on by default

warning: unused imports: `DMatrix` and `DVector`
 --> crates/locald/src/scoring/mahalanobis.rs:4:16
  |
4 | use nalgebra::{DMatrix, DVector};
  |                ^^^^^^^  ^^^^^^^

warning: unused import: `std::path::Path`
 --> crates/locald/src/signal_persistence.rs:7:5
  |
7 | use std::path::Path;
  |     ^^^^^^^^^^^^^^^

warning: unused import: `StoredRecord`
  --> crates/locald/src/evidence/deref.rs:12:58
   |
12 | use super::evidence_store::{EvidenceStore, RecordSource, StoredRecord};
   |                                                          ^^^^^^^^^^^^

warning: unused imports: `SeekFrom` and `Seek`
  --> crates/locald/src/evidence/deref.rs:16:35
   |
16 | use std::io::{BufRead, BufReader, Seek, SeekFrom};
   |                                   ^^^^  ^^^^^^^^

warning: unused import: `EvidenceIntegrityError`
 --> crates/locald/src/evidence/evidence_store.rs:8:27
  |
8 | use super::evidence_ptr::{EvidenceIntegrityError, EvidencePtr};
  |                           ^^^^^^^^^^^^^^^^^^^^^^

warning: unused imports: `AnalystAction`, `AssertionType`, `Assertion`, `FocusWindow`, and `Session`
  --> crates/locald/src/../../../locald/hypothesis/api.rs:13:22
   |
13 | use super::session::{AnalystAction, Assertion, AssertionType, FocusWindow, Session, SessionMode};
   |                      ^^^^^^^^^^^^^  ^^^^^^^^^  ^^^^^^^^^^^^^  ^^^^^^^^^^^  ^^^^^^^

warning: unused import: `calculate_maturity`
 --> crates/locald/src/../../../locald/hypothesis/arbitration.rs:7:46
  |
7 | use super::promotion::{calculate_confidence, calculate_maturity, Severity};
  |                                              ^^^^^^^^^^^^^^^^^^

warning: unused import: `HashMap`
  --> crates/locald/src/../../../locald/hypothesis/arbitration.rs:11:24
   |
11 | use std::collections::{HashMap, HashSet};
   |                        ^^^^^^^

warning: unused import: `super::arbitration::ArbitrationResponse`
 --> crates/locald/src/../../../locald/hypothesis/copilot.rs:7:5
  |
7 | use super::arbitration::ArbitrationResponse;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused imports: `ExplanationVisibilityState`, `MissingEvidence`, `QueryContext`, and `SlotStatusSummary`
  --> crates/locald/src/../../../locald/hypothesis/copilot.rs:11:49
   |
11 |     Claim, ClaimCertainty, ExplanationResponse, ExplanationVisibilityState, MissingEvidence,
   |                                                 ^^^^^^^^^^^^^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^^
12 |     MissingReason, QueryContext, SlotStatusSummary,
   |                    ^^^^^^^^^^^^  ^^^^^^^^^^^^^^^^^

warning: unused import: `super::hypothesis_state::HypothesisStatus`
  --> crates/locald/src/../../../locald/hypothesis/copilot.rs:14:5
   |
14 | use super::hypothesis_state::HypothesisStatus;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `super::session::SessionMode`
  --> crates/locald/src/../../../locald/hypothesis/copilot.rs:16:5
   |
16 | use super::session::SessionMode;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `super::incident::Incident`
 --> crates/locald/src/../../../locald/hypothesis/explanation.rs:9:5
  |
9 | use super::incident::Incident;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `Severity`
  --> crates/locald/src/../../../locald/hypothesis/explanation.rs:11:53
   |
11 | use super::promotion::{ConfidenceSeverityBreakdown, Severity};
   |                                                     ^^^^^^^^

warning: unused import: `EvidencePtr`
 --> crates/locald/src/../../../locald/hypothesis/storage.rs:5:46
  |
5 | use super::canonical_event::{CanonicalEvent, EvidencePtr};
  |                                              ^^^^^^^^^^^

warning: unused imports: `Deserialize` and `Serialize`
  --> crates/locald/src/../../../locald/hypothesis/storage.rs:11:13
   |
11 | use serde::{Deserialize, Serialize};
   |             ^^^^^^^^^^^  ^^^^^^^^^

warning: unused import: `Duration`
 --> crates/locald/src/../../../locald/hypothesis/changes.rs:6:24
  |
6 | use chrono::{DateTime, Duration, Utc};
  |                        ^^^^^^^^

warning: unused import: `HashSet`
 --> crates/locald/src/../../../locald/hypothesis/changes.rs:8:33
  |
8 | use std::collections::{HashMap, HashSet};
  |                                 ^^^^^^^

warning: unused import: `Duration`
 --> crates/locald/src/../../../locald/hypothesis/visibility.rs:7:24
  |
7 | use chrono::{DateTime, Duration, Utc};
  |                        ^^^^^^^^

warning: unused import: `ScopeKey`
  --> crates/locald/src/hypothesis_controller.rs:13:38
   |
13 |     LateArrivalPolicy, QueryContext, ScopeKey, SessionMode, StreamWatermark,
   |                                      ^^^^^^^^

warning: unused import: `DateTime`
  --> crates/locald/src/hypothesis_controller.rs:15:14
   |
15 | use chrono::{DateTime, Utc};
   |              ^^^^^^^^

warning: unused variable: `event`
  --> crates/locald/src/baseline/baseline_update.rs:89:54
   |
89 |     pub fn update_rates(baseline: &mut HostBaseline, event: &Event, metric_name: &str, rate: f64) {
   |                                                      ^^^^^ help: if this is intentional, prefix it with an underscore: `_event`
   |
   = note: `#[warn(unused_variables)]` on by default

warning: unused variable: `our_unique`
   --> crates/locald/src/../../../locald/hypothesis/disambiguator.rs:406:13
    |
406 |         let our_unique: Vec<_> = our_slots.difference(&their_slots).collect();
    |             ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_our_unique`

warning: unused variable: `fill_count`
   --> crates/locald/src/../../../locald/hypothesis/hypothesis_state.rs:581:29
    |
581 |         let (should_update, fill_count, fill_first_ts, fill_last_ts, time_ok) = {
    |                             ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_fill_count`

warning: unused variable: `fill_last_ts`
   --> crates/locald/src/../../../locald/hypothesis/hypothesis_state.rs:627:9
    |
627 |         fill_last_ts: DateTime<Utc>,
    |         ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_fill_last_ts`

warning: unused variable: `host_id`
   --> crates/locald/src/../../../locald/hypothesis/hypothesis_state.rs:841:9
    |
841 |         host_id: &str,
    |         ^^^^^^^ help: if this is intentional, prefix it with an underscore: `_host_id`

warning: associated items `from_results`, `resolved_contents`, `missing_ptrs`, and `has_integrity_errors` are never used
   --> crates/locald/src/evidence/deref.rs:270:12
    |
269 | impl BulkDerefResult {
    | -------------------- associated items in this implementation
270 |     pub fn from_results(results: Vec<DerefResult>) -> Self {
    |            ^^^^^^^^^^^^
...
317 |     pub fn resolved_contents(&self) -> Vec<(&EvidencePtr, &str)> {
    |            ^^^^^^^^^^^^^^^^^
...
326 |     pub fn missing_ptrs(&self) -> Vec<&EvidencePtr> {
    |            ^^^^^^^^^^^^
...
340 |     pub fn has_integrity_errors(&self) -> bool {
    |            ^^^^^^^^^^^^^^^^^^^^
    |
    = note: `#[warn(dead_code)]` on by default

warning: struct `EvidencePtrBuilder` is never constructed
   --> crates/locald/src/evidence/evidence_ptr.rs:214:12
    |
214 | pub struct EvidencePtrBuilder {
    |            ^^^^^^^^^^^^^^^^^^

warning: associated items `new`, `with_sha256`, `with_sha256_from_bytes`, `with_ts`, and `build` are never used
   --> crates/locald/src/evidence/evidence_ptr.rs:223:12
    |
222 | impl EvidencePtrBuilder {
    | ----------------------- associated items in this implementation
223 |     pub fn new(
    |            ^^^
...
237 |     pub fn with_sha256(mut self, sha256: impl Into<String>) -> Self {
    |            ^^^^^^^^^^^
...
242 |     pub fn with_sha256_from_bytes(mut self, bytes: &[u8]) -> Self {
    |            ^^^^^^^^^^^^^^^^^^^^^^
...
247 |     pub fn with_ts(mut self, ts: i64) -> Self {
    |            ^^^^^^^
...
252 |     pub fn build(self) -> EvidencePtr {
    |            ^^^^^

warning: field `db_path` is never read
   --> crates/locald/src/../../../locald/hypothesis/canonical_event.rs:509:5
    |
507 | pub struct EvidenceDerefEngine {
    |            ------------------- field in this struct
508 |     /// Database connection for canonical table lookups
509 |     db_path: String,
    |     ^^^^^^^

warning: field `citation_counter` is never read
   --> crates/locald/src/../../../locald/hypothesis/copilot.rs:416:5
    |
414 | pub struct TemplatedRenderer {
    |            ----------------- field in this struct
415 |     /// Citation counter for generating tokens
416 |     citation_counter: usize,
    |     ^^^^^^^^^^^^^^^^

warning: method `next_citation` is never used
   --> crates/locald/src/../../../locald/hypothesis/copilot.rs:430:8
    |
421 | impl TemplatedRenderer {
    | ---------------------- method in this implementation
...
430 |     fn next_citation(&mut self, claim: &Claim) -> String {
    |        ^^^^^^^^^^^^^

warning: method `check_ordering_constraints` is never used
   --> crates/locald/src/../../../locald/hypothesis/hypothesis_state.rs:649:8
    |
447 | impl HypothesisState {
    | -------------------- method in this implementation
...
649 |     fn check_ordering_constraints(&self, slot: &Slot, fill: &SlotFill) -> bool {
    |        ^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: field `track_non_incident` is never read
   --> crates/locald/src/../../../locald/hypothesis/changes.rs:251:5
    |
247 | pub struct DiffEngine {
    |            ---------- field in this struct
...
251 |     track_non_incident: bool,
    |     ^^^^^^^^^^^^^^^^^^

warning: unused imports: `ClaimEntry` and `TimelineEntry`
  --> crates/server/src/bundle_exchange.rs:16:21
   |
16 | use crate::report::{ClaimEntry, IntegrityNoteEntry, ReportBundle, TimelineEntry};
   |                     ^^^^^^^^^^                                    ^^^^^^^^^^^^^
   |
   = note: `#[warn(unused_imports)]` on by default

warning: unused import: `std::collections::HashMap`
  --> crates/server/src/golden_bundle.rs:26:5
   |
26 | use std::collections::HashMap;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `std::collections::BTreeMap`
  --> crates/server/src/health.rs:15:5
   |
15 | use std::collections::BTreeMap;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused variable: `path_str`
   --> crates/server/src/write_isolation.rs:275:9
    |
275 |     let path_str = path.to_string_lossy();
    |         ^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_path_str`
    |
    = note: `#[warn(unused_variables)]` on by default

warning: field `telemetry_root` is never read
  --> crates/server/src/write_isolation.rs:54:5
   |
52 | pub struct WriteIsolationContext {
   |            --------------------- field in this struct
53 |     /// Root telemetry directory
54 |     telemetry_root: PathBuf,
   |     ^^^^^^^^^^^^^^
   |
   = note: `#[warn(dead_code)]` on by default

warning: fields `id` and `tx` are never read
   --> crates/integration/src/realtime_sync.rs:100:5
    |
99  | struct ClientSession {
    |        ------------- fields in this struct
100 |     id: String,
    |     ^^
101 |     subscription: Subscription,
102 |     tx: broadcast::Sender<SyncMessage>,
    |     ^^
    |
    = note: `#[warn(dead_code)]` on by default

warning: unused import: `PathBuf`
 --> crates/agent-windows/src/evtlog_state.rs:9:23
  |
9 | use std::path::{Path, PathBuf};
  |                       ^^^^^^^
  |
  = note: `#[warn(unused_imports)]` on by default

warning: unused import: `DateTime`
 --> crates/agent-windows/src/sensors/sysmon_adapter.rs:4:14
  |
4 | use chrono::{DateTime, Utc};
  |              ^^^^^^^^

warning: unexpected `cfg` condition value: `windows_etw`
   --> crates/agent-windows/src/sensors/mod.rs:122:30
    |
122 |             enable_etw: cfg!(feature = "windows_etw"),
    |                              ^^^^^^^^^^^^^^^^^^^^^^^ help: remove the condition
    |
    = note: no expected values for `feature`
    = help: consider adding `windows_etw` as a feature in `Cargo.toml`
    = note: see <https://doc.rust-lang.org/nightly/rustc/check-cfg/cargo-specifics.html> for more information about checking conditional configuration
    = note: `#[warn(unexpected_cfgs)]` on by default

warning: unused variable: `host`
  --> crates/agent-windows/src/sensors/collect.rs:24:24
   |
24 | fn collect_evtx_events(host: &HostCtx) -> Vec<Event> {
   |                        ^^^^ help: if this is intentional, prefix it with an underscore: `_host`
   |
   = note: `#[warn(unused_variables)]` on by default

warning: variable does not need to be mutable
  --> crates/agent-windows/src/sensors/collect.rs:25:9
   |
25 |     let mut events = Vec::new();
   |         ----^^^^^^
   |         |
   |         help: remove this `mut`
   |
   = note: `#[warn(unused_mut)]` on by default

warning: variable does not need to be mutable
  --> crates/agent-windows/src/sensors/lateral_movement_monitor.rs:46:13
   |
46 |         let mut fields = event.fields.clone();
   |             ----^^^^^^
   |             |
   |             help: remove this `mut`

warning: variable does not need to be mutable
  --> crates/agent-windows/src/sensors/log_tamper_monitor.rs:43:13
   |
43 |         let mut fields = event.fields.clone();
   |             ----^^^^^^
   |             |
   |             help: remove this `mut`

warning: variable does not need to be mutable
  --> crates/agent-windows/src/sensors/powershell_monitor.rs:44:13
   |
44 |         let mut fields = event.fields.clone();
   |             ----^^^^^^
   |             |
   |             help: remove this `mut`

warning: variable does not need to be mutable
  --> crates/agent-windows/src/sensors/registry_monitor.rs:47:13
   |
47 |         let mut fields = event.fields.clone();
   |             ----^^^^^^
   |             |
   |             help: remove this `mut`

warning: variable does not need to be mutable
  --> crates/agent-windows/src/sensors/service_monitor.rs:43:13
   |
43 |         let mut fields = event.fields.clone();
   |             ----^^^^^^
   |             |
   |             help: remove this `mut`

warning: variable does not need to be mutable
  --> crates/agent-windows/src/sensors/task_scheduler_monitor.rs:44:13
   |
44 |         let mut fields = event.fields.clone();
   |             ----^^^^^^
   |             |
   |             help: remove this `mut`

warning: variable does not need to be mutable
  --> crates/agent-windows/src/sensors/wmi_monitor.rs:44:13
   |
44 |         let mut fields = event.fields.clone();
   |             ----^^^^^^
   |             |
   |             help: remove this `mut`

warning: fields `channels` and `dedup` are never read
   --> crates/agent-windows/src/wevt_reader.rs:137:5
    |
136 | pub struct WevtReader {
    |            ---------- fields in this struct
137 |     channels: Vec<ChannelConfig>,
    |     ^^^^^^^^
...
141 |     dedup: std::sync::Mutex<BTreeMap<String, (HashSet<u64>, VecDeque<u64>)>>,
    |     ^^^^^
    |
    = note: `#[warn(dead_code)]` on by default

warning: methods `is_duplicate`, `record_event`, `record_event_values_failure`, `record_xml_truncated`, and `record_render_failure` are never used
   --> crates/agent-windows/src/wevt_reader.rs:219:8
    |
147 | impl WevtReader {
    | --------------- methods in this implementation
...
219 |     fn is_duplicate(&self, channel: &str, record_id: u64) -> bool {
    |        ^^^^^^^^^^^^
...
242 |     fn record_event(&self, channel: &str, ts: DateTime<Utc>) {
    |        ^^^^^^^^^^^^
...
256 |     fn record_event_values_failure(&self, channel: &str) {
    |        ^^^^^^^^^^^^^^^^^^^^^^^^^^^
...
267 |     fn record_xml_truncated(&self) {
    |        ^^^^^^^^^^^^^^^^^^^^
...
273 |     fn record_render_failure(&self, _channel: &str) {
    |        ^^^^^^^^^^^^^^^^^^^^^

warning: function `poll_windows` is never used
   --> crates/agent-windows/src/wevt_reader.rs:886:12
    |
886 |     pub fn poll_windows(_reader: &mut WevtReader) -> Result<Vec<WevtRecord>, String> {
    |            ^^^^^^^^^^^^

warning: unused variable: `sensors_count`
   --> crates/agent-linux/src/capture_linux_rotating.rs:133:13
    |
133 |         let sensors_count = sensor_events.len();
    |             ^^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_sensors_count`
    |
    = note: `#[warn(unused_variables)]` on by default

warning: unused variable: `ebpf_read`
   --> crates/agent-linux/src/capture_linux_rotating.rs:383:9
    |
383 |         ebpf_read: u64,
    |         ^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_ebpf_read`

warning: unused variable: `sensors_read`
   --> crates/agent-linux/src/capture_linux_rotating.rs:384:9
    |
384 |         sensors_read: u64,
    |         ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_sensors_read`

warning: unused variable: `decode_failed`
   --> crates/agent-linux/src/capture_linux_rotating.rs:385:9
    |
385 |         decode_failed: u64,
    |         ^^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_decode_failed`

warning: unused variable: `stream_id`
  --> crates/agent-linux/src/sensors/ebpf_primitives/composite_detectors.rs:57:9
   |
57 |     let stream_id = base_event
   |         ^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_stream_id`

warning: unreachable pattern
  --> crates/agent-linux/src/sensors/ebpf_primitives/net_connect.rs:77:15
   |
76 |         53 | 5353 |  // DNS (tunneling)
   |         -- matches all the relevant values
77 |         123 | 53 |  // NTP, DNS (timing channels)
   |               ^^ no value can reach this
   |
   = note: `#[warn(unreachable_patterns)]` on by default

warning: unused variable: `argv_joined`
  --> crates/agent-linux/src/sensors/ebpf_primitives/script_exec.rs:66:9
   |
66 |     let argv_joined = argv.join(" ");
   |         ^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_argv_joined`

warning: unused variable: `sensor_name`
   --> crates/agent-linux/src/sensors_v2/config.rs:210:20
    |
210 | pub fn record_poll(sensor_name: &str) {
    |                    ^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_sensor_name`

warning: constant `EVASION_TOOLS` is never used
  --> crates/agent-linux/src/sensors/ebpf_primitives/defense_evasion.rs:44:7
   |
44 | const EVASION_TOOLS: &[&str] = &[
   |       ^^^^^^^^^^^^^
   |
   = note: `#[warn(dead_code)]` on by default

warning: constant `EPISODE_KIND_THRESHOLD` is never used
  --> crates/agent-linux/src/sensors_v2/alert_engine.rs:22:7
   |
22 | const EPISODE_KIND_THRESHOLD: usize = 3;
   |       ^^^^^^^^^^^^^^^^^^^^^^

warning: constant `EPISODE_WINDOW_MS` is never used
  --> crates/agent-linux/src/sensors_v2/alert_engine.rs:23:7
   |
23 | const EPISODE_WINDOW_MS: u64 = 600000; // 10 minutes
   |       ^^^^^^^^^^^^^^^^^

warning: function `episode_id` is never used
   --> crates/agent-linux/src/sensors_v2/alert_engine.rs:124:4
    |
124 | fn episode_id(event_keys: &[String]) -> String {
    |    ^^^^^^^^^^

warning: constant `PERSISTENCE_PATHS` is never used
  --> crates/agent-linux/src/sensors_v2/persistence_watch.rs:15:7
   |
15 | const PERSISTENCE_PATHS: &[&str] = &[
   |       ^^^^^^^^^^^^^^^^^

warning: `edr-locald` (lib) generated 35 warnings (run `cargo fix --lib -p edr-locald` to apply 22 suggestions)
warning: `edr-server` (lib) generated 5 warnings (run `cargo fix --lib -p edr-server` to apply 3 suggestions)
warning: `integration` (lib) generated 1 warning
warning: `agent-windows` (lib) generated 15 warnings (run `cargo fix --lib -p agent-windows` to apply 10 suggestions)
warning: `agent-linux` (lib) generated 13 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.51s
   Doc-tests agent_linux

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests agent_macos

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests agent_windows

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests edr_core

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests edr_locald

running 2 tests
test crates/locald/src/lib.rs - (line 27) ... ignored
test crates/locald/src/safety.rs - safety::safe_join_under (line 44) ... ignored

test result: ok. 0 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests edr_server

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests integration

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests workbench

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

[0;32m✓ Gate 4: Doc tests OK[0m

[1;33m[5/5] Running format check and clippy (advisory)...[0m
[0;32m✓ Format check passed[0m
[1;33mRunning clippy (advisory)...[0m
    Checking edr-core v0.1.0 (/Users/jermainebaker/git-glue/crates/core)
    Checking integration v0.1.0 (/Users/jermainebaker/git-glue/crates/integration)
    Checking workbench v0.1.0 (/Users/jermainebaker/git-glue/crates/workbench)
    Checking agent-windows v0.1.0 (/Users/jermainebaker/git-glue/crates/agent-windows)
    Checking agent-macos v0.1.0 (/Users/jermainebaker/git-glue/crates/agent-macos)
    Checking agent-linux v0.1.0 (/Users/jermainebaker/git-glue/crates/agent-linux)
    Checking edr-locald v0.1.0 (/Users/jermainebaker/git-glue/crates/locald)
warning: this `if` statement can be collapsed
   --> crates/agent-linux/src/sensors_v2/process_monitor.rs:85:37
    |
85  | / ...                   if (target_str.contains("ld-musl")
86  | | ...                       || target_str.contains("ld-linux"))
87  | | ...                       && target_str.contains("/tmp")
...   |
107 | | ...                   }
    | |_______________________^
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#collapsible_if
    = note: `#[warn(clippy::collapsible_if)]` on by default
help: collapse nested if block
    |
85  ~                                     if (target_str.contains("ld-musl")
86  +                                         || target_str.contains("ld-linux")) && target_str.contains("/tmp") && common::seen_once(
87  +                                             "process_monitor",
88  +                                             &format!("ld_inject:{}", pid_str),
89  +                                         ) {
90  +                                         let mut fields = BTreeMap::new();
91  +                                         fields.insert("pid".to_string(), json!(pid_str));
92  +                                         fields.insert(
93  +                                             "injected_lib".to_string(),
94  +                                             json!(target_str),
95  +                                         );
96  +                                         events.push(event_builders::event(
97  +                                             host,
98  +                                             "process_monitor",
99  +                                             "library_injection",
100 +                                             "warn",
101 +                                             fields,
102 +                                         ));
103 +                                     }
    |

warning: this `if` statement can be collapsed
  --> crates/agent-linux/src/sensors_v2/alert_engine.rs:62:5
   |
62 | /     if event_count > MAX_EVENTS_IN_MEMORY {
63 | |         if super::common::rate_limit("alert_engine", "buffer_overflow", 30000) {
64 | |             let mut fields = BTreeMap::new();
65 | |             fields.insert("event_count".to_string(), json!(event_count));
...  |
76 | |     }
   | |_____^
   |
   = help: for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#collapsible_if
help: collapse nested if block
   |
62 ~     if event_count > MAX_EVENTS_IN_MEMORY && super::common::rate_limit("alert_engine", "buffer_overflow", 30000) {
63 +         let mut fields = BTreeMap::new();
64 +         fields.insert("event_count".to_string(), json!(event_count));
65 +         fields.insert("max_capacity".to_string(), json!(MAX_EVENTS_IN_MEMORY));
66 + 
67 +         events.push(event_builders::event(
68 +             host,
69 +             "alert_engine",
70 +             "alert_candidate",
71 +             "warn",
72 +             fields,
73 +         ));
74 +     }
   |

warning: this `if` statement can be collapsed
  --> crates/agent-linux/src/sensors_v2/file_hash_watcher.rs:77:13
   |
77 | /             if known_malicious_hashes.contains(&hash.as_str()) {
78 | |                 if super::common::rate_limit(
79 | |                     "file_hash_watcher",
80 | |                     &format!("ioc:{}", file_path),
...  |
99 | |             }
   | |_____________^
   |
   = help: for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#collapsible_if
help: collapse nested if block
   |
77 ~             if known_malicious_hashes.contains(&hash.as_str()) && super::common::rate_limit(
78 +                     "file_hash_watcher",
79 +                     &format!("ioc:{}", file_path),
80 +                     3600000,
81 +                 ) {
82 +                 let mut fields = BTreeMap::new();
83 +                 fields.insert(
84 +                     event_keys::FILE_PATH.to_string(),
85 +                     json!(file_path.to_string()),
86 +                 );
87 +                 fields.insert(event_keys::FILE_HASH_SHA256.to_string(), json!(hash));
88 +                 fields.insert("ioc_match".to_string(), json!(true));
89 + 
90 +                 events.push(event_builders::event(
91 +                     host,
[0;32m✓ Gate 5: Format OK (clippy advisory only)[0m

[0;34m============================================================[0m
[0;32m  ALL GATES PASSED[0m
[0;34m============================================================[0m

Verified:
  ✓ Namespace isolation (imported bundles filtered by default)
  ✓ Path safety (directory traversal blocked)
  ✓ Selfcheck runtime behavior (BLOCKED on real failures)
  ✓ Support bundle redaction (user paths sanitized)
  ✓ Core test suite passing

Duration: 57s

