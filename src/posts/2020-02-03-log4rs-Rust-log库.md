# log4rs Rust log crate

---

在用 Rust 写练手项目时经常要用到日志库, 我之前使用过 env_logger 和 pretty_env_logger 这两个日志库, 两个库总体上都满足我之前的需求, 但在配置 log 将 log 写入文件而不仅仅是控制台时, 遇到挺大的麻烦.
辗转之后发现了 [log4rs](https://github.com/estk/log4rs) 这个库, 虽然上手有些复杂, 但有以下特性让我非常喜欢:

1. 支持通过文件(yaml文件, 我个人非常推荐)和代码配置
2. 支持写入 log 文件
3. 自动重载配置文件更新 log 配置!
4. 支持为每个模块单独配置 log
5. 支持对 log 信息模式进行配置

虽然 log4rs 功能挺多, 可惜项目 [README](https://github.com/estk/log4rs/blob/master/README.md) 过于简单, 不少功能需要配合 docs.rs/log4rs 和 项目源码才能被发现, 现在总结下 log4rs 使用方法.

## Quick Start

首先需要添加依赖

```toml
[dependencies]
log = "0.4.8"
log4rs = "0.10.0"
```

log4rs 有很多 feature, 默认都是开启的, 这里只需要指定版本就好了.

接着需要写一个 yaml 配置文件 `log4rs.yaml` 放在项目根目录下

```yaml
---
# log4rs.yaml
# 检查配置文件变动的时间间隔
refresh_rate: 30 seconds
# appender 负责将日志收集到控制台或文件, 可配置多个
appenders:
  stdout:
    kind: console
  file:
    kind: file
    path: "log/log.log"
    encoder:
      # log 信息模式
      pattern: "{d} - {m}{n}"
# 对全局 log 进行配置
root:
  level: info
  appenders:
    - stdout
    - file
```

接着在代码中使用 `log4rs::init_file` 方法读取配置文件进行初始化, 然后使用 `log` 中的 `info!` 等宏在需要的地方输出日志.

```rust
use log::info;
use log4rs;

fn main() {
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();
    info!("INFO");
}
```

`cargo run` 运行项目会在项目下新建 `log/log.log` 文件, 且内容如下, 同时控制台也会输出
类似的内容.

```log
2020-02-03T15:38:02.659851+08:00 - INFO
```

## 概念讲解

根据 docs.rs/log4rs, log4rs 主要主要有以下概念

- appenders
- encoders
- filers
- loggers

### Appenders

appender 负责将 log 发往控制台或文件. 目前一共有三种类型可选 `console`, `file` 和 `rolling_file`.

`console` 配置相对简单

```yaml
appenders:
  console:
    kind: console
    target: stdout # 或者 stderr
    encoder: # console 同样可以配置 encoder
      kind: pattern
```

`file` 和 `rolling_file` 都是将日志收集到文件中, 但对于长时间运行的程序, `rolling_file` 更合适, 因为它可以配置 log rotate, 避免 log 文件占用太多硬盘空间.

先说二者的共同配置项

```yaml
appenders:
  log_file:
    kind: file # 或 rolling_file
    path: <log_file_path> # log 文件路径
    append: true # 追加模式, 即每次在已有文件末尾添加日志, 默认为 true
    encoder:
      kind: pattern
```

对于 `rolling_file` 需要额外配置 log rotate policy

```yaml
  log_file:
    kind: rolling_file
    # ...
    policy:
      kind: compound # 默认值, 即使用所有 policy
      trigger: # 当文件超过10mb 时触发 rotate
        kind: size
        limit: 10mb
      roller: # rotate 类型
        kind: delete # 直接原有文件
        # 或者用 fixed_window
        kind: fixed_window
        pattern: "compressed-log-{}-.log" # 注意, 需要至少包含 "{}" 用于插入索引值
        base: 0 # 压缩日志索引值起点
        count: 2 # 最大保存压缩文件数
```

fixed_window 的逻辑稍微有些复杂, 当 log 文件触发 rotate 时 log4rs 会把该文件压缩(需启用 gzip feature) 然后以 pattern 中定义文件模式并将 `{}` 插入索引值保存.
以上述配置为例, 文件产生顺序为 `compressed-log-0-.log` -> `compressed-log-1-.log`, 当文件再次触发
rotate 时 log4rs 会删除 `compressed-log-0-.log`, 然后将 `compressed-log-1-.log` 重命名为 `compressed-log-0-.log`, 最新的 log 压缩文件命名为 `compressed-log-1-.log`.

**注意**, 如果压缩文件数量达到最大时, log4rs 会逐个重命名文件, 此时如果 count 的很大, 有可能会造成性能问题.

### Encoders

encoder 负责将 log 信息转换为合适的格式, 如固定格式的平文本或 json, 每个 appender 都可以指定一个
encoder.

目前有2个 encoder `pattern` 和 `json`

`json` encoder 负责将日志转为 json 格式, 配置项只有一个

```yaml
encoder:
  kind: json
```

输出例子如下

```json
{
    "time": "2016-03-20T14:22:20.644420340-08:00",
    "message": "the log message",
    "module_path": "foo::bar",
    "file": "foo/bar/mod.rs",
    "line": 100,
    "level": "INFO",
    "target": "foo::bar",
    "thread": "main",
    "thread_id": 123,
    "mdc": {
        "request_id": "123e4567-e89b-12d3-a456-426655440000"
    }
}
```

**注意** 实际写入时并不会增加缩进, 而且 log4rs 有个 bug, 其输出的 json 文件最外层没有一个包含所有
log 信息的列表, 导致该文件不是有效的 json 文件.

mdc 是 mapped diagnostic context 即映射诊断环境, 用于向 log 中添加额外信息帮助 debug. 详见 [MDC](https://crates.io/crates/log-mdc).

`pattern` encoder 则是将 log 转换为特定格式的纯文本

```yaml
encoder:
  kind: pattern
  pattern: <log_message_pattern>
```

log 信息的 pattern 可以在 [log4rs::encoder::pattern](https://docs.rs/log4rs/0.10.0/log4rs/encode/pattern/index.html) 找到详细定义, 这里简要说明一下.

pattern 类似于正则表达式, `{}` 加一个特定字符代表要插入的某个信息, 如 `{m}` 代表 log 信息, 常见字符如下

- d, data 日期, 默认为 ISO 9601 格式, 可以通过 `{d(%Y-%m-%d %H:%M:%S)}` 这种方式改变日期格式
- l log 级别
- L, line log消息所在行数
- m, message log 消息
- M, Module log 消息所在模块
- X, mdc 映射诊断环境

如果要在 log 消息中使用 "{}()\" 字符, 需要进行转义, log4rs 可以通过重复字符转义, 如 '\{\{' -> '{', 或者通过添加 '\' 进行转义, 如 '\{' -> '{'.

### Filters

filter 顾名思义即为过滤掉某些 log 信息, 文档好像有问题, 我读的也不是很清楚, 略...

### Loggers

在之前的信息配置中都没有出现 `loggers` 的配置, 因为我们配置 `root` 即全局 logger, 在 log4rs 中除了全局 logger, 还可以创建额外的 logger, logger 以 Rust 的模块为结构, 实现了配置继承.

举个例子, 假设程序结构如下

```bash
src
├── ask.rs
└── main.rs

0 directories, 2 files
```

log 配置文件如下

```yaml
refresh_rate: 30 seconds
appenders:
  file:
    kind: file
    path: "log/log.log"
    encoder:
      kind: pattern
      pattern: "{d} {l} {M} - {m}{n}"
root:
  level: info
  appenders:
    - file
```

在 `ask.rs` 内容如下

```rust
use log::{debug, info};

pub fn ask() {
    debug!("inside ask module");
    info!("inside ask module");
}
```

假设我们在 `main.rs` 中使用了 `ask::ask` 你会发现没有 `debug!` 打印的信息, 这是因为 `ask` 模块的 logger 默认使用 `root` logger 的 level, 即 info.

现在新增一个 appender 和 logger, 并把这个 logger level 设置为 debug.

```yaml
appenders:
  ask:
    kind: file
    path: "log/ask.log"
    encoder:
      pattern: "{d} {l} {M} - {m}{n}"
loggers:
  play::ask: # logger 名称必须与模块名相同, 如 app::request::db, 第一个应该为 crate 名称
    level: debug
    appenders:
      - ask
    additivity: false
```

重新运行程序, 会发现在 `log/ask.log` 中出现了 debug 级别的日志. 这里额外设置了 `additivity` 为 false, 因为 log4rs 默认会把子 logger 的信息也收集起来, 因此会在 `log/log.log` 和 `log/ask.log` 中都含有 `ask` 模块的 log 信息, 为了减少冗余, 可以将 `additivity` 设置为 false, 这样子模块的 log 就不会输出到父模块上.

## 总结

跟人感觉 log4rs 的功能挺丰富的, 除了 yaml 格式的, 它还支持 json, toml, xml 和在代码中配置, 基本上都满足了我的需求, 推荐在项目中试一试. **但我还没有对它进行性能测试**.