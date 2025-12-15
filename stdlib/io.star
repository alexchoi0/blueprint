"""File I/O operations for Blueprint."""

def read_file(path):
    """Read a file and return its contents as a string.

    Args:
        path: Path to the file to read

    Returns:
        OpResult containing file contents as a string
    """
    return __bp_read_file(path)

def write_file(path, content):
    """Write content to a file.

    Args:
        path: Path to the file to write
        content: Content to write (string)

    Returns:
        OpResult
    """
    return __bp_write_file(path, content)

def append_file(path, content):
    """Append content to a file.

    Args:
        path: Path to the file to append to
        content: Content to append (string)

    Returns:
        OpResult
    """
    return __bp_append_file(path, content)

def delete_file(path):
    """Delete a file.

    Args:
        path: Path to the file to delete

    Returns:
        OpResult
    """
    return __bp_delete_file(path)

def file_exists(path):
    """Check if a file exists.

    Args:
        path: Path to check

    Returns:
        OpResult containing True if file exists, False otherwise
    """
    return __bp_file_exists(path)

def is_dir(path):
    """Check if a path is a directory.

    Args:
        path: Path to check

    Returns:
        OpResult containing True if path is a directory
    """
    return __bp_is_dir(path)

def is_file(path):
    """Check if a path is a file.

    Args:
        path: Path to check

    Returns:
        OpResult containing True if path is a file
    """
    return __bp_is_file(path)

def mkdir(path, recursive=False):
    """Create a directory.

    Args:
        path: Path to the directory to create
        recursive: If True, create parent directories as needed

    Returns:
        OpResult
    """
    return __bp_mkdir(path, recursive)

def rmdir(path, recursive=False):
    """Remove a directory.

    Args:
        path: Path to the directory to remove
        recursive: If True, remove contents recursively

    Returns:
        OpResult
    """
    return __bp_rmdir(path, recursive)

def list_dir(path):
    """List contents of a directory.

    Args:
        path: Path to the directory to list

    Returns:
        OpResult containing list of filenames
    """
    return __bp_list_dir(path)

def copy_file(src, dst):
    """Copy a file.

    Args:
        src: Source file path
        dst: Destination file path

    Returns:
        OpResult
    """
    return __bp_copy_file(src, dst)

def move_file(src, dst):
    """Move/rename a file.

    Args:
        src: Source file path
        dst: Destination file path

    Returns:
        OpResult
    """
    return __bp_move_file(src, dst)

def file_size(path):
    """Get the size of a file in bytes.

    Args:
        path: Path to the file

    Returns:
        OpResult containing file size in bytes
    """
    return __bp_file_size(path)
