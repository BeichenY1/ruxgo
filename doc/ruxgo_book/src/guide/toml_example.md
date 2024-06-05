# TOML 文件示例

Ruxgo 有如下三种适用场景:

- 在本地构建自我开发的应用程序；

- 在 RuxOS 上开发或移植应用程序；

- 在 RuxOS 上直接运行应用程序可执行文件。

下面将以 [sqlite3 ](https://github.com/syswonder/ruxgo/tree/master/apps/sqlite3)为例，分别给出这三种情况下对应的 TOML 文件示例：

**（1）在本地主机（linux平台）构建 Sqlite3:**

```toml
[build]
compiler = "gcc"

[[targets]]
name = "libsqlite3"
src = "./sqlite-amalgamation-3410100"
src_exclude = ["shell.c"]
include_dir = "./sqlite-amalgamation-3410100"
type = "static"
cflags = "-w -DSQLITE_THREADSAFE=0 -DSQLITE_OMIT_FLOATING_POINT -DSQLITE_OMIT_LOAD_EXTENSION"
archive = "ar"
ldflags = "rcs"

[[targets]]
name = "local_sqlite3"
src = "./"
src_only = ["main.c"]
include_dir = "./"
type = "exe"
cflags = "-w -g"
ldflags = ""
deps = ["libsqlite3"]
```

**（2）在 RuxOS 上构建 Sqlite3:**

```toml
[build]
compiler = "gcc"

[os]
name = "ruxos"
services = ["fp_simd","alloc","paging","fs","blkfs"]
ulib = "ruxlibc"

[os.platform]
name = "x86_64-qemu-q35"
smp = "2"
mode = "release"
log = "error"

[os.platform.qemu]
blk = "y"
graphic = "n"

[[targets]]
name = "libsqlite3"
src = "./sqlite-amalgamation-3410100"
src_exclude = ["shell.c"]
include_dir = "./sqlite-amalgamation-3410100"
type = "static"
cflags = "-w -DSQLITE_THREADSAFE=0 -DSQLITE_OMIT_FLOATING_POINT -DSQLITE_OMIT_LOAD_EXTENSION"
archive = "ar"
ldflags = "rcs"

[[targets]]
name = "ruxos_sqlite3"
src = "./"
src_only = ["main.c"]
include_dir = "./"
type = "exe"
cflags = "-w -g"
linker = "rust-lld -flavor gnu"
ldflags = ""
deps = ["libsqlite3"]
```

**（3）在 RuxOS 上运行 Sqlite3 可执行文件:**

```toml
[build]
compiler = "gcc"
app = "your_app_path"   # 在这里指定可执行文件路径

[os]
name = "ruxos"
services = ["fp_simd","alloc","paging","fs","blkfs"]
ulib = "ruxlibc"

[os.platform]
name = "x86_64-qemu-q35"
smp = "2"
mode = "release"
log = "error"

[os.platform.qemu]
blk = "y"
graphic = "n"
args = ""   # 在这里指定应用程序所需参数
```