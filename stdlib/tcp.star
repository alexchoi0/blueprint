"""TCP socket operations for Blueprint using event primitives."""

def connect(host, port):
    """Connect to a TCP server.

    Args:
        host: Host to connect to
        port: Port to connect to

    Returns:
        Event source handle for the connection
    """
    return __bp_event_source("tcp_connect", {"host": host, "port": port})

def listen(host, port):
    """Start a TCP listener.

    Args:
        host: Host to bind to (e.g., "0.0.0.0")
        port: Port to listen on

    Returns:
        Event source handle for the listener
    """
    return __bp_event_source("tcp_listen", {"host": host, "port": port})

def send(handle, data):
    """Send data on a TCP connection.

    Args:
        handle: Connection handle from connect() or accept event
        data: Data to send (string)

    Returns:
        Number of bytes sent
    """
    return __bp_event_write(handle, data)

def recv(handle, timeout_ms=-1):
    """Receive data from a TCP connection.

    Args:
        handle: Connection handle
        timeout_ms: Timeout in milliseconds (-1 for infinite)

    Returns:
        Received data as string, or None on timeout
    """
    event = __bp_event_poll([handle], timeout_ms)
    if event == None:
        return None
    if event["type"] == "closed":
        return None
    if event["type"] == "error":
        fail("TCP recv failed: " + event["data"]["message"])
    return event["data"]["data"]

def close(handle):
    """Close a TCP connection or listener.

    Args:
        handle: Handle to close
    """
    __bp_event_source_close(handle)

def accept(listener_handle, timeout_ms=-1):
    """Accept a connection on a TCP listener.

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
        fail("TCP accept failed: " + event["data"]["message"])
    return event["data"]["client_handle"]

def serve(host, port, handler, max_connections=100):
    """Run a simple TCP server.

    Calls handler(client_handle) for each incoming connection.
    The handler is responsible for closing the client handle.

    Args:
        host: Host to bind to
        port: Port to listen on
        handler: Function to call for each connection
        max_connections: Maximum connections to handle before stopping
    """
    listener = listen(host, port)
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

def request(host, port, data, timeout_ms=30000):
    """Make a simple request/response over TCP.

    Connects, sends data, receives response, and closes.

    Args:
        host: Host to connect to
        port: Port to connect to
        data: Data to send
        timeout_ms: Timeout in milliseconds

    Returns:
        Response data as string
    """
    conn = connect(host, port)
    send(conn, data)
    response = recv(conn, timeout_ms)
    close(conn)
    return response
