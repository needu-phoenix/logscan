# logscan

A fast, streaming command-line tool for analysing web-server access logs in the Apache/nginx **Combined Log Format**. Parses large logs in constant memory and reports traffic summaries, top clients, status-code breakdowns, and filtered views — in human-readable tables or JSON.

## Features

- **`summary`** — total requests, unique IPs, status-code breakdown, total bytes, and a count of unparseable lines
- **`top`** — the most frequent client IP addresses, ranked, with percentages
- **`status`** — request counts per HTTP status code, with human-readable meanings
- **`filter`** — show only the requests matching a given status code
- **Table or JSON** output for every command (`--format json`)
- **Streams input line-by-line** — memory stays flat regardless of file size (see [Design](#design))
- Malformed lines are skipped and counted, never fatal

## Building

Requires Rust (stable).

```bash
git clone https://github.com/<you>/logscan
cd logscan
cargo build --release
```

The binary is at `target/release/logscan`.

## Usage

All commands take `--file-name <path>` and an optional `--format table|json` (default: `table`).

### summary

Overall traffic statistics for the log.

```bash
logscan --file-name access.log summary
```

```
Total requests:            994
Unique IPs:                153
Total bytes:          17749598
Ignored lines:               6
Status breakdown:
  2xx            714 -- (71.8%)
  3xx            127 -- (12.8%)
  4xx            124 -- (12.5%)
  5xx             29 -- ( 2.9%)
```

### top

The most frequent client IPs, ranked. `-n` sets how many to show.

```bash
logscan --file-name access.log top -n 5
```

```
RANK IP             REQUESTS  % OF TOTAL
1    203.0.113.7         116       11.67
2    203.0.113.42        102       10.26
3    198.51.100.7         95        9.56
4    192.168.1.100        87        8.75
5    192.168.1.58         11        1.11
```

### status

Request counts per HTTP status code, with meanings.

```bash
logscan --file-name access.log status
```

```
INDEX STATUS COUNT  MEANING
1     200    714    OK
2     301    41     Moved Permanently
3     302    25     Found
4     304    61     Not Modified
5     400    14     Bad Request
6     401    15     Unauthorized
7     403    13     Forbidden
8     404    82     Not Found
9     500    16     Internal Server Error
10    502    8      Bad Gateway
11    503    5      Service Unavailable
```

### filter

Show only requests matching a given status code.

```bash
logscan --file-name access.log filter --status 404
```

```
IP                  PATH                          SIZE
203.0.113.7         /products                     128374
192.168.1.52        /api/data                     0
203.0.113.42        /.env                         2326
...
```

### JSON output

Any command accepts `--format json` for machine-readable output — useful for piping into `jq` or feeding a dashboard.

```bash
logscan --file-name access.log top -n 3 --format json
```

```json
[
  {
    "rank": 1,
    "ip": "203.0.113.7",
    "requests": 116,
    "percent": 11.670020120724347
  },
  {
    "rank": 2,
    "ip": "203.0.113.42",
    "requests": 102,
    "percent": 10.261569416498995
  },
  {
    "rank": 3,
    "ip": "198.51.100.7",
    "requests": 95,
    "percent": 9.557344064386317
  }
]
```

Percentages are emitted at full `f64` precision — the JSON carries exact data, and any rounding for display is left to the consumer (the table output rounds to two decimals for humans).

## Design

A few decisions shaped how logscan is built:

**Streaming, not loading.** logscan reads the input one line at a time and aggregates on the fly — it never holds the whole file in memory. Memory usage is constant in the number of *lines*; it grows only with the number of distinct keys being tracked (e.g. unique IPs). This means it processes arbitrarily large logs (tested on multi-million-line files) without loading them: a 10-million-line log uses roughly the same memory as a thousand-line one, because the file is streamed rather than read into memory. The one real limit is cardinality — exact unique-IP counting must store each distinct IP, so memory scales with the number of *unique* clients, not with file size.

**Lenient parsing.** Lines that don't match the expected format are skipped and counted (reported as "Ignored lines") rather than aborting the run. Real logs contain malformed, truncated, or interleaved lines, and a log tool should degrade gracefully and tell you how much it couldn't parse.

**Compute/output separation.** Each command separates its aggregation logic from its output formatting. The aggregation returns plain data; a separate step renders it as a table or as JSON. This keeps the aggregation independently testable (fed in-memory readers, asserted on returned data) and makes supporting two output formats straightforward.

## Testing

```bash
cargo test
```

Unit tests cover:

- **The line parser** — valid lines (all fields extracted correctly), malformed lines (rejected), and edge cases (empty lines, incomplete lines, dash-for-zero byte sizes, non-IPv4 addresses).
- **The aggregation logic** of the `summary`, `top`, and `filter` commands — total counts, malformed-line counting, unique-IP deduplication, ranking with deterministic tie-breaking, and status-based selection.

Tests use in-memory readers (`std::io::Cursor`), so they run without touching the filesystem. The `status` command is verified manually against a known dataset; unit-test effort is concentrated on the parser and the more complex aggregations.

The codebase is `cargo fmt`-clean and passes `cargo clippy --all-targets -- -D warnings`.

## Scope

logscan targets the Apache/nginx **Combined Log Format** with IPv4 addresses. Lines in other formats, or with IPv6 addresses, are treated as unparseable and counted as ignored. Configurable log formats and IPv6 support are possible future additions.


