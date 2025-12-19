print("=== Parallel Execution Example ===")

def slow_task(name, delay):
    print(f"[{name}] Starting...")
    sleep(delay)
    print(f"[{name}] Done after {delay}s")
    return name

start = now()

results = parallel([
    lambda: slow_task("Task A", 0.3),
    lambda: slow_task("Task B", 0.2),
    lambda: slow_task("Task C", 0.1),
])

elapsed = now() - start

print(f"\nResults: {results}")
print("Total time:", elapsed, "s (sequential would be 0.6s)")
