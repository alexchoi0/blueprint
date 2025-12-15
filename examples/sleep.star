"""Sleep example - demonstrates async timer."""

load("@bp/util", "sleep", "now")

print("Testing sleep functionality...")

start = now()

print("Sleeping 0.1s...")
sleep(0.1)

print("Sleeping 0.25s...")
sleep(0.25)

print("Sleeping 0.5s...")
sleep(0.5)

print("Sleeping 1s (as integer)...")
sleep(1)

elapsed = now() - start
print("Total elapsed:", elapsed, "seconds")
print("Expected: ~1.85 seconds")
