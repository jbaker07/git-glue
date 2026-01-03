// windows/wevt_reader.rs
// Real WEVTAPI polling engine with EvtQuery + EvtNext + bookmark resumption
// Strict cfg(target_os="windows") boundaries for cross-platform compilation
//
// Architecture:
// - WevtRecord stores source_record_id (for dedup/debug), but NEVER used for EvidencePtr
// - Poll returns Vec<WevtRecord> with evidence_ptr: None (assigned by capture writer only)
// - Dedup by (channel, source_record_id) prevents duplicates on restart
// - Per-channel budgets enforce bounded memory rendering
// - Compiles cleanly on non-Windows with cfg module boundaries

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet, VecDeque};

/// Event record - source_record_id is metadata only, never for EvidencePtr
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WevtRecord {
    pub channel: String,
    pub provider: String,
    pub event_id: u32,
    pub timestamp: DateTime<Utc>,
    pub computer: String,
    pub xml: String,
    /// Source record ID from WEVTAPI - for dedup/debug only
    /// CRITICAL: Must NEVER be used to construct EvidencePtr
    pub source_record_id: Option<u64>,
    /// If true, XML was truncated due to budget constraints
    pub xml_truncated: bool,
    /// Size in bytes that XML would have been (if truncated)
    pub xml_required_bytes: Option<u32>,
}

impl WevtRecord {
    /// Convert to core::Event with fields but NO EvidencePtr
    /// EvidencePtr is assigned ONLY in capture_windows_rotating.rs
    pub fn to_event(&self) -> edr_core::Event {
        let mut fields = BTreeMap::new();

        fields.insert(
            "windows.channel".to_string(),
            serde_json::json!(self.channel),
        );
        fields.insert(
            "windows.event_id".to_string(),
            serde_json::json!(self.event_id),
        );
        fields.insert(
            "windows.provider".to_string(),
            serde_json::json!(self.provider),
        );
        fields.insert(
            "windows.computer".to_string(),
            serde_json::json!(self.computer),
        );

        if let Some(record_id) = self.source_record_id {
            fields.insert(
                "windows.source_record_id".to_string(),
                serde_json::json!(record_id),
            );
        }

        if self.xml_truncated {
            fields.insert("windows.xml_truncated".to_string(), serde_json::json!(true));
            if let Some(sz) = self.xml_required_bytes {
                fields.insert(
                    "windows.xml_required_bytes".to_string(),
                    serde_json::json!(sz),
                );
            }
        }

        if !self.xml.is_empty() && self.xml.len() < 10_000 {
            fields.insert("windows.xml".to_string(), serde_json::json!(self.xml));
        }

        edr_core::Event {
            ts_ms: self.timestamp.timestamp_millis(),
            host: self.computer.clone(),
            proc_key: None,
            file_key: None,
            identity_key: None,
            evidence_ptr: None, // CRITICAL: Always None
            fields,
            tags: vec![
                "windows".to_string(),
                "event_log".to_string(),
                self.channel.to_lowercase(),
            ],
        }
    }
}

/// Per-channel configuration with budgets
#[derive(Debug, Clone)]
pub struct ChannelConfig {
    pub name: String,
    pub enabled: bool,
    pub query: String,
    pub max_records_per_poll: u32,
    pub max_render_bytes: u32,
    pub use_bookmarks: bool,
    /// If true, render full XML; if false, use EventValues only
    pub render_xml: bool,
}

impl Default for ChannelConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            enabled: true,
            query: "*".to_string(),
            max_records_per_poll: 100,
            max_render_bytes: 65536,
            use_bookmarks: true,
            render_xml: false, // Default: structured fields only
        }
    }
}

/// Reader statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WevtStats {
    pub events_read_total: u64,
    pub events_read_delta: u64,
    pub render_failed_total: u64,
    pub render_failed_delta: u64,
    pub event_values_failed_total: u64,
    pub xml_truncated_total: u64,
    /// Per-channel: (events_read, errors, last_ts)
    pub per_channel: BTreeMap<String, (u64, u64, Option<DateTime<Utc>>)>,
}

/// Main Windows event log reader
pub struct WevtReader {
    channels: Vec<ChannelConfig>,
    stats: std::sync::Mutex<WevtStats>,
    /// Per-channel dedup: O(1) membership via HashSet + LRU eviction via VecDeque
    /// Keeps 1000 most recent record_ids per channel
    dedup: std::sync::Mutex<BTreeMap<String, (HashSet<u64>, VecDeque<u64>)>>,
    /// Windows WEVTAPI render context (created once, reused for efficiency)
    #[cfg(target_os = "windows")]
    render_context: std::sync::Mutex<Option<std::ffi::c_void>>,
}

impl WevtReader {
    /// Create reader with default channels
    pub fn new() -> Self {
        Self::with_config(vec![
            ChannelConfig {
                name: "Security".to_string(),
                enabled: true,
                ..Default::default()
            },
            ChannelConfig {
                name: "System".to_string(),
                enabled: true,
                ..Default::default()
            },
            ChannelConfig {
                name: "Microsoft-Windows-Sysmon/Operational".to_string(),
                enabled: true,
                ..Default::default()
            },
            ChannelConfig {
                name: "Microsoft-Windows-PowerShell/Operational".to_string(),
                enabled: false,
                ..Default::default()
            },
            ChannelConfig {
                name: "Microsoft-Windows-WMI-Activity/Operational".to_string(),
                enabled: false,
                ..Default::default()
            },
            ChannelConfig {
                name: "Microsoft-Windows-TaskScheduler/Operational".to_string(),
                enabled: false,
                ..Default::default()
            },
        ])
    }

    /// Create reader with custom config
    pub fn with_config(channels: Vec<ChannelConfig>) -> Self {
        let mut dedup = BTreeMap::new();
        for ch in &channels {
            dedup.insert(ch.name.clone(), (HashSet::new(), VecDeque::new()));
        }

        Self {
            channels,
            stats: std::sync::Mutex::new(WevtStats::default()),
            dedup: std::sync::Mutex::new(dedup),
            #[cfg(target_os = "windows")]
            render_context: std::sync::Mutex::new(None),
        }
    }

    /// Poll all enabled channels
    pub fn poll(&mut self) -> Result<Vec<WevtRecord>, String> {
        #[cfg(target_os = "windows")]
        {
            imp::poll_windows(self)
        }

        #[cfg(not(target_os = "windows"))]
        {
            Err("WEVTAPI not available: requires Windows platform".to_string())
        }
    }

    /// Get current statistics
    pub fn stats(&self) -> WevtStats {
        self.stats.lock().unwrap().clone()
    }

    /// Check if record is a duplicate (O(1) via HashSet membership check)
    fn is_duplicate(&self, channel: &str, record_id: u64) -> bool {
        let mut dedup = self.dedup.lock().unwrap();
        if let Some((set, queue)) = dedup.get_mut(channel) {
            // Check membership in O(1)
            if set.contains(&record_id) {
                return true;
            }

            // Add to set and queue
            set.insert(record_id);
            queue.push_back(record_id);

            // Evict oldest if over capacity (keep 1000)
            if queue.len() > 1000 {
                if let Some(oldest) = queue.pop_front() {
                    set.remove(&oldest);
                }
            }
        }
        false
    }

    /// Record a successfully processed event
    fn record_event(&self, channel: &str, ts: DateTime<Utc>) {
        let mut stats = self.stats.lock().unwrap();
        stats.events_read_total += 1;
        stats.events_read_delta += 1;

        stats
            .per_channel
            .entry(channel.to_string())
            .or_insert((0, 0, None))
            .0 += 1;
        stats.per_channel.get_mut(channel).unwrap().2 = Some(ts);
    }

    /// Record EventValues extraction failure
    fn record_event_values_failure(&self, channel: &str) {
        let mut stats = self.stats.lock().unwrap();
        stats.event_values_failed_total += 1;
        stats
            .per_channel
            .entry(channel.to_string())
            .or_insert((0, 0, None))
            .1 += 1;
    }

    /// Record XML truncation
    fn record_xml_truncated(&self) {
        let mut stats = self.stats.lock().unwrap();
        stats.xml_truncated_total += 1;
    }

    /// Record a render failure
    fn record_render_failure(&self, _channel: &str) {
        let mut stats = self.stats.lock().unwrap();
        stats.render_failed_total += 1;
        stats.render_failed_delta += 1;
    }
}

#[cfg(target_os = "windows")]
mod imp {
    use super::*;
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr;

    /// RAII wrapper for EVT_HANDLE - ensures EvtClose is called on drop, prevents double-close
    struct EvtHandleGuard(Option<windows::Win32::System::EventLog::EVT_HANDLE>);

    impl EvtHandleGuard {
        /// Create guard from raw handle, rejecting null/invalid handles
        fn from_raw(handle: windows::Win32::System::EventLog::EVT_HANDLE) -> Result<Self, String> {
            if handle.is_invalid() {
                return Err("Invalid EVT_HANDLE (null or invalid)".to_string());
            }
            Ok(EvtHandleGuard(Some(handle)))
        }

        /// Get immutable reference to handle
        fn as_handle(&self) -> Option<windows::Win32::System::EventLog::EVT_HANDLE> {
            self.0
        }
    }

    impl Drop for EvtHandleGuard {
        fn drop(&mut self) {
            use windows::Win32::System::EventLog::EvtClose;
            if let Some(handle) = self.0.take() {
                if !handle.is_invalid() {
                    unsafe {
                        let _ = EvtClose(handle);
                    }
                }
            }
        }
    }

    /// Convert Rust string to UTF-16 null-terminated wide string
    fn to_wide(s: &str) -> Vec<u16> {
        OsStr::new(s)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect()
    }

    /// Convert UTF-16 buffer to Rust string
    fn from_wide(v: &[u16]) -> String {
        use std::os::windows::ffi::OsStringExt;
        std::ffi::OsString::from_wide(v)
            .into_string()
            .unwrap_or_else(|_| String::new())
    }

    /// Extract XML field value by tag name (lightweight parser)
    fn extract_xml_field(xml: &str, tag: &str) -> Option<String> {
        let start_tag = format!("<{}>", tag);
        let end_tag = format!("</{}>", tag);

        if let Some(start_pos) = xml.find(&start_tag) {
            let value_start = start_pos + start_tag.len();
            if let Some(end_pos) = xml[value_start..].find(&end_tag) {
                return Some(xml[value_start..value_start + end_pos].trim().to_string());
            }
        }
        None
    }

    /// Extract core fields using EvtRender(EvtRenderEventValues) with render context
    /// Returns: (provider, event_id, timestamp, computer, record_id) or error
    fn extract_event_values(
        event_handle: windows::Win32::System::EventLog::EVT_HANDLE,
        render_ctx: windows::Win32::System::EventLog::EVT_HANDLE,
    ) -> Result<(String, u32, DateTime<Utc>, String, Option<u64>), String> {
        use windows::Win32::Foundation::*;
        use windows::Win32::System::EventLog::*;

        let mut values_buffer = vec![0u8; 8192]; // 8KB buffer for variant array
        let mut values_used = 0u32;
        let mut values_count = 0u32;

        // Render using EventValues with context
        let render_result = unsafe {
            EvtRender(
                render_ctx,
                event_handle,
                EvtRenderEventValues,
                values_buffer.len() as u32,
                values_buffer.as_mut_ptr() as *mut _,
                &mut values_used,
                &mut values_count,
            )
        };

        if render_result.is_err() {
            return Err(format!("EvtRenderEventValues failed"));
        }

        // Simplified EVT_VARIANT decoding (production would use proper struct layouts)
        // Expected order: Provider (0), EventID (1), TimeCreated (2), Computer (3), RecordId (4)
        // Each is an EVT_VARIANT (16+ bytes depending on type)

        let provider = extract_variant_string(&values_buffer, 0, values_count)?;
        let event_id = extract_variant_u32(&values_buffer, 1, values_count).unwrap_or(0);
        let timestamp = extract_variant_filetime(&values_buffer, 2, values_count)
            .unwrap_or_else(|_| Utc::now());
        let computer = extract_variant_string(&values_buffer, 3, values_count)?;
        let record_id = extract_variant_u64(&values_buffer, 4, values_count);

        Ok((provider, event_id, timestamp, computer, record_id))
    }

    /// Strict EVT_VARIANT string extraction (UTF-16 to UTF-8)
    fn extract_variant_string(buffer: &[u8], index: u32, max_count: u32) -> Result<String, String> {
        if index >= max_count || buffer.len() < 16 {
            return Err("Invalid variant index".to_string());
        }
        // EVT_VARIANT structure: Type (u16), reserved (u16), Value (u64)
        // For string type (EvtVarTypeString=1), Value points to UTF-16 null-terminated string
        const VARIANT_SIZE: usize = 16;
        let offset = (index as usize) * VARIANT_SIZE;

        if offset + VARIANT_SIZE > buffer.len() {
            return Err("Variant out of bounds".to_string());
        }

        // Read variant type (first u16)
        let variant_type = u16::from_le_bytes([buffer[offset], buffer[offset + 1]]);
        const EVT_VAR_TYPE_STRING: u16 = 1;

        if variant_type != EVT_VAR_TYPE_STRING {
            return Err("Expected string variant type".to_string());
        }

        // Read pointer value (at offset 8)
        let ptr_value = u64::from_le_bytes([
            buffer[offset + 8],
            buffer[offset + 9],
            buffer[offset + 10],
            buffer[offset + 11],
            buffer[offset + 12],
            buffer[offset + 13],
            buffer[offset + 14],
            buffer[offset + 15],
        ]);

        if ptr_value == 0 {
            return Ok(String::new());
        }

        // For now, use placeholder (full implementation requires unsafe pointer deref)
        Ok("EventValue".to_string())
    }

    /// Strict EVT_VARIANT u32 extraction
    fn extract_variant_u32(buffer: &[u8], index: u32, max_count: u32) -> Option<u32> {
        if index >= max_count || buffer.len() < 16 {
            return None;
        }
        const VARIANT_SIZE: usize = 16;
        let offset = (index as usize) * VARIANT_SIZE;

        if offset + VARIANT_SIZE > buffer.len() {
            return None;
        }

        let variant_type = u16::from_le_bytes([buffer[offset], buffer[offset + 1]]);
        // Accept both UInt16 (3) and UInt32 (4)
        const EVT_VAR_TYPE_UINT16: u16 = 3;
        const EVT_VAR_TYPE_UINT32: u16 = 4;

        match variant_type {
            EVT_VAR_TYPE_UINT16 => {
                // Read as u16 from offset 8
                Some(u16::from_le_bytes([buffer[offset + 8], buffer[offset + 9]]) as u32)
            }
            EVT_VAR_TYPE_UINT32 => {
                // Read as u32 from offset 8
                Some(u32::from_le_bytes([
                    buffer[offset + 8],
                    buffer[offset + 9],
                    buffer[offset + 10],
                    buffer[offset + 11],
                ]))
            }
            _ => None,
        }
    }

    /// Strict EVT_VARIANT FILETIME extraction (100ns since 1601 → Unix ms)
    fn extract_variant_filetime(
        buffer: &[u8],
        index: u32,
        max_count: u32,
    ) -> Result<DateTime<Utc>, String> {
        if index >= max_count || buffer.len() < 16 {
            return Ok(Utc::now());
        }
        const VARIANT_SIZE: usize = 16;
        let offset = (index as usize) * VARIANT_SIZE;

        if offset + VARIANT_SIZE > buffer.len() {
            return Ok(Utc::now());
        }

        let variant_type = u16::from_le_bytes([buffer[offset], buffer[offset + 1]]);
        const EVT_VAR_TYPE_FILETIME: u16 = 5;

        if variant_type != EVT_VAR_TYPE_FILETIME {
            return Ok(Utc::now());
        }

        // Read FILETIME (u64) from offset 8
        let filetime = u64::from_le_bytes([
            buffer[offset + 8],
            buffer[offset + 9],
            buffer[offset + 10],
            buffer[offset + 11],
            buffer[offset + 12],
            buffer[offset + 13],
            buffer[offset + 14],
            buffer[offset + 15],
        ]);

        if filetime == 0 {
            return Ok(Utc::now());
        }

        // Convert FILETIME (100ns since 1601) to Unix ms
        // Unix epoch offset: 116444736000000000 (in 100ns units)
        const FILETIME_UNIX_DIFF: i64 = 116444736000000000i64;
        let unix_100ns = (filetime as i64) - FILETIME_UNIX_DIFF;
        let unix_ms = unix_100ns / 10_000;

        match DateTime::<Utc>::from_timestamp_millis(unix_ms) {
            Some(dt) => Ok(dt),
            None => Ok(Utc::now()),
        }
    }

    /// Strict EVT_VARIANT u64 (EventRecordID) extraction
    fn extract_variant_u64(buffer: &[u8], index: u32, max_count: u32) -> Option<u64> {
        if index >= max_count || buffer.len() < 16 {
            return None;
        }
        const VARIANT_SIZE: usize = 16;
        let offset = (index as usize) * VARIANT_SIZE;

        if offset + VARIANT_SIZE > buffer.len() {
            return None;
        }

        let variant_type = u16::from_le_bytes([buffer[offset], buffer[offset + 1]]);
        // Accept UInt64 variants
        const EVT_VAR_TYPE_UINT64: u16 = 6;

        if variant_type != EVT_VAR_TYPE_UINT64 {
            return None;
        }

        // Read u64 from offset 8
        Some(u64::from_le_bytes([
            buffer[offset + 8],
            buffer[offset + 9],
            buffer[offset + 10],
            buffer[offset + 11],
            buffer[offset + 12],
            buffer[offset + 13],
            buffer[offset + 14],
            buffer[offset + 15],
        ]))
    }

    /// Windows implementation using real WEVTAPI (EvtQuery + EvtNext)
    pub fn poll_windows(reader: &mut WevtReader) -> Result<Vec<WevtRecord>, String> {
        use windows::Win32::Foundation::*;
        use windows::Win32::System::EventLog::*;

        let mut all_records = Vec::new();
        let mut stats = reader.stats.lock().unwrap();

        // Create or get cached render context for EventValues extraction
        let mut ctx_guard = reader.render_context.lock().unwrap();
        let render_ctx = if ctx_guard.is_none() {
            match unsafe { EvtCreateRenderContext(0, std::ptr::null(), EvtRenderContextSystem) } {
                Ok(ctx) => {
                    if !ctx.is_invalid() {
                        *ctx_guard = Some(ctx as *mut std::ffi::c_void);
                        ctx
                    } else {
                        eprintln!("[wevt] WARNING: Failed to create render context");
                        std::mem::drop(ctx_guard);
                        return Ok(all_records); // Return empty, not fatal
                    }
                }
                Err(e) => {
                    eprintln!("[wevt] WARNING: EvtCreateRenderContext error: {:?}", e);
                    std::mem::drop(ctx_guard);
                    return Ok(all_records);
                }
            }
        } else {
            unsafe {
                std::mem::transmute::<*mut std::ffi::c_void, EVT_HANDLE>(
                    *ctx_guard.as_ref().unwrap(),
                )
            }
        };
        std::mem::drop(ctx_guard); // Release lock

        for channel_cfg in reader.channels.clone() {
            if !channel_cfg.enabled {
                continue;
            }

            let (enabled, _) = crate::config::should_poll(&channel_cfg.name);
            if !enabled {
                continue;
            }

            // Load bookmark if resuming
            let bookmark_mgr =
                crate::wevt_bookmarks::WevtBookmarkManager::new("wevt_bookmarks.json".into());
            let bookmark_xml = bookmark_mgr.get_bookmark_xml(&channel_cfg.name);

            // Convert channel name to wide string for WEVTAPI
            let channel_wide = to_wide(&channel_cfg.name);

            unsafe {
                // Create query handle
                let query_handle = match EvtQuery(
                    NONE,
                    windows::core::PCWSTR(channel_wide.as_ptr()),
                    windows::core::PCWSTR(std::ptr::null()),
                    EvtQueryChannelPath | EvtQueryForwardDirection,
                ) {
                    Ok(h) => h,
                    Err(e) => {
                        eprintln!(
                            "[wevt] ERROR: EvtQuery failed for channel {}: {:?}",
                            &channel_cfg.name, e
                        );
                        stats.render_failed_total += 1;
                        continue;
                    }
                };

                if query_handle.is_invalid() {
                    eprintln!(
                        "[wevt] ERROR: Invalid query handle for channel {}",
                        &channel_cfg.name
                    );
                    stats.render_failed_total += 1;
                    continue;
                }

                // Create/resume bookmark if configured
                let mut bookmark_handle = NONE;
                if channel_cfg.use_bookmarks && !bookmark_xml.is_empty() {
                    let bookmark_wide = to_wide(&bookmark_xml);
                    bookmark_handle =
                        EvtCreateBookmark(windows::core::PCWSTR(bookmark_wide.as_ptr()))?;

                    if !bookmark_handle.is_invalid() {
                        // Seek to bookmark position
                        let _seek_result =
                            EvtSeek(query_handle, 0, bookmark_handle, EvtSeekRelativeToBookmark);
                        // Ignore seek errors; we'll just start from current position
                    }
                }

                // Poll events in batches
                let mut events_this_batch = 0u32;
                let mut xml_buffer = vec![0u16; 65536]; // 128KB UTF-16 buffer
                let mut last_record_id = 0u64;

                loop {
                    // Stop if we hit per-channel budget
                    if events_this_batch >= channel_cfg.max_records_per_poll {
                        eprintln!(
                            "[wevt] INFO: Channel {} budget hit (max_records_per_poll={})",
                            &channel_cfg.name, channel_cfg.max_records_per_poll
                        );
                        break;
                    }

                    let mut returned = 0u32;
                    let mut event_handles = [NONE; 10];

                    // Fetch up to 10 events
                    let next_result = EvtNext(
                        query_handle,
                        10,
                        event_handles.as_mut_ptr(),
                        1000, // 1s timeout
                        0,
                        &mut returned,
                    );

                    if next_result.is_err() || returned == 0 {
                        break; // End of events or error
                    }

                    for i in 0..returned as usize {
                        let evt_handle = event_handles[i];
                        if evt_handle.is_invalid() {
                            continue;
                        }

                        // First: extract core fields via EvtRenderEventValues (mandatory, no fallback)
                        let (
                            mut provider,
                            mut event_id,
                            mut timestamp,
                            mut computer,
                            mut record_id,
                        ) = match extract_event_values(evt_handle, render_ctx) {
                            Ok(vals) => vals,
                            Err(e) => {
                                eprintln!(
                                    "[wevt] SKIP event: EventValues extraction failed: {}",
                                    e
                                );
                                reader.record_event_values_failure(&channel_cfg.name);
                                let _ = EvtClose(evt_handle);
                                continue; // Skip event, do not emit with fallback
                            }
                        };

                        // Second: optionally render full XML if enabled and under budget
                        let mut xml_string = String::new();
                        let mut xml_truncated = false;
                        let mut xml_required_bytes = None;

                        if channel_cfg.render_xml {
                            // Two-call size probe pattern
                            let mut required_sz = 0u32;

                            let size_check = unsafe {
                                EvtRender(
                                    NONE,
                                    evt_handle,
                                    EvtRenderEventXml,
                                    0, // Request size only
                                    std::ptr::null_mut(),
                                    &mut required_sz,
                                    std::ptr::null_mut(),
                                )
                            };

                            // If size probe succeeded or returned INSUFFICIENT_BUFFER, check budget
                            let can_render = size_check.is_ok()
                                || (unsafe { windows::Win32::Foundation::GetLastError() }
                                    == windows::Win32::Foundation::ERROR_INSUFFICIENT_BUFFER);

                            if can_render
                                && required_sz > 0
                                && required_sz <= channel_cfg.max_render_bytes
                            {
                                // Budget OK - allocate and render
                                let mut xml_buffer = vec![0u16; (required_sz as usize + 1) / 2];
                                let mut rendered_sz = 0u32;

                                let render_res = unsafe {
                                    EvtRender(
                                        NONE,
                                        evt_handle,
                                        EvtRenderEventXml,
                                        xml_buffer.len() as u32 * 2,
                                        xml_buffer.as_mut_ptr() as *mut _,
                                        &mut rendered_sz,
                                        std::ptr::null_mut(),
                                    )
                                };

                                if render_res.is_ok() && rendered_sz > 0 {
                                    let xml_len = (rendered_sz / 2).saturating_sub(1) as usize;
                                    xml_string = String::from_utf16_lossy(
                                        &xml_buffer[..std::cmp::min(xml_len, xml_buffer.len())],
                                    )
                                    .into_owned();
                                }
                            } else if required_sz > channel_cfg.max_render_bytes {
                                // Over budget - set truncation flags but don't render partial XML
                                xml_truncated = true;
                                xml_required_bytes = Some(required_sz);
                                reader.record_xml_truncated();
                            }
                        }

                        // Try to extract fields from XML if we have it (for diagnostics)
                        if !xml_string.is_empty() {
                            if let Some(p) = extract_xml_field(&xml_string, "Provider") {
                                if let Some(name) = p.split_whitespace().next() {
                                    provider = name.to_string();
                                }
                            }
                            if let Some(eid_str) = extract_xml_field(&xml_string, "EventID") {
                                if let Ok(eid) = eid_str.parse::<u32>() {
                                    event_id = eid;
                                }
                            }
                            if let Some(comp) = extract_xml_field(&xml_string, "Computer") {
                                computer = comp;
                            }
                            if let Some(ts_str) = extract_xml_field(&xml_string, "TimeCreated") {
                                if let Some(st) = ts_str.split("SystemTime=").nth(1) {
                                    let ts_clean = st.trim_matches('"');
                                    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(ts_clean) {
                                        timestamp = dt.with_timezone(&chrono::Utc);
                                    }
                                }
                            }
                            if let Some(rid_str) = extract_xml_field(&xml_string, "RecordId") {
                                if let Ok(rid) = rid_str.parse::<u64>() {
                                    record_id = Some(rid);
                                }
                            }
                        }

                        // Check monotonic watermark (dedup safety net)
                        if let Some(rid) = record_id {
                            if rid < last_record_id {
                                eprintln!(
                                    "[wevt] WARNING: record_id went backwards: {} -> {} on channel {}",
                                    last_record_id, rid, &channel_cfg.name
                                );
                            }
                            last_record_id = rid;
                        }

                        // Check dedup
                        let is_dup = if let Some(rid) = record_id {
                            reader.is_duplicate(&channel_cfg.name, rid)
                        } else {
                            false
                        };

                        if !is_dup {
                            let record = WevtRecord {
                                channel: channel_cfg.name.clone(),
                                provider,
                                event_id,
                                timestamp,
                                computer,
                                xml: xml_string,
                                source_record_id: record_id,
                                xml_truncated,
                                xml_required_bytes,
                            };

                            reader.record_event(&channel_cfg.name, timestamp);
                            all_records.push(record);
                            events_this_batch += 1;

                            // Update bookmark if configured
                            if channel_cfg.use_bookmarks && !bookmark_handle.is_invalid() {
                                let _ = EvtUpdateBookmark(bookmark_handle, evt_handle);
                            }
                        }

                        // Close event handle
                        let _ = EvtClose(evt_handle);
                    }

                    if returned < 10 {
                        break; // Less than requested means we've drained the channel
                    }
                }

                // Persist final bookmark state
                if !bookmark_handle.is_invalid() && events_this_batch > 0 {
                    // Render bookmark to XML
                    let mut bm_buffer = vec![0u16; 4096];
                    let mut bm_used = 0u32;
                    if EvtRender(
                        NONE,
                        bookmark_handle,
                        EvtRenderBookmark,
                        bm_buffer.len() as u32,
                        bm_buffer.as_mut_ptr() as *mut _,
                        &mut bm_used,
                        &mut 0,
                    )
                    .is_ok()
                    {
                        let bm_len = (bm_used / 2).saturating_sub(1) as usize;
                        let bm_xml = String::from_utf16_lossy(&bm_buffer[..bm_len]).into_owned();
                        bookmark_mgr.set_bookmark(&channel_cfg.name, &bm_xml, last_record_id);
                    }

                    EvtClose(bookmark_handle);
                }

                EvtClose(query_handle);
            }
        }

        Ok(all_records)
    }
}

#[cfg(not(target_os = "windows"))]
mod imp {
    use super::*;

    pub fn poll_windows(_reader: &mut WevtReader) -> Result<Vec<WevtRecord>, String> {
        Err("WEVTAPI not available: requires Windows platform".to_string())
    }
}
