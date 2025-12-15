"""Blueprint Standard Library.

This is the main entry point for the Blueprint standard library.
Import this module to access all Blueprint functionality.

Example:
    load("@bp", "read_file", "write_file", "http")

    config = read_file("config.json")
    response = http.get("https://api.example.com/data")
    write_file("output.json", response["body"])
"""

load("@bp/io",
    "read_file",
    "write_file",
    "append_file",
    "delete_file",
    "file_exists",
    "is_dir",
    "is_file",
    "mkdir",
    "rmdir",
    "list_dir",
    "copy_file",
    "move_file",
    "file_size",
)

load("@bp/http",
    http_get = "get",
    http_post = "post",
    http_put = "put",
    http_delete = "delete",
    http_patch = "patch",
    http_head = "head",
    http_request = "request",
)

load("@bp/json",
    json_encode = "encode",
    json_decode = "decode",
    json_load = "load_file",
    json_save = "save_file",
)

load("@bp/exec",
    "run",
    "shell",
    "env",
)

load("@bp/parallel",
    "gather",
    "race",
    "at_least",
    "at_most",
    "after",
    "sequence",
)

load("@bp/util",
    "map",
    "filter",
    "reduce",
    "sleep",
    "now",
    "log",
)

load("@bp/ops",
    "bp_bool",
    "bp_int",
    "bp_float",
    "bp_str",
    "bp_len",
    "bp_add",
    "bp_sub",
    "bp_mul",
    "bp_div",
    "bp_floor_div",
    "bp_mod",
    "bp_neg",
    "bp_eq",
    "bp_ne",
    "bp_lt",
    "bp_le",
    "bp_gt",
    "bp_ge",
    "bp_not",
    "bp_concat",
    "bp_contains",
)

io = struct(
    read_file = read_file,
    write_file = write_file,
    append_file = append_file,
    delete_file = delete_file,
    file_exists = file_exists,
    is_dir = is_dir,
    is_file = is_file,
    mkdir = mkdir,
    rmdir = rmdir,
    list_dir = list_dir,
    copy_file = copy_file,
    move_file = move_file,
    file_size = file_size,
)

http = struct(
    get = http_get,
    post = http_post,
    put = http_put,
    delete = http_delete,
    patch = http_patch,
    head = http_head,
    request = http_request,
)

json = struct(
    encode = json_encode,
    decode = json_decode,
    load_file = json_load,
    save_file = json_save,
)

exec = struct(
    run = run,
    shell = shell,
    env = env,
)

parallel = struct(
    gather = gather,
    race = race,
    at_least = at_least,
    at_most = at_most,
    after = after,
    sequence = sequence,
)

util = struct(
    map = map,
    filter = filter,
    reduce = reduce,
    sleep = sleep,
    now = now,
    log = log,
)

ops = struct(
    bool = bp_bool,
    int = bp_int,
    float = bp_float,
    str = bp_str,
    len = bp_len,
    add = bp_add,
    sub = bp_sub,
    mul = bp_mul,
    div = bp_div,
    floor_div = bp_floor_div,
    mod = bp_mod,
    neg = bp_neg,
    eq = bp_eq,
    ne = bp_ne,
    lt = bp_lt,
    le = bp_le,
    gt = bp_gt,
    ge = bp_ge,
    not_ = bp_not,
    concat = bp_concat,
    contains = bp_contains,
)
