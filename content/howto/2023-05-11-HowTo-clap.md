# \[HowTo\] clap

HowTo 系列旨在提供常见工程问题解决办法, 多包含示例代码. 本期的主题是命令行解析工具 [clap](https://docs.rs/clap/latest/clap/index.html)


- [\[HowTo\] clap](#howto-clap)
  - [添加依赖](#添加依赖)
  - [从环境变量读取参数值](#从环境变量读取参数值)
  - [校验参数/转换](#校验参数转换)
  - [自动展开](#自动展开)
  - [解析 enum 值](#解析-enum-值)



## 添加依赖

```bash
cargo add clap -F derive -F env
```


## 从环境变量读取参数值

使用 env 属性可以让 clap 尝试从环境变量中读取, 

```rust
use clap::Parser;

#[derive(Parser)]
struct Cmd {
    #[arg(env="USER")]
    user: String,
}

fn main() {
    let cmd = Cmd::parse();
    println!("user: {}", cmd.user);
}
```

```bash
echo $USER
rookie
cargo run
user: rookie
# 从命令行传入的值会覆盖环境变量值
cargo run -- xd
user: xd
```

## 校验参数/转换

常见场景如传入路径时需要校验路径是否存在

```rust
use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
struct Cmd {
    #[arg(value_parser=ensure_file)]
    file: PathBuf,
}

fn ensure_file(path: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(path);
    if path.exists() {
        Ok(path)
    } else {
        Err("file not exists".into())
    }
}

fn main() {
    let cmd = Cmd::parse();
    dbg!(cmd.file);
}
```


```bash
cargo run -- Cargo.toml
[src/main.rs:22] cmd.file = "Cargo.toml"
# 文件不存在时报错
cargo run -- Cargo.tomx
error: invalid value 'Cargo.tomx' for '<FILE>': file not exists
```

当然也可以一步到位将直接读取路径后解序列化成某个类型, 这在读取配置文件时非常有用

```bash
cargo add serde -F derive
cargo add toml
```

```rust
use std::fs::read_to_string;

use clap::Parser;
use serde::Deserialize;

#[derive(Parser)]
struct Cmd {
    #[arg(value_parser=load_conf)]
    conf: Config,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub name: String,
}

fn load_conf(path: &str) -> Result<Config, String> {
    let s = read_to_string(path).map_err(|e| e.to_string())?;
    toml::from_str(&s).map_err(|e| e.to_string())
}

fn main() {
    let cmd = Cmd::parse();
    dbg!(cmd.conf);
}
```

```bash
echo 'name = "rookie"' > demo.toml
cargo run -- demo.toml
[src/main.rs:24] cmd.conf = Config {
    name: "rookie",
}
cargo run -- demo.tomx
error: invalid value 'demo.tomx' for '<CONF>': No such file or directory (os error 2)
```

## 自动展开

创建有多个子命令的程序时, 不同子命令往往共享某些参数, 可以将这些参数放入同一个结构体, 方便管理, 同时借助 `#[command(flatten)]` 属性, 可以在解析时自动展开对应的命令行参数.

```rust
use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
enum Cmd {
    Fetch(FetchCmd),
    Search(SearchCmd),
}

#[derive(Debug, Parser)]
struct FetchCmd {
    name: String,
    #[command(flatten)]
    fmt: FmtOptions,
}
#[derive(Debug, Parser)]
struct SearchCmd {
    pattern: String,
    limit: u64,
    offset: u64,
    #[command(flatten)]
    fmt: FmtOptions,
}

#[derive(Debug, Parser)]
struct FmtOptions {
    #[arg(short, long)]
    format: String,
    #[arg(short, long)]
    wide: u64,
    #[arg(short, long)]
    output: PathBuf,
}

fn main() {
    let cmd = Cmd::parse();
    dbg!(cmd);
}
```

```bash
cargo run -- fetch -h
Usage: demo fetch --format <FORMAT> --wide <WIDE> --output <OUTPUT> <NAME>

Arguments:
  <NAME>  

Options:
  -f, --format <FORMAT>  
  -w, --wide <WIDE>      
  -o, --output <OUTPUT>  
  -h, --help

cargo run -- search -h
Usage: demo search --format <FORMAT> --wide <WIDE> --output <OUTPUT> <PATTERN> <LIMIT> <OFFSET>

Arguments:
  <PATTERN>  
  <LIMIT>    
  <OFFSET>   

Options:
  -f, --format <FORMAT>  
  -w, --wide <WIDE>      
  -o, --output <OUTPUT>  
  -h, --help             Print help
```

## 解析 enum 值

enum 在表示有限选项时非常有用, clap 提供了 [ValueEnum](https://docs.rs/clap/latest/clap/trait.ValueEnum.html) 宏, 能简化 enum 解析

```rust
use clap::{Parser, ValueEnum};

#[derive(Debug, Parser)]
struct Cmd {
    color: Color,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Color {
    Red,
    Green,
    Blue,
}

fn main() {
    let cmd = Cmd::parse();
    dbg!(cmd);
}
```

```bash
cargo run -- -h
Usage: demo <COLOR>

Arguments:
  <COLOR>  [possible values: red, green, blue]

Options:
  -h, --help  Print help
```

---

我的[xLog链接](https://privaterookie-5756.xlog.app/2023-05-11-HowTo-clapmd)
