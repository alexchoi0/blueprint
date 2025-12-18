# Blueprint

A high-performance Starlark script executor with implicit async I/O, built on Tokio.

Blueprint lets you write simple, synchronous-looking scripts while the runtime automatically handles async I/O operations under the hood. Perfect for automation, deployment scripts, and data processing tasks.

## Features

- **Implicit Async I/O** - Write sync code, get async performance
- **Concurrent Execution** - Run multiple scripts in parallel with `bp run "*.star"`
- **Native Functions** - File I/O, HTTP requests, process execution, JSON, and more
- **Full Starlark Support** - Functions, lambdas, comprehensions, f-strings
- **Fast** - Built on Rust and Tokio for maximum performance

## Installation

```bash
# Clone and build
git clone https://github.com/alexchoi0/blueprint.git
cd blueprint
cargo build --release

# Add to PATH
cp target/release/bp /usr/local/bin/
```

## Quick Start

```bash
# Evaluate an expression
bp eval "1 + 2"
# => 3

# Run a script
bp run script.star

# Run multiple scripts concurrently
bp run "scripts/*.star"

# Limit concurrency
bp run "scripts/*.star" -j 4

# Check syntax without running
bp check script.star
```

## Example Script

```python
# deploy.star
print("Starting deployment...")

# Read configuration
config = json.decode(read_file("config.json"))
print(f"Deploying {config['app']} to {config['env']}")

# Run build
result = shell("npm run build")
if result.code != 0:
    fail("Build failed: " + result.stderr)

# Upload files
for file in glob("dist/*"):
    print(f"Uploading {file}...")

print("Deployment complete!")
```

Run it:
```bash
bp run deploy.star
```

## Native Functions

### File Operations
```python
content = read_file("path/to/file")
write_file("path/to/file", "content")
append_file("path/to/file", "more content")
exists("path/to/file")      # True/False
is_file("path")             # True/False
is_dir("path")              # True/False
mkdir("new/directory")
rm("file_or_dir")
cp("src", "dst")
mv("src", "dst")
files = glob("**/*.star")
```

### Process Execution
```python
result = run(["echo", "hello"])
print(result.stdout)        # "hello\n"
print(result.code)          # 0

result = shell("echo hello && pwd")
print(result.stdout)

# With options
result = run(["cmd"], cwd="/some/dir", env={"KEY": "value"})
```

### HTTP Requests
```python
resp = http_get("https://api.example.com/data")
print(resp.status)          # 200
print(resp.body)            # response body
print(resp.headers)         # {"content-type": "..."}

resp = http_post("https://api.example.com/data",
                 body='{"key": "value"}',
                 headers={"Content-Type": "application/json"})
```

### JSON
```python
data = {"name": "Blueprint", "version": 1}
json_str = json.encode(data)
json_pretty = json.encode(data, indent=2)

parsed = json.decode('{"key": "value"}')
```

### Time
```python
start = now()               # Unix timestamp as float
sleep(0.5)                  # Sleep for 500ms
elapsed = now() - start
```

### Console
```python
print("Hello", "World")     # Print to stdout
eprint("Error!")            # Print to stderr
name = input("Name: ")      # Read from stdin
```

### Builtins
```python
len([1, 2, 3])              # 3
range(5)                    # [0, 1, 2, 3, 4]
range(1, 5)                 # [1, 2, 3, 4]
sum([1, 2, 3])              # 6
min([3, 1, 2])              # 1
max([3, 1, 2])              # 3
sorted([3, 1, 2])           # [1, 2, 3]
reversed([1, 2, 3])         # [3, 2, 1]
enumerate(["a", "b"])       # [(0, "a"), (1, "b")]
zip([1, 2], ["a", "b"])     # [(1, "a"), (2, "b")]
str(123)                    # "123"
int("42")                   # 42
float("3.14")               # 3.14
bool(1)                     # True
list((1, 2, 3))             # [1, 2, 3]
type(42)                    # "int"
```

### String Methods
```python
s = "Hello, World!"
s.upper()                   # "HELLO, WORLD!"
s.lower()                   # "hello, world!"
s.strip()                   # Remove whitespace
s.split(",")                # ["Hello", " World!"]
s.replace("World", "BP")    # "Hello, BP!"
s.startswith("Hello")       # True
s.endswith("!")             # True
s.find("World")             # 7
", ".join(["a", "b", "c"])  # "a, b, c"
"Hi {}!".format("there")    # "Hi there!"
```

## Script Globals

Scripts have access to:
- `argv` - List of command-line arguments (first element is script path)
- `__file__` - Absolute path to the current script

```python
# script.star
print("Script:", __file__)
print("Args:", argv[1:])
```

```bash
bp run script.star -- arg1 arg2
```

## Concurrent Execution

Blueprint runs multiple scripts concurrently on a single Tokio runtime:

```bash
# Run all .star files in parallel (default: unlimited concurrency)
bp run "scripts/*.star"

# Limit to 4 concurrent scripts
bp run "scripts/*.star" -j 4

# Verbose output shows progress
bp run "scripts/*.star" -v
```

## Project Structure

```
blueprint/
├── Cargo.toml              # Workspace configuration
├── crates/
│   ├── blueprint_cli/      # CLI binary (bp)
│   ├── blueprint_core/     # Core types (Value, Error)
│   ├── blueprint_eval/     # Async evaluator
│   └── blueprint_parser/   # Starlark parser wrapper
└── examples/               # Example scripts
```

## Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Run examples
cargo run --bin bp -- run "examples/*.star"
```

## License

MIT
