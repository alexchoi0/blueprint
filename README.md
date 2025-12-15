# Blueprint

A two-phase execution engine for Starlark scripts with controlled system access and approval workflows.

## Overview

Blueprint separates script planning from execution, enabling safe and auditable automation:

1. **Planning Phase**: Scripts are parsed and compiled into an operational plan without executing side effects
2. **Execution Phase**: Plans execute in parallel while respecting dependencies and approval policies

This architecture allows you to inspect exactly what a script will do before it runs, making Blueprint ideal for automation tasks that require oversight.

## Features

- **Starlark Language**: Python-like syntax that's easy to read and write
- **Two-Phase Execution**: Plan first, execute later with full visibility
- **Parallel Execution**: Independent operations run concurrently
- **Approval System**: Policy-based approval for sensitive operations
- **Rich Standard Library**: File I/O, HTTP, JSON, shell execution, networking

## Installation

```bash
cargo install --path .
```

## Quick Start

Create a script `hello.star`:

```python
load("@bp", "io")

io.write_file("/tmp/hello.txt", "Hello, Blueprint!")
```

Check what the script will do:

```bash
blueprint schema hello.star
```

Run the script:

```bash
blueprint run hello.star
```

## CLI Commands

| Command | Description |
|---------|-------------|
| `check <script>` | Validate script syntax |
| `schema <script>` | Show what operations the script will perform |
| `compile <script>` | Compile to a binary plan file |
| `run <script>` | Parse, compile, and execute in one step |
| `exec <plan>` | Execute a pre-compiled plan |
| `inspect <file>` | Examine compiled plans or schemas |

## Standard Library

Blueprint includes a comprehensive standard library:

| Module | Description |
|--------|-------------|
| `io` | File operations (read, write, delete, copy, etc.) |
| `http` | HTTP requests |
| `json` | JSON encode/decode |
| `exec` | Shell command execution |
| `parallel` | Concurrent operations (gather, race, sequence) |
| `tcp` | TCP client/server |
| `udp` | UDP operations |
| `unix` | Unix domain sockets |

### Example: Parallel HTTP Requests

```python
load("@bp", "http", "parallel")

urls = [
    "https://api.example.com/users",
    "https://api.example.com/posts",
    "https://api.example.com/comments",
]

def fetch(url):
    return http.request("GET", url)

results = parallel.gather([fetch(url) for url in urls])
```

### Example: File Processing

```python
load("@bp", "io", "json")

data = json.decode(io.read_file("config.json"))
data["updated"] = True
io.write_file("config.json", json.encode(data))
```

## Architecture

Blueprint is organized into several crates:

| Crate | Description |
|-------|-------------|
| `blueprint_common` | Shared types (Op, Plan, Schema) |
| `blueprint_generator` | Starlark → Schema → Plan compilation |
| `blueprint_interpreter` | Async plan execution |
| `blueprint_approval` | Policy-based approval system |
| `blueprint_storage` | SQLite state persistence |

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Copyright

Copyright 2025 Alex Choi
