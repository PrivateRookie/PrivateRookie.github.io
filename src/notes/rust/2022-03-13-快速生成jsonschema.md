# 快速生成 jsonschema

json 格式因其简单易用且与动态文件交互方便的特性被广泛应用, 也不乏有将 json 当作配置的应用场景. 当用于配置文件时
json 格式通常有确定的数据类型, 这时候我们可以依靠 jsonschema 对 json 数据类型进行描述.
但 jsonschema 规范更多, 也更加复杂, 手写复杂类型 jsonschema 非常困难, 所以又诞生了从 json 或其他描述生成 jsonschema 的工具.

这里介绍一种借助 Rust 和 [schema-rs](https://docs.rs/schemars/latest/schemars/) 快速生成 jsonschema 并应用于
json, yaml, toml 等文件校验补全的方法.

其原理是借助 Rust 静态强类型语言和强大的过程宏, 首先将期望的数据类型在 Rust 中表达出来, 然后使用 `schema-rs` 提供的过程宏生成结构体对应的 jsonschema. 接着在 vscode 中把生成的 jsonschema 与需校验的文件关联起来即可.

假设要添加如下配置文件校验

```rust
/// service config
pub struct Config {
    /// set service workspace dir
    workspace: PathBuf,
    /// max worker
    #[serde(d)]
    max_workers: usize
}
```