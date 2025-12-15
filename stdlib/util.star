"""Utility functions for Blueprint."""

def map(fn, items):
    """Apply a function to each item and return the results as a list.

    Args:
        fn: Function to apply to each item
        items: Iterable of items

    Returns:
        List of results
    """
    return [fn(item) for item in items]

def filter(fn, items):
    """Filter items by a predicate function.

    Args:
        fn: Predicate function (returns bool)
        items: Iterable of items

    Returns:
        List of items where fn(item) is True
    """
    return [item for item in items if fn(item)]

def reduce(fn, items, initial=None):
    """Reduce items to a single value using a function.

    Args:
        fn: Function taking (accumulator, item) and returning new accumulator
        items: Iterable of items
        initial: Initial accumulator value (uses first item if not provided)

    Returns:
        Final accumulated value
    """
    items_list = list(items)
    if initial == None:
        if len(items_list) == 0:
            fail("reduce() of empty sequence with no initial value")
        acc = items_list[0]
        items_list = items_list[1:]
    else:
        acc = initial
    for item in items_list:
        acc = fn(acc, item)
    return acc

def sleep(seconds):
    """Sleep for a specified number of seconds.

    Args:
        seconds: Number of seconds to sleep (can be fractional, e.g., 0.5 for 500ms)

    Returns:
        OpResult (for use with after() and other combinators)
    """
    return __bp_sleep(seconds)

def now():
    """Get the current Unix timestamp.

    Returns:
        OpResult containing the current time as a float (seconds since epoch)
    """
    return __bp_now()

def _log_info(message):
    """Log an info message to stdout."""
    __bp_stdout("[INFO]", message)

def _log_warn(message):
    """Log a warning message to stdout."""
    __bp_stdout("[WARN]", message)

def _log_error(message):
    """Log an error message to stderr."""
    __bp_stderr("[ERROR] " + str(message))

def _log_debug(message):
    """Log a debug message to stdout."""
    __bp_stdout("[DEBUG]", message)

log = struct(
    info = _log_info,
    warn = _log_warn,
    error = _log_error,
    debug = _log_debug,
)
