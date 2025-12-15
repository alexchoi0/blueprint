load("@bp/io", "mkdir", "write_file", "list_dir")

mkdir("/tmp/blueprint_test", recursive=True)

files = ["one.txt", "two.txt", "three.txt"]
for i, name in enumerate(files):
    path = "/tmp/blueprint_test/" + name
    write_file(path, "File %d: %s" % (i + 1, name))

listing = list_dir("/tmp/blueprint_test")
