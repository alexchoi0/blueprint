"""Command execution for Blueprint."""

def run(command, args=None):
    """Execute a command and wait for completion.

    Args:
        command: Command to execute
        args: Optional list of arguments

    Returns:
        OpResult containing {code, stdout, stderr}
    """
    return __bp_exec(command, args)

def shell(command):
    """Execute a command through the shell.

    This runs the command through sh -c, allowing shell features like
    pipes and redirection.

    Args:
        command: Shell command string

    Returns:
        OpResult containing {code, stdout, stderr}
    """
    return __bp_exec("sh", ["-c", command])

def env(name, default=None):
    """Get an environment variable.

    Args:
        name: Name of the environment variable
        default: Default value if not set

    Returns:
        OpResult containing the environment variable value or default
    """
    return __bp_env_get(name, default)
