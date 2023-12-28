# Ruxgo 要实现的功能

既然是组件化的新尝试，功能先暂定如下：

- **编译器类型多样化：** C/C++ 有多种编译器可供选择，如 GCC、Clang、MSVC 等。不同编译器对语言标准的支持程度、性能优化的策略等可能有所不同，可利用此特点组合出最佳性能。

- **中间过程多样化：** 可以根据实际需求选择生成静态库，动态库或者一系列中间文件，如预处理后的代码、汇编代码、目标文件等。这些中间文件可以用于调试和优化，也可以用于构建过程的加速。

- **增量构建：** 采用增量构建的方式缩短构建时间，只重新编译修改过的文件或受影响的模块。

- **并行构建：** 将构建任务并行化，利用多核处理器来同时编译多个文件。

- **依赖管理：** 对于要依赖的外部库和组件，可以充分利用 [crate.io](https://crates.io/) 的生态，或者使用一些包管理工具。

- **可移植性：** 在代码中尽量使用平台无关的 API 和规范，避免平台特定的实现方式。

- **编译错误诊断：** 使用更友好的编译器错误输出显示，包括清晰的错误提示、指向源代码行数等信息。