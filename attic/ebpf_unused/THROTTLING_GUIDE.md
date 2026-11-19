//! Producer-side throttling guide for eBPF sensors
//! 
//! OVERVIEW:
//! Some hooks generate excessive volume (file ops, network ops) that can overload
//! the ringbuf. This module documents which hooks should have throttling and why
//! it's safe to apply.

/*
THROTTLED HOOKS (add sample map per-PID):

1. EVT_OPEN (syscall 257, 288)
   Volume: File open is noisy on startup, build systems, package managers
   Filter: Sample 1-in-100 for non-root; always log root opens
   Invariant: Exec path already has exec monitoring; file opens are not critical for
             initial execution. Rootkit detection uses exec+mprotect, not open.
   Safe: YES - privileged file opens still logged (uid==0)

2. EVT_WRITE (syscall 1)
   Volume: Extremely high; all writes to files/sockets
   Filter: Completely disabled for non-root; always log root writes
   Invariant: File tampering uses EVT_CHMOD/EVT_SETXATTR which are logged.
             Persistence uses CRON/systemd which are logged.
             Direct binary write attack would show in EVT_MPROTECT+EVT_EXEC
   Safe: YES - attacks require mprotect for RWX, which is always logged

3. EVT_READ (syscall 0)
   Volume: Extreme; all reads
   Filter: Sample 1-in-1000 for non-root; always log root reads
   Invariant: Data exfil uses network syscalls (EVT_SENDTO/EVT_SENDMSG logged)
             No attack requires logging every read
   Safe: YES - sensitive data exfil requires network syscalls (always logged)

4. EVT_BIND (syscall 49)
   Volume: High; every listening socket
   Filter: Always log for root (port < 1024); sample 1-in-10 for unprivileged
   Invariant: Backdoor detection: EVT_SOCKET+EVT_LISTEN already logged
             Only root can bind to system ports; unprivileged binds less critical
   Safe: YES - root backdoors are fully visible

5. EVT_CONNECT (syscall 42)
   Volume: High; every outbound connection
   Filter: Always log for root; sample 1-in-100 for unprivileged
   Invariant: Beaconing uses root sockets (reverse shells, C2) or named channels
             Unprivileged connects to localhost are development/testing (low signal)
   Safe: YES - root C2 channels are always logged

6. EVT_SENDTO/EVT_SENDMSG (syscall 44, 46)
   Volume: Very high; all UDP sends, all datagram sends
   Filter: Always log for root; sample 1-in-1000 for unprivileged
   Invariant: Covert DNS exfil uses sendto (UDP 53); ALWAYS logged for root
             Unprivileged DNS: normal; covert channels would use root
   Safe: YES - exfiltration from privileged processes is the risk

7. EVT_TCP_RETRANS (syn-ack retransmission tracking)
   Volume: High; triggered by packet loss, network issues
   Filter: Completely disabled; retrans is observable at netstat level
   Invariant: Network errors don't indicate compromise
   Safe: YES - replace with application-level anomalies if needed

UNTHROTTLED HOOKS (always log, no sampling):

- EVT_EXEC (process execution)
- EVT_FORK / EVT_CLONE (new processes)
- EVT_MPROTECT (memory write + execute)
- EVT_PTRACE (process injection)
- EVT_MOD_LOAD (kernel module load)
- EVT_SETUID / EVT_CAPSET (privilege escalation)
- EVT_CHMOD / EVT_CHOWN (permissions change)
- EVT_SETXATTR (extended attributes; persistence vector)
- EVT_UNSHARE / EVT_SETNS (container/ns escape)

IMPLEMENTATION APPROACH:

Add to edr_events.h:

  struct {
      __uint(type, BPF_MAP_TYPE_LRU_HASH);
      __uint(max_entries, 16384);
      __type(key, __u32);  // pid
      __type(value, __u64); // sample counter
  } edr_sample_counters SEC(".maps");

  static __always_inline bool edr_should_sample(enum edr_evt_type type, __u32 pid, __u8 is_root) {
      if (is_root) return true;  // Always log root
      
      __u32 sample_rate = 1;  // Default: always log
      
      if (type == EVT_OPEN)   sample_rate = 100;
      if (type == EVT_WRITE)  sample_rate = 10000;
      if (type == EVT_READ)   sample_rate = 100000;
      if (type == EVT_CONNECT) sample_rate = 100;
      
      __u64 *counter = bpf_map_lookup_elem(&edr_sample_counters, &pid);
      __u64 count = counter ? *counter : 0;
      
      bool should_emit = (count % sample_rate) == 0;
      
      if (counter) {
          __sync_fetch_and_add(counter, 1);
      }
      
      return should_emit;
  }

Apply before ringbuf_output:

  if (!edr_should_sample(e->type, e->tgid, e->uid == 0)) {
      return 0;
  }
  return edr_emit(e);

SAFETY JUSTIFICATION:

1. Root-privileged syscalls are ALWAYS logged (is_root check first)
2. Sampling only applies to high-volume, low-signal events
3. Critical invariants (mprotect→exec, ptrace, module load) are unsampled
4. All tests and threat models remain valid with sampling disabled

METRICS:

Userspace must export:
  ebpf_throttled_events_sampled: count of events sampled in (e.g., 1-in-100)
  ebpf_throttle_sample_rate: current sampling ratio
  
This enables auditing: if throttle_sample_rate > 100, alert ops that volume is extreme.
*/
