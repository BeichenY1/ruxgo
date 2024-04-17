# 运行不同的app

在`ruxgo/apps/`目录下放置了所有经过测试的 Toml 文件。

- 如果你正在开发自己的应用程序并希望构建和运行它，你可以参考模板写一个 Toml 文件，然后把它放在你的项目目录下，即可使用  ruxgo 来构建和运行它。

- 如果你想在 RuxOS 上构建一个已经支持的应用程序，你需要将`config_linux.toml`从`ruxgo/apps/<name>/ruxos`复制到`ruxos/apps/c/<name>`，然后参考说明使用 ruxgo 构建并运行它。

- 如果你有自己的应用程序可执行文件，并希望在 RuxOS 上运行它，你可以参考`ruxgo/apps/loader_app`下的模板并配置你自己的 Toml 文件，然后使用 ruxgo 来构建和运行它。

**注:** 有关详细信息，请参阅每个 app 目录下的 README.md。以下应用程序已获支持:

* [x] [redis](https://github.com/syswonder/ruxgo/tree/master/apps/redis)
* [x] [sqlite3](https://github.com/syswonder/ruxgo/tree/master/apps/sqlite3)
* [x] [iperf](https://github.com/syswonder/ruxgo/tree/master/apps/iperf)
* [x] [loader_app](https://github.com/syswonder/ruxgo/tree/master/apps/loader_app)
* [x] helloworld
* [x] memtest
* [x] httpclient
* [x] httpserver
* [x] nginx
