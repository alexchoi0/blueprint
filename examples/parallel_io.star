"""Parallel I/O example - demonstrates concurrent file operations."""

load("@bp", "read_file", "write_file", "parallel")

w1 = write_file("/tmp/bp_test1.txt", "Hello from file 1")
w2 = write_file("/tmp/bp_test2.txt", "Hello from file 2")

writes_done = parallel.gather([w1, w2])

r1 = parallel.after(writes_done, read_file("/tmp/bp_test1.txt"))
r2 = parallel.after(writes_done, read_file("/tmp/bp_test2.txt"))

parallel.gather([r1, r2])

print("All done!")
