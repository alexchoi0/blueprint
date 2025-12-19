def square(x):
    return x * x

def cube(x):
    return x * x * x

def power(base, exp):
    result = 1
    for i in range(exp):
        result = result * base
    return result

def sum_list(items):
    total = 0
    for item in items:
        total = total + item
    return total
