print("=== Module Loading with Aliases ===")

load("lib/utils.star", say_hello="greet", sum="add")
load("lib/math.star", sq="square", pow="power")

msg = say_hello("World")
print(msg)

result = sum(5, 3)
print(f"sum(5, 3) = {result}")

print("sq(9) =", sq(9))
print("pow(3, 4) =", pow(3, 4))
