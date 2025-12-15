"""Parallel execution combinators for Blueprint."""

def gather(ops):
    """Wait for all operations to complete and gather results.

    Like asyncio.gather - waits for all provided operations to complete
    and returns a list of their results.

    Args:
        ops: List of OpResult values

    Returns:
        OpResult containing a list of all results
    """
    return __bp_gather(ops)

def race(ops):
    """Wait for any operation to complete.

    Returns the result of the first operation to complete successfully.

    Args:
        ops: List of OpResult values

    Returns:
        OpResult containing the first successful result
    """
    return __bp_any(ops)

def at_least(count, ops):
    """Wait for at least N operations to complete.

    Args:
        count: Minimum number of operations that must complete
        ops: List of OpResult values

    Returns:
        OpResult containing True if at least count operations completed
    """
    return __bp_at_least(count, ops)

def at_most(count, ops):
    """Wait for at most N operations to complete.

    Args:
        count: Maximum number of operations that should complete
        ops: List of OpResult values

    Returns:
        OpResult containing True if at most count operations completed
    """
    return __bp_at_most(count, ops)

def after(dependency, op):
    """Execute op only after dependency completes.

    Creates an artificial dependency to force sequential execution
    of operations that would otherwise run in parallel.

    Args:
        dependency: OpResult that must complete first
        op: OpResult to execute after dependency

    Returns:
        OpResult containing op's result (after dependency completes)

    Example:
        a = http.get(url1)
        b = after(a, http.get(url2))  # b waits for a
        c = after(b, http.get(url3))  # c waits for b
    """
    return __bp_after(dependency, op)

def sequence(ops):
    """Execute operations sequentially, in order.

    Even if operations have no data dependencies, they will
    execute one at a time in list order.

    Args:
        ops: List of OpResult values to execute sequentially

    Returns:
        OpResult containing a list of all results (in order)

    Example:
        results = sequence([
            http.get(url1),
            http.get(url2),
            http.get(url3),
        ])
    """
    if len(ops) == 0:
        return __bp_gather([])
    if len(ops) == 1:
        return __bp_gather(ops)

    chained = [ops[0]]
    for i in range(1, len(ops)):
        chained.append(__bp_after(chained[i - 1], ops[i]))

    return __bp_gather(chained)
