# Ruxgo

<p align="center">
    <img src="./guide/images/ruxgo-logo.svg" alt="ruxgo-logo" width="55%">
</p>

Ruxgo is a companion command-line tool for RuxOS.

**To start using Ruxgo**, learn more at [The Ruxgo Book](https://ruxgo.syswonder.org/).

## Installation

Ruxgo currently requires at least Rust version 1.75. You can install it with Cargo:

```sh
cargo install ruxgo
```

## Usage

Ruxgo supports building self-developed applications to run on linux or windows platforms, and also supports assembling and building Unikernel-based RuxOS and running applications on it. You just need to Write a `config_linux.toml` for linux or `config_win32.toml` for windows in the project directory.

The `ruxgo/apps/` directory places all the Toml files that have been tested, you can switch to either directory and follow the instructions to build the application.

- If you are developing your own application, you can refer to the template to write a Toml file, then put it in your project directory, and use ruxgo to build and run it.

- If you want to build an already supported app on ruxos, you need to copy `config_<platform>.toml` from `ruxgo/apps/<name>/ruxos` into `ruxos/apps/c/<name>`, then refer to the instructions and use ruxgo to build and run it.

- If you have your own app executable and want to run it on RuxOS, you can refer to the template under `ruxgo/apps/loader_app` and configure your own, then use ruxgo to build and run it.

**Note:** Refer to the README.md in each app directory for details. The following applications are already supported:

* [x] [redis](apps/redis)
* [x] [sqlite3](apps/sqlite3)
* [x] [iperf](apps/iperf)
* [x] [nginx](apps/nginx)
* [x] [loader_app](apps/loader_app)
* [x] helloworld
* [x] memtest
* [x] httpclient
* [x] httpserver
* [x] python3