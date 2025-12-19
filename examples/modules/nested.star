print("=== Nested Module Loading ===")

load("lib/advanced.star", "square_sum", "pow_chain")

result = square_sum(3, 4)
print("square_sum(3, 4) =", result)

result2 = pow_chain(2, 2, 3)
print("pow_chain(2, 2, 3) =", result2)

print("\nNested modules work!")
