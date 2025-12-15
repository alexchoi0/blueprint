"""Streaming combinators for Blueprint.

These primitives work with results as they complete, rather than
waiting for all operations to finish. Useful for:
- Processing results incrementally
- Early termination patterns
- Resource-efficient pipelines

Note: Full streaming requires the streaming executor. With the
level-based executor, some functions may block until a level completes.
"""

def as_completed(ops):
    """Iterate over ops as they complete.

    Yields results in completion order, not list order.
    Enables processing results as soon as they're available.

    Args:
        ops: List of OpResult values

    Yields:
        Results in completion order

    Example:
        for result in as_completed([http.get(url) for url in urls]):
            process(result)  # Runs as each completes
    """
    # TODO: Requires streaming executor support
    # Fallback: return all results (loses ordering benefit)
    return __bp_gather(ops)

def first(ops):
    """Get the first op to complete.

    Unlike `any`, this returns based on completion time,
    not list position.

    Args:
        ops: List of OpResult values

    Returns:
        Result of first op to complete
    """
    # TODO: Requires streaming executor support
    # Fallback: return first in list
    return __bp_any(ops)

def take(n, ops):
    """Get the first N ops to complete.

    Args:
        n: Number of results to take
        ops: List of OpResult values

    Returns:
        List of first N results (by completion time)
    """
    # TODO: Requires streaming executor for true completion-order
    # Fallback: require at_least N and return all
    return __bp_at_least(n, ops)

def race(ops):
    """Race ops against each other, return winner.

    Semantically equivalent to first(), but named to
    emphasize the racing/competitive aspect.

    Args:
        ops: List of OpResult values

    Returns:
        Result of first op to complete
    """
    return first(ops)

def zip_completed(ops_a, ops_b):
    """Zip two op lists by completion order.

    Pairs results as they complete from each list.

    Args:
        ops_a: First list of OpResult values
        ops_b: Second list of OpResult values

    Returns:
        List of (result_a, result_b) tuples
    """
    # TODO: Requires streaming executor support
    # Fallback: zip by list order after all complete
    results_a = __bp_gather(ops_a)
    results_b = __bp_gather(ops_b)
    return zip(results_a, results_b)

def pipeline(fns, items):
    """Process items through a pipeline of functions.

    Each function receives items as they complete from
    the previous stage, enabling streaming pipelines.

    Args:
        fns: List of functions, each taking an item
        items: Initial list of items

    Returns:
        Final results after all pipeline stages

    Example:
        results = pipeline([fetch, parse, validate], urls)
    """
    current = items
    for fn in fns:
        current = [fn(item) for item in current]
    return __bp_gather(current)
