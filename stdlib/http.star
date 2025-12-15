"""HTTP operations for Blueprint."""

def get(url, headers=None):
    """Make an HTTP GET request.

    Args:
        url: URL to request
        headers: Optional dict of headers

    Returns:
        OpResult containing {status, headers, body}
    """
    return __bp_http_request("GET", url, None, headers)

def post(url, body, headers=None):
    """Make an HTTP POST request.

    Args:
        url: URL to request
        body: Request body (string)
        headers: Optional dict of headers

    Returns:
        OpResult containing {status, headers, body}
    """
    return __bp_http_request("POST", url, body, headers)

def put(url, body, headers=None):
    """Make an HTTP PUT request.

    Args:
        url: URL to request
        body: Request body (string)
        headers: Optional dict of headers

    Returns:
        OpResult containing {status, headers, body}
    """
    return __bp_http_request("PUT", url, body, headers)

def delete(url, headers=None):
    """Make an HTTP DELETE request.

    Args:
        url: URL to request
        headers: Optional dict of headers

    Returns:
        OpResult containing {status, headers, body}
    """
    return __bp_http_request("DELETE", url, None, headers)

def patch(url, body, headers=None):
    """Make an HTTP PATCH request.

    Args:
        url: URL to request
        body: Request body (string)
        headers: Optional dict of headers

    Returns:
        OpResult containing {status, headers, body}
    """
    return __bp_http_request("PATCH", url, body, headers)

def head(url, headers=None):
    """Make an HTTP HEAD request.

    Args:
        url: URL to request
        headers: Optional dict of headers

    Returns:
        OpResult containing {status, headers, body}
    """
    return __bp_http_request("HEAD", url, None, headers)

def request(method, url, body=None, headers=None):
    """Make an HTTP request with any method.

    Args:
        method: HTTP method (GET, POST, PUT, DELETE, PATCH, HEAD, etc.)
        url: URL to request
        body: Optional request body (string)
        headers: Optional dict of headers

    Returns:
        OpResult containing {status, headers, body}
    """
    return __bp_http_request(method, url, body, headers)
