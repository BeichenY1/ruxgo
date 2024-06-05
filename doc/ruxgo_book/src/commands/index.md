# Ruxgo 命令

参数信息：

```shell
A Cargo-like build tool for building C and C++ applications

Usage: ruxgo [OPTIONS] [CHOICES]... [COMMAND]

Commands:
  init    Initialize a new project Defaults to C++ if no language is specified
  pkg     Package management
  config  Configuration settings
  help    Print this message or the help of the given subcommand(s)

Arguments:
  [CHOICES]...  Choose which parts to delete

Options:
  -b, --build                   Build your project
  -c, --clean                   Clean the obj and bin intermediates
  -r, --run                     Run the executable
      --path <PATH>             Path argument to pass to switch to the specified directory
      --bin-args=<BIN_ARGS>...  Arguments to pass to the executable when running
      --gen-cc                  Generate compile_commands.json
      --gen-vsc                 Generate .vscode/c_cpp_properties.json
  -h, --help                    Print help
  -V, --version                 Print version
```
* [通用命令](./general-commands.md)

* [构建命令](./build-commands.md)
