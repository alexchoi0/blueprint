"""UDP socket operations for Blueprint using event primitives."""

def bind(host, port):
    """Bind a UDP socket.

    Args:
        host: Host to bind to (e.g., "0.0.0.0")
        port: Port to bind to

    Returns:
        Event source handle for the socket
    """
    return __bp_event_source("udp", {"host": host, "port": port})

def send_to(handle, data, host, port):
    """Send a datagram to a specific address.

    Args:
        handle: Socket handle from bind()
        data: Data to send (string)
        host: Destination host
        port: Destination port

    Returns:
        Number of bytes sent
    """
    return __bp_event_write(handle, data, host, port)

def recv_from(handle, timeout_ms=-1):
    """Receive a datagram.

    Args:
        handle: Socket handle
        timeout_ms: Timeout in milliseconds (-1 for infinite)

    Returns:
        Dict with {data, host, port}, or None on timeout
    """
    event = __bp_event_poll([handle], timeout_ms)
    if event == None:
        return None
    if event["type"] == "error":
        fail("UDP recv failed: " + event["data"]["message"])
    return {
        "data": event["data"]["data"],
        "host": event["data"]["from_host"],
        "port": event["data"]["from_port"],
    }

def close(handle):
    """Close a UDP socket.

    Args:
        handle: Socket handle to close
    """
    __bp_event_source_close(handle)

def request(host, port, data, bind_port=0, timeout_ms=5000):
    """Make a simple UDP request/response.

    Binds to a local port, sends a datagram, waits for response, and closes.

    Args:
        host: Destination host
        port: Destination port
        data: Data to send
        bind_port: Local port to bind (0 for any)
        timeout_ms: Timeout in milliseconds

    Returns:
        Response dict with {data, host, port}, or None on timeout
    """
    sock = bind("0.0.0.0", bind_port)
    send_to(sock, data, host, port)
    response = recv_from(sock, timeout_ms)
    close(sock)
    return response
