"""JSON utilities for Blueprint."""

def encode(value):
    """Encode a value as a JSON string.

    Args:
        value: Value to encode (dict, list, string, number, bool, None)

    Returns:
        OpResult containing the JSON string
    """
    return __bp_json_encode(value)

def decode(string):
    """Decode a JSON string into a value.

    Args:
        string: JSON string to decode

    Returns:
        OpResult containing the decoded value
    """
    return __bp_json_decode(string)

def load_file(path):
    """Load and decode JSON from a file.

    Args:
        path: Path to the JSON file

    Returns:
        OpResult containing the decoded JSON
    """
    content = __bp_read_file(path)
    return __bp_json_decode(content)

def save_file(path, value):
    """Encode value as JSON and save to a file.

    Args:
        path: Path to save the JSON file
        value: Value to encode and save

    Returns:
        OpResult (None on success)
    """
    content = __bp_json_encode(value)
    return __bp_write_file(path, content)
