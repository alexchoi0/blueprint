load("@bp/io", "write_file")
load("@bp/json", "encode")

data = {
    "name": "Blueprint",
    "version": "0.1.0",
    "features": ["starlark", "schema", "plan", "execute"],
}

encoded = encode(data)
write_file("/tmp/config.json", encoded)
