# \[HowTo\] logging

输出日志是非常常见的需求, 随着近些年 rust 社区发展, 日志生态也逐步成熟, 我之前写过的 [log4rs 文章](https://zhuanlan.zhihu.com/p/104921298)有些过时, 因此重新整理常见日志需求和基于tracing生态的解决方案.


- [\[HowTo\] logging](#howto-logging)
  - [使用 tracing-subscriber](#使用-tracing-subscriber)
  - [控制日志格式](#控制日志格式)
  - [定制时间戳格式](#定制时间戳格式)
  - [将日志输出到 stderr 和 stdout](#将日志输出到-stderr-和-stdout)
  - [将日志直接写入文件](#将日志直接写入文件)
  - [添加文件 buffer](#添加文件-buffer)
  - [文件按时间自动切分](#文件按时间自动切分)
  - [更多日志格式](#更多日志格式)
  - [记录用户定义的类型](#记录用户定义的类型)
  - [与 vector 集成](#与-vector-集成)



## 使用 tracing-subscriber

使用 cargo 添加依赖即可

```bash
cargo add tracing
cargo add tracing-subscriber
```

最简例子

```rust
fn main() {
    tracing_subscriber::fmt().init();
    tracing::info!("hello world");
}
```

控制台输出如下

```bash
2023-12-07T08:27:18.796139Z  INFO demo: hello world
```

## 控制日志格式

tracing-subscriber 通过[SubscribeBuilder](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/fmt/struct.SubscriberBuilder.html#)控制日志格式, 以下是开启常见日志格式组件的例子

```rust
fn main() {
    tracing_subscriber::fmt()
        .with_ansi(true)        // C1
        .with_target(true)      // C2
        .with_file(true)        // C3
        .with_line_number(true) // C4
        .with_level(true)       // C5
        .with_thread_ids(true)  // C6
        .with_thread_names(true)// C7
        // .without_time()      // C8
        .init();
    tracing::info!("hello world");
}
```

输出如下
```bash
2023-12-07T08:34:40.723120Z  INFO main ThreadId(01) demo: examples/demo.rs:11: hello world
```

下面我们按每行日志输出顺序介绍各个组件

`2023-12-07T08:34:40.723120Z` 为时间戳, 默认 UTC 时区, 精确到微秒, 如果要关闭时间输出可以打开 C8 所在行代码, 如果要设置时区和格式化可以参考后面章节.

`INFO` 为日志等级, 可以通过 C5 代码控制.

`main` 为线程名称名称, 对应 C7. 主线程默认名称为 main, 如果创建线程时没有指定名称, 则不输出, 此外线程名不会继承, 可以参考下面3个case

```rust
// 指定名称
std::thread::Builder::new()
    .name("t1".into())
    .spawn(|| tracing::info!("named"))
    .unwrap()
    .join()
    .unwrap();
// 不指定
std::thread::Builder::new()
    .spawn(|| tracing::info!("no name"))
    .unwrap()
    .join()
    .unwrap();
std::thread::Builder::new()
    .name("nested".into())
    .spawn(|| {
        // 在具名线程里再派生的线程不会自动继承名称
        std::thread::Builder::new()
            .spawn(|| tracing::info!("nested"))
            .unwrap()
            .join()
            .unwrap();
    })
    .unwrap()
    .join()
    .unwrap();
```

输出如下

```bash
2023-12-07T11:35:24.626899Z  INFO t1 ThreadId(02) demo: examples/demo.rs:14: named
2023-12-07T11:35:24.627002Z  INFO ThreadId(03) demo: examples/demo.rs:19: no name
2023-12-07T11:35:24.627144Z  INFO ThreadId(05) demo: examples/demo.rs:27: nested
```

`ThreadId(01)` 对应 C6, 顾名思义就是线程 id.

`demo:` 为 target 名称, 对应 C2, 如果在入口处, 则为程序名称, 假设调用如下模块内定义的函数, target 组件输出为 `demo::a::b::c:`

```rust
mod a {
    pub mod b {
        pub mod c {
            pub fn hello() {
                tracing::info!("hello world");
            }
        }
    }
}
a::b::c::hello();
```

`examples/demo.rs:` 即为文件名, 对应 C3, `11:` 为行数, 对应 C4, 这两个通常一起开启, 方便定位. 值得注意的是如果是在 vscode 或其他一些编辑器里, 在终端上输出 `examples/demo.rs:11:` 按住 ctrl 同时点击文字可以直接跳转到对应文件和对应行数, 在调试时非常方便.

`hello world` 为 log message, 也是我们要记录信息.

## 定制时间戳格式

在上面的例子里, 日志的时间戳格式使用 UTC 而不是东八区, 查看日期还必须加减8小时, 非常不便. tracing-subscriber 允许定制时间戳格式.
tracing-subscriber 支持 time 和 chrono 两个时间库, 但我推荐 time, 因为 time 不仅 api 更好用且 chrono 在某些版本存在性能问题.

首先需要添加 feature 和依赖

```bash
cargo add tracing-subscriber -F time
cargo add time -F parsing -F macros -F parsing
```

如果要将日志时间戳精度固定到毫秒, 且使用东八区, 可以参考如下代码

```rust
fn main() {
    let format = time::format_description::parse(
        "[year]-[month padding:zero]-[day padding:zero] [hour]:[minute]:[second].[subsecond digits:3]",
    )
    .unwrap();
    let offset = time::macros::offset!(+8);
    tracing_subscriber::fmt()
        .with_timer(tracing_subscriber::fmt::time::OffsetTime::new(
            offset, format,
        ))
        .init();
    tracing::info!("hello world");
}
```
time 使用自己的DSL控制格式, 详细描述可以参考[Book](https://time-rs.github.io/book/api/format-description.html)
可以看到时间戳精度已经限制到毫秒, 且使用了东八区

```bash
2023-12-07 20:24:04.560  INFO demo: hello world
```

如果希望时使用标准格式, time 也提供了三种, 这里推荐使用 rfc3339 格式

```rust
fn main() {
    let offset = time::macros::offset!(+8);
    tracing_subscriber::fmt()
        .with_timer(tracing_subscriber::fmt::time::OffsetTime::new(
            offset,
            time::format_description::well_known::Rfc3339,
        ))
        .init();
    tracing::info!("hello world");
}
```


## 将日志输出到 stderr 和 stdout

tracing-subscriber 默认输出到程序 stdout, 一个需求是希望将 ERROR 及以上级别的日志输出到 stderr, 其他级别日志则输出到 stdout.

因为需要使用 metadata 过滤, 所以要添加依赖. tracing-subscriber 支持通过 [make_writer](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/fmt/struct.SubscriberBuilder.html#method.with_writer) 传入自定义的 Writer, 同时也可以使用 `and`, `or_else` 等组合函数组合不同的 Writer, 可以将一份日志进行过滤等操作.

```bash
cargo add tracing-core
```

我们可以通过 [with_filter](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/fmt/writer/trait.MakeWriterExt.html#method.with_filter) 添加过滤逻辑. 需要注意的是, tracing-core 里 TRACE 级别日志是最高, ERROR 最低.


```rust
use tracing_core::Level;
use tracing_subscriber::fmt::writer::MakeWriterExt;

fn main() {
    tracing_subscriber::fmt()
        .with_writer(
            std::io::stdout
                .with_filter(|meta| meta.level() > &Level::ERROR)
                .or_else(std::io::stderr),
        )
        .init();
    tracing::info!("all is well");
    tracing::error!("oh no");
}
```

可以通过只重定向 stdout 检查我们的重定向逻辑是否正常

```bash
cargo run -q --example demo > /tmp/out
2023-12-08T07:15:20.785404Z ERROR demo: oh no
```

可以看到 `oh no` 确实被输出到 stderr 没有被重定向到 /tmp/out


## 将日志直接写入文件

`with_writer` 支持传入 `Mutex<File>`, 但要注意, tracing-subscriber 在向终端写入日志时, 默认会将日志级别自动加上 ansi-term 颜色转义字符, 这样在终端上可以看到 INFO 显示为绿色, ERROR 为红色等, 但输出到文件时我们不需要这些转义字符, 所以需要通过 `with_ansi(false)` 关掉.

```rust
use std::{fs::File, sync::Mutex};

fn main() {
    // 这里开启追加写和自动创建创建选项, 避免手动创建文件
    // 和覆盖之前的文件
    let log_file = File::options()
        .append(true)
        .create(true)
        .write(true)
        .open("logs/app.log")
        .unwrap();
    tracing_subscriber::fmt()
        .with_ansi(false)
        .with_writer(Mutex::new(log_file))
        .init();
    tracing::info!("all is well");
    tracing::error!("oh no");
}
```

## 添加文件 buffer

在上面的例子里直接向一个没有任何 buffer 的文件写入日志性能不会很好, 如果程序需要频繁写日志, 日志反而可能成为性能热点. 
最简单的办法是使用 BufWriter 添加一层 buffer, 但 tracing-subscriber  没有为 BufWriter 实现 MakeWriter trait; 而且我在自己实现时发现如果将整个 BufWriter 通过 with_writer 方法传进去后即使程序退出也没有调用 BufWriter drop 方法, 导致还在 buffer 中的日志没有被刷到磁盘上, 导致日志总是缺了最后一部分.

如果你不想自己解决这些问题, 可以直接使用我已经开发好的

```bash
cargo add tracing-rolling
```

```rust
use tracing_rolling::{Checker, ConstFile};

fn main() {
    let (writer, token) = ConstFile::new("logs/c.log")
        // 添加 8k buffer
        .buffer_with(8192)
        .build()
        .unwrap();
    tracing_subscriber::fmt()
        .with_ansi(false)
        .with_writer(writer)
        .init();
    for _ in 0..100 {
        tracing::info!("all is well");
    }
    // 在程序最后手动 drop token, 确保退出时自动刷新
    drop(token);
}
```

## 文件按时间自动切分

生产环境里, 文件大概率会按天自动切分, tracing-subscriber 没有提供这个功能, 虽然也有其他库实现了, 但依然不能满足我当时的要求, 所以我在上面介绍的 tracing-rolling 库里实现了, 按天,小时,分钟自动切分日志的功能, 并且可以值得切分时使用什么时区判断是否该切分.

下面是一个按天分割的日志配置例子

```rust
use std::time::Duration;

use time::macros::offset;
use tokio::time::sleep;
use tracing::info;
use tracing_rolling::{Checker, Daily};

#[tokio::main]
async fn main() {
    // create a daily rolling file with custom date format
    // output file pattern is testing.20230323.log
    let (writer, token) = Daily::new("logs/testing.log", "[year][month][day]", offset!(+8))
        // Daily::new("logs/testing.log", None, offset!(+8))
        .buffered() // buffer file if needed
        .build()
        .unwrap();

    tracing_subscriber::fmt()
        .with_ansi(false)
        .with_target(false)
        .with_file(true)
        .with_line_number(true)
        .with_writer(writer)
        .init();
    let mut count = 0;
    info!("start");
    while count < 100 {
        count += 1;
        sleep(Duration::from_millis(50)).await;
        info!("{count}");
    }
    drop(token);
}
```

## 更多日志格式

常见的普通文本格式便于阅读和处理, 但我们有时候需要输出其他格式日志, 方便与其他服务集成. tracing-subscriber 支持 json 格式.

```bash
cargo add tracing-subscriber -F json
```

```rust
use tracing::info;
fn main() {
    tracing_subscriber::fmt().json().init();
    info!("hello");
}
```

```js
{"timestamp":"2023-12-09T05:21:06.335149Z","level":"INFO","fields":{"message":"hello"},"target":"demo"}
```

可以看到输出的 json 里日志内容处于 `fields.message` 字段, 那如果我们想让 message 也变成 json 对象而不是字符串以记录更多信息呢? 

`info` 等宏支持更多格式, 通过以下方式可以输出更加格式化的内容

```rust
use tracing::info;
fn main() {
    tracing_subscriber::fmt().json().init();
    info!(name = "PrivateRookie", age = 15);
    // 也可以改变 默认 target
    info!(target: "login", name = "PrivateRookie", age = 15);
    let age = 100;
    // short hand 用法也是可以的
    info!(age);
}
```

```js
{"timestamp":"2023-12-09T05:48:09.969417Z","level":"INFO","fields":{"name":"PrivateRookie","age":15},"target":"demo"}
{"timestamp":"2023-12-09T05:48:09.969458Z","level":"INFO","fields":{"name":"PrivateRookie","age":15},"target":"login"}
{"timestamp":"2023-12-09T05:55:17.316813Z","level":"INFO","fields":{"age":100},"target":"demo"}
```

## 记录用户定义的类型

上面提到使用 `key=value` 格式可以输出更好格式化的内容, 但 tracing 只支持基础类型, 如果遇到用户定义的类型, 会报错. 其实 tracing 也支持复杂类型, 但需要开启 unstable 编译条件和使用 [valuable](https://docs.rs/valuable/latest/valuable/?search=).

首先需要开启 tracing-unstable 编译条件, 在项目根目录下创建 `.cargo/config.toml` 文件, 添加如下内容

```toml
[build]
rustflags = ["--cfg", "tracing_unstable"]
```

接着添加或开启依赖 feature

```bash
cargo add tracing-subscriber -F json -F valuable
cargo add valuable -F derive
```

接着我们就可以在打日志时通过

```rust
use tracing::info;
fn main() {
    tracing_subscriber::fmt().json().init();

    #[derive(Debug, Clone, Valuable)]
    pub struct User {
        pub name: String,
        pub age: usize,
    }
    let u = User {
        name: "PrivateRookie".into(),
        age: 15,
    };
    info!(user = u.as_value());
}
```

```bash
# json 格式
{"timestamp":"2023-12-09T06:18:07.987381Z","level":"INFO","fields":{"user":{"name":"PrivateRookie","age":15}},"target":"demo"}
# 普通文本格式
2023-12-09T06:18:29.662892Z  INFO demo: user=User { name: "PrivateRookie", age: 15 
```

**值得注意的是 tracing-subscriber 不开启 `valuable` feature 程序也不会报错, 但无法正确字符的引号.**

## 与 vector 集成

[vector](https://vector.dev/) 是一个轻量,高性能且功能丰富的日志处理库, 它可以处理多种数据源. 如果要将日志传输给 vector 处理, 最常见的方式是使用 [File source](https://vector.dev/docs/reference/configuration/sources/file/) 直接读取日志, 但这种方式还需要手动解析日志. 这里介绍一种偷懒取巧的办法. 使用 tcp socket 作为 writer, json 作为日志格式, vector 则使用 [socket source](https://vector.dev/docs/reference/configuration/sources/socket/) 作为源, 同时将编码类型设置为按行分割的 json, 这样就可以在 vector 直接已经格式化好的日志.

下面是一个将日志发送到 vector, 然后使用 vector 将其转换成 csv 并输出到控制台的例子, 仅用作说明, 不考虑tcp断开等情况.


rust 程序如下
```rust
use std::{net::TcpStream, sync::Arc};

use tracing::info;
fn main() {
    let stream = TcpStream::connect("127.0.0.1:9000").unwrap();
    tracing_subscriber::fmt()
        .with_writer(Arc::new(stream))
        .json()
        .init();
    for age in 0..15 {
        info!(name = "PrivateRookie", age);
    }
}
```

vector 配置, 要注意日志内容都被放入 fields 字段, 其他都是 tracing 加上的 metadata.

```yaml
# vector.yaml
sources:
  in:
    type: "socket"
    address: "0.0.0.0:9000"
    mode: "tcp"
    decoding:
      codec: "json"

sinks:
  out:
    inputs:
      - "in"
    type: "console"
    encoding:
      codec: "csv"
      csv:
        fields: [".fields.name", ".fields.age"]
```

先启动 vector, `vector --config vector.toml`, 然后启动程序, 可以看到 vector 启动终端看到输出的csv


![Alt text](/static/assets/2023_12_09/image.png)

