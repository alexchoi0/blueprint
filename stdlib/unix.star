"""Unix socket operations for Blueprint using event primitives.

Note: This module only works on Unix-like systems (Linux, macOS).
"""

def connect(path):
    """Connect to a Unix domain socket.

    Args:
        path: Path to the Unix socket

    Returns:
        Event source handle for the connection
    """
    return __bp_event_source("unix_connect", {"path": path})

def listen(path):
    """Start a Unix socket listener.

    Args:
        path: Path for the Unix socket

    Returns:
        Event source handle for the listener
    """
    return __bp_event_source("unix_listen", {"path": path})

def send(handle, data):
    """Send data on a Unix socket connection.

    Args:
        handle: Connection handle from connect() or accept event
        data: Data to send (string)

    Returns:
        Number of bytes sent
    """
    return __bp_event_write(handle, data)

def recv(handle, timeout_ms=-1):
    """Receive data from a Unix socket connection.

    Args:
        handle: Connection handle
        timeout_ms: Timeout in milliseconds (-1 for infinite)

    Returns:
        Received data as string, or None on timeout/close
    """
    event = __bp_event_poll([handle], timeout_ms)
    if event == None:
        return None
    if event["type"] == "closed":
        return None
    if event["type"] == "error":
        fail("Unix recv failed: " + event["data"]["message"])
    return event["data"]["data"]

def close(handle):
    """Close a Unix socket connection or listener.

    Args:
        handle: Handle to close
    """
    __bp_event_source_close(handle)

def accept(listener_handle, timeout_ms=-1):
    """Accept a connection on a Unix socket listener.

    Args:
        listener_handle: Listener handle from listen()
        timeout_ms: Timeout in milliseconds (-1 for infinite)

    Returns:
        Client connection handle, or None on timeout
    """
    event = __bp_event_poll([listener_handle], timeout_ms)
    if event == None:
        return None
    if event["type"] == "error":
        fail("Unix accept failed: " + event["data"]["message"])
    return event["data"]["client_handle"]

def serve(path, handler, max_connections=100):
    """Run a simple Unix socket server.

    Calls handler(client_handle) for each incoming connection.
    The handler is responsible for closing the client handle.

    Args:
        path: Path for the Unix socket
        handler: Function to call for each connection
        max_connections: Maximum connections to handle before stopping
    """
    listener = listen(path)
    _serve_loop(listener, handler, max_connections)
    close(listener)

def _serve_loop(listener, handler, remaining):
    """Recursive server loop."""
    if remaining <= 0:
        return
    client = accept(listener, timeout_ms=1000)
    if client != None:
        handler(client)
    _serve_loop(listener, handler, remaining - 1)

def request(path, data, timeout_ms=30000):
    """Make a simple request/response over Unix socket.

    Connects, sends data, receives response, and closes.

    Args:
        path: Path to the Unix socket
        data: Data to send
        timeout_ms: Timeout in milliseconds

    Returns:
        Response data as string
    """
    conn = connect(path)
    send(conn, data)
    response = recv(conn, timeout_ms)
    close(conn)
    return response
