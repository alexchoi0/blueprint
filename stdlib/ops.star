"""Composable operations for Blueprint.

These functions allow operating on OpResult values at planning time,
creating new operations that will be executed when the plan runs.
"""

def bp_bool(value):
    """Convert an OpResult to boolean.

    Args:
        value: An OpResult or literal value

    Returns:
        OpResult representing the boolean conversion
    """
    return __bp_bool(value)

def bp_int(value):
    """Convert an OpResult to integer.

    Args:
        value: An OpResult or literal value

    Returns:
        OpResult representing the integer conversion
    """
    return __bp_int(value)

def bp_float(value):
    """Convert an OpResult to float.

    Args:
        value: An OpResult or literal value

    Returns:
        OpResult representing the float conversion
    """
    return __bp_float(value)

def bp_str(value):
    """Convert an OpResult to string.

    Args:
        value: An OpResult or literal value

    Returns:
        OpResult representing the string conversion
    """
    return __bp_str(value)

def bp_len(value):
    """Get the length of an OpResult.

    Args:
        value: An OpResult (string, list, dict) or literal value

    Returns:
        OpResult representing the length
    """
    return __bp_len(value)

def bp_add(left, right):
    """Add two values.

    Args:
        left: First operand (OpResult or literal)
        right: Second operand (OpResult or literal)

    Returns:
        OpResult representing the addition
    """
    return __bp_add(left, right)

def bp_sub(left, right):
    """Subtract two values.

    Args:
        left: First operand (OpResult or literal)
        right: Second operand (OpResult or literal)

    Returns:
        OpResult representing the subtraction
    """
    return __bp_sub(left, right)

def bp_mul(left, right):
    """Multiply two values.

    Args:
        left: First operand (OpResult or literal)
        right: Second operand (OpResult or literal)

    Returns:
        OpResult representing the multiplication
    """
    return __bp_mul(left, right)

def bp_div(left, right):
    """Divide two values (true division).

    Args:
        left: First operand (OpResult or literal)
        right: Second operand (OpResult or literal)

    Returns:
        OpResult representing the division
    """
    return __bp_div(left, right)

def bp_floor_div(left, right):
    """Integer divide two values (floor division).

    Args:
        left: First operand (OpResult or literal)
        right: Second operand (OpResult or literal)

    Returns:
        OpResult representing the floor division
    """
    return __bp_floor_div(left, right)

def bp_mod(left, right):
    """Modulo of two values.

    Args:
        left: First operand (OpResult or literal)
        right: Second operand (OpResult or literal)

    Returns:
        OpResult representing the modulo
    """
    return __bp_mod(left, right)

def bp_neg(value):
    """Negate a value.

    Args:
        value: An OpResult or literal value

    Returns:
        OpResult representing the negation
    """
    return __bp_neg(value)

def bp_eq(left, right):
    """Check equality of two values.

    Args:
        left: First operand (OpResult or literal)
        right: Second operand (OpResult or literal)

    Returns:
        OpResult representing equality check
    """
    return __bp_eq(left, right)

def bp_ne(left, right):
    """Check inequality of two values.

    Args:
        left: First operand (OpResult or literal)
        right: Second operand (OpResult or literal)

    Returns:
        OpResult representing inequality check
    """
    return __bp_ne(left, right)

def bp_lt(left, right):
    """Check if left < right.

    Args:
        left: First operand (OpResult or literal)
        right: Second operand (OpResult or literal)

    Returns:
        OpResult representing the comparison
    """
    return __bp_lt(left, right)

def bp_le(left, right):
    """Check if left <= right.

    Args:
        left: First operand (OpResult or literal)
        right: Second operand (OpResult or literal)

    Returns:
        OpResult representing the comparison
    """
    return __bp_le(left, right)

def bp_gt(left, right):
    """Check if left > right.

    Args:
        left: First operand (OpResult or literal)
        right: Second operand (OpResult or literal)

    Returns:
        OpResult representing the comparison
    """
    return __bp_gt(left, right)

def bp_ge(left, right):
    """Check if left >= right.

    Args:
        left: First operand (OpResult or literal)
        right: Second operand (OpResult or literal)

    Returns:
        OpResult representing the comparison
    """
    return __bp_ge(left, right)

def bp_not(value):
    """Logical NOT of a value.

    Args:
        value: An OpResult or literal value

    Returns:
        OpResult representing the logical NOT
    """
    return __bp_not(value)

def bp_concat(left, right):
    """Concatenate two values (strings or lists).

    Args:
        left: First operand (OpResult or literal)
        right: Second operand (OpResult or literal)

    Returns:
        OpResult representing the concatenation
    """
    return __bp_concat(left, right)

def bp_contains(haystack, needle):
    """Check if needle is in haystack.

    Args:
        haystack: Collection to search (OpResult or literal)
        needle: Value to find (OpResult or literal)

    Returns:
        OpResult representing the containment check
    """
    return __bp_contains(haystack, needle)
