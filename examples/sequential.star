"""Sequential execution example - demonstrates after() and sequence()."""

load("@bp", "sleep", "now", "parallel")

start = now()

a = sleep(0.1)
b = parallel.after(a, sleep(0.1))
c = parallel.after(b, sleep(0.1))

parallel.all([a, b, c])

end = now()

print("Sequential sleep with after(): ~0.3s expected")
print("Elapsed:", end - start)

start2 = now()

sleeps = parallel.sequence([
    sleep(0.05),
    sleep(0.05),
    sleep(0.05),
    sleep(0.05),
])

end2 = now()

print("Sequential sleep with sequence(): ~0.2s expected")
print("Elapsed:", end2 - start2)

start3 = now()

parallel_sleeps = parallel.all([
    sleep(0.1),
    sleep(0.1),
    sleep(0.1),
])

end3 = now()

print("Parallel sleep: ~0.1s expected")
print("Elapsed:", end3 - start3)
