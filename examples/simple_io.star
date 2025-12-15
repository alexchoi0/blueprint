"""Simple I/O test - demonstrates sequential operations using sequence()."""

load("@bp", "write_file", "read_file", "parallel")

parallel.sequence([
    write_file("/tmp/bp_simple.txt", "Hello Blueprint!"),
    read_file("/tmp/bp_simple.txt"),
])

print("Done!")
