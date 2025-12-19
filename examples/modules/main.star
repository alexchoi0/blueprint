print("=== Module Loading Example ===")

load("lib/utils.star", "greet", "add", "factorial", "PI")
load("lib/math.star", "square", "cube", "power")

print(f"PI = {PI}")

message = greet("Blueprint")
print(message)

result = add(10, 20)
print(f"add(10, 20) = {result}")

fact5 = factorial(5)
print(f"factorial(5) = {fact5}")

sq = square(7)
print(f"square(7) = {sq}")

cu = cube(3)
print(f"cube(3) = {cu}")

pw = power(2, 10)
print(f"power(2, 10) = {pw}")

print("\nAll module functions loaded and executed successfully!")
