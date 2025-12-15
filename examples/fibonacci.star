"""Fibonacci sequence - pure computation example."""

def fib(n):
    """Calculate nth Fibonacci number recursively."""
    if n <= 1:
        return n
    return fib(n - 1) + fib(n - 2)

def fib_iter(n):
    """Calculate nth Fibonacci number iteratively."""
    if n <= 1:
        return n
    a, b = 0, 1
    for _ in range(2, n + 1):
        a, b = b, a + b
    return b

def fib_sequence(n):
    """Generate first n Fibonacci numbers."""
    result = []
    a, b = 0, 1
    for _ in range(n):
        result.append(a)
        a, b = b, a + b
    return result

print("Fibonacci sequence (first 20):")
seq = fib_sequence(20)
print(seq)

print("Fib(10) =", fib_iter(10))
print("Fib(15) =", fib_iter(15))
print("Fib(20) =", fib_iter(20))
