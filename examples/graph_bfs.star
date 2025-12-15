"""Graph algorithms - BFS and DFS in Starlark."""

def make_graph():
    """Create an adjacency list graph."""
    return {
        "A": ["B", "C"],
        "B": ["A", "D", "E"],
        "C": ["A", "F"],
        "D": ["B"],
        "E": ["B", "F"],
        "F": ["C", "E"],
    }

def bfs(graph, start):
    """Breadth-first search using recursion."""
    return bfs_helper(graph, [start], [])

def bfs_helper(graph, queue, visited):
    """BFS helper with explicit queue."""
    if len(queue) == 0:
        return visited
    node = queue[0]
    rest = queue[1:]
    if node in visited:
        return bfs_helper(graph, rest, visited)
    new_visited = visited + [node]
    neighbors = [n for n in graph.get(node, []) if n not in new_visited]
    return bfs_helper(graph, rest + neighbors, new_visited)

def dfs(graph, start, visited = None):
    """Depth-first search."""
    if visited == None:
        visited = []

    if start in visited:
        return visited

    new_visited = visited + [start]
    for neighbor in graph.get(start, []):
        new_visited = dfs(graph, neighbor, new_visited)

    return new_visited

def find_path(graph, start, end, path = None):
    """Find a path between two nodes."""
    if path == None:
        path = []

    path = path + [start]

    if start == end:
        return path

    if start not in graph:
        return None

    for node in graph[start]:
        if node not in path:
            new_path = find_path(graph, node, end, path)
            if new_path:
                return new_path

    return None

def find_all_paths(graph, start, end, path = None):
    """Find all paths between two nodes."""
    if path == None:
        path = []

    path = path + [start]

    if start == end:
        return [path]

    if start not in graph:
        return []

    paths = []
    for node in graph[start]:
        if node not in path:
            new_paths = find_all_paths(graph, node, end, path)
            paths = paths + new_paths

    return paths

g = make_graph()

print("Graph:", g)
print("BFS from A:", bfs(g, "A"))
print("DFS from A:", dfs(g, "A"))
print("Path A -> F:", find_path(g, "A", "F"))
print("All paths A -> F:", find_all_paths(g, "A", "F"))
