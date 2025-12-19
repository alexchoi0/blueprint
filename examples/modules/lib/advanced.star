load("math.star", "square", "power")

def square_sum(a, b):
    return square(a) + square(b)

def pow_chain(base, exp1, exp2):
    return power(power(base, exp1), exp2)
