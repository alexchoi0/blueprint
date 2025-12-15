"""Async sleep example - demonstrates parallel sleep operations."""

load("@bp", "sleep", "parallel")

s1 = sleep(1)
s2 = sleep(0.5)

parallel.gather([s1, s2])

print("hello world (both sleeps completed)")
