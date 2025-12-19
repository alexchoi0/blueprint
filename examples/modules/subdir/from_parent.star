print("=== Loading from Parent Directory ===")

load("../lib/utils.star", "greet", "PI")
load("../lib/math.star", "square")

print(f"PI = {PI}")
print(greet("Subdir"))
print("square(5) =", square(5))
