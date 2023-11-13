# 【使用 Rust 写 Parser】2. 解析Redis协议

---

在基本熟悉 nom 之后, 这次我们准备用 nom 实现一个 redis 通信协议的解析器. 选择 redis 是
因为 redis 的通信协议易读且比较简单.

## 准备

如果你对 redis 通信协议不熟悉的话可以查阅 [通信协议（protocol）](http://redisdoc.com/topic/protocol.html#id1).
简单来说 redis 通信协议分为统一请求协议(这里只讨论新版请求协议)和回复协议, 请求协议可以方便地通过 Rust 内置的 `format!` 拼接构成, 而通信协议则使用 nom 解析. redis 协议非常简单,
这里不再赘述.

首先我们需要一个 redis 服务器, 这里我在开发的机器上用 docker 启动一个 redis 服务器:

```bash
docker run -d --name redis -p 6379:6379 redis redis-server --appendonly yes
```

测试下 redis 服务

```bash
telnet localhost 6379
Trying 127.0.0.1...
Connected to localhost.
Escape character is '^]'.
ping
+PONG
```

出现 +PONG 说明服务器已正常运行

## 实现基本功能

首先创建项目

```bash
cargo new rcli && cd rcli
```

添加如下依赖

```toml
[dependencies]
tokio = { version = "0.2", features = ["full"]}
nom = "5"
bytes = "0.5.4"
structopt = "0.3.14"
```

structopt 可以帮助我们快速构建命令行工具输入 redis 命令帮助测试, bytes 则可以帮助我们处理字节, tokio 依赖是上个测试代码遗留的依赖, 刚好新代码也需要 tcp 连接, 索性使用 tokio 处理 tcp 连接, nom 自然是用于解析回复.

首先我们需要创建 tcp 连接与 redis 通信, 并且写入一些数据看看协议是否管用:

```rust
use bytes::{BufMut, BytesMut};
use std::error::Error;
use tokio::net::TcpStream;
use tokio::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    let mut stream = TcpStream::connect("127.0.0.1:6379").await?;
    let mut buf = [0u8; 1024];
    let mut resp = BytesMut::with_capacity(1024);

    let (mut reader, mut writer) = stream.split();
    // 向服务器发送 PING
    writer.write(b"*1\r\n$4\r\nPING\r\n").await?;
    let n = reader.read(&mut buf).await?;
    resp.put(&buf[0..n]);
    // 返回结果应该是 PONG
    println!("{:?}", reply);
    Ok(())
}
```

如上面代码展示的, 我们创建一个 tcp 连接和一个缓冲buf, 在成功连接后根据协议尝试写入 `*1\r\n$4\r\nPING\r\n`, 预期结果是服务器返回 `"+PONG\r\n"`.

现在我们可以创建 CLI 实现几个常用的 redis 命令, 方便我们向服务器发送命令. 创建 `commands.rs` 文件, 记得在 `main.rs` 中导入它.

以 `rpush` 为例, `rpush` 命令用法为 `RPUSH key value [value …]`

使用 structopt 可以这样定义一个枚举(使用结构体也可以, 但因为将来有很多子命令, 所以枚举更合适)

```rust
use structopt::StructOpt;

#[derive(Debug, Clone, StructOpt)]
pub enum Commands {
    /// push value to list
    Rpush {
        /// redis key
        key: String,

        /// value
        values: Vec<String>,
    },
}
```

接着在 `main.rs` 中使用 `Commands` 解析命令行

```rust
use structopt::StructOpt;
mod commands;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // 创建 tcp 连接, buf 等...
    let com = commands::Commands::from_args();
    // 发送命令 ...
}
```

运行项目看下效果

```bash
cargo run -- help

push value to list

USAGE:
    rrdis-cli rpush <key> [values]...

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

ARGS:
    <key>          redis key
    <values>...    value
```

接下来要把从命令行传来的参数转换为 redis 统一请求. redis 以 `\r\n` 为分隔符, redis 请求格式以 `*argc` 开头,
`argc` 是此次请求的参数个数, 每个参数先以 `$<参数长度>` 声明参数长度, 接着 `\r\n` 分割符, 然后是参数数据, 若有多个参数则重复此步骤. 最后以 `\r\n` 结尾.

比如上面的 `PING` 转换为 `*1\r\n$4\r\nPING\r\n`, 而 `GET` 转换为 `*2\r\n$3\r\nGET\r\n$3\r\nkey\r\n`.

可以使用一个 builder 帮助我们转换:

```rust
use bytes::{BufMut, BytesMut};

#[derive(Debug, Clone)]
struct CmdBuilder {
    args: Vec<String>,
}

impl CmdBuilder {
    fn new() -> Self {
        CmdBuilder { args: vec![] }
    }
    fn arg(mut self, arg: &str) -> Self {
        self.args.push(format!("${}", arg.len()));
        self.args.push(arg.to_string());
        self
    }
    fn add_arg(&mut self, arg: &str) {
        self.args.push(format!("${}", arg.len()));
        self.args.push(arg.to_string());
    }
    fn to_bytes(&self) -> BytesMut {
        let mut bytes = BytesMut::new();
        bytes.put(&format!("*{}\r\n", self.args.len() / 2).into_bytes()[..]);
        bytes.put(&self.args.join("\r\n").into_bytes()[..]);
        bytes.put(&b"\r\n"[..]);
        bytes
    }
}
```

`CmdBuilder` 做的很简单, 保存通过 `arg` 或 `add_arg` 传入的参数, 在 `to_bytes` 方法中拼接
这些参数为有效的请求.

例如可以通过如下方式构建一个 `GET` 命令

```rust
let cmd = CmdBuilder::new().arg("GET").arg("key").to_bytes()
```

接下来使用 `CmdBuilder` 为 `Commands` 实现 `to_bytes` 方法

```rust
impl Commands {
    pub fn to_bytes(&self) -> bytes::BytesMut {
        let cmd = match self {
            Commands::Rpush { key, values } => {
                let mut builder = CmdBuilder::new().arg("RPUSH").arg(key);
                values.iter().for_each(|v| builder.add_arg(v));
                builder.to_bytes()
            }
        };
        cmd
    }
}
```

改写 `main` 函数发送构建的请求

```rust
// ... 省略
let com = commands::Commands::from_args();
writer.write(&com.to_bytes()).await?;
```

```bash
cargo run -- rpush list a b c d

# redis 成功返回响应
:3\r\n
```

All is well, 对于其他命令可以通过相同方法实现, 可以在 [rrdis-cli/src/commands.rs](https://github.com/PrivateRookie/rrdis-cli/blob/master/src/commands.rs) 看到完整实现.

## 解析回复

现在终于到 nom 出场了. 新建 `reply.rs` 文件, 并在 `main.rs` 导入. 首先导入需要使用的 nom 方法, 接着定义 `Reply`, 因为 redis 回复种类有限, 所以用一个枚举是非常合适的.

```rust
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::bytes::complete::{take_while, take_while1, take_while_m_n};
use nom::combinator::map;
use nom::multi::many_m_n;
use nom::sequence::delimited;
use nom::IResult;

#[derive(Debug)]
pub enum Reply {
    // 状态回复或单行回复
    SingleLine(String),
    // 错误回复
    Err(String),
    // 整数回复
    Int(i64),
    // 批量回复
    Batch(Option<String>),
    // 多条批量回复
    MultiBatch(Option<Vec<Reply>>),
    // 回复中没有, 这里是为了方便进行错误处理添加的
    BadReply(String),
}
```

### 单行回复

协议中单行回复定义如下:
> 一个状态回复（或者单行回复，single line reply）是一段以 "+" 开始、 "\r\n" 结尾的单行字符串。

所以解析思路是: 如果回复以"+"开头, 则读取余下字节存作为回复, 直到 "\r\n", 伪代码如下

```rust
take_if("+"), take_util_new_line, take_if("\r\n")
```

nom 中的 `tag` 可以完美实现伪代码中的 `take_if` 功能, 令人惊喜的是对于"消耗输入直到不符合某种条件"这个常见解析模式, nom 提供了 `take_while` 函数, 所以我们的解析函数可以写成:

```rust
fn parse_single_line(i: &str) -> IResult<&str, Reply> {
    let (i, _) = tag("+")(i)?;
    let (i, resp) = take_while(|c| c != '\r' && c != '\n')(i)?;
    let (i, _) = tag("\r\n")(i)?;
    Ok((i, Reply::SingleLine(resp.to_string())))
}
```

`tag` 和 `take_while` 让解析函数的功能非常直观地展现出来, 这让它看着想伪代码, 但它真的能运行!

在函数中只有 `take_while` 返回的结果是我们想要的, 但两个 `tag` 又是不可或缺, 对于这一常见解析模式 nom 提供了 `delimited` 这个组合子函数, 这个组合子函数接受三个类似 `tag("xx")` 这样的基本函数, 依次应用这三个函数, 如果成功, 则返回第二个函数解析的结果.

所以我们的函数可以这样写:

```rust
fn parse_single_line(i: &str) -> IResult<&str, Reply> {
    let (i, resp) = delimited(
        tag("+"),
        take_while(|c| c != '\r' && c != '\n'),
        tag("\r\n"),
    )(i)?;
    Ok((i, Reply::SingleLine(String::from(resp))))
}
```

### 错误回复

错误回复定义:

> 错误回复和状态回复非常相似， 它们之间的唯一区别是， 错误回复的第一个字节是 "-" ， 而状态回复的第一个字节是 "+"

所以错误回复解析函数和上面的差不多:

```rust
fn parse_err(i: &str) -> IResult<&str, Reply> {
    let (i, resp) = delimited(
        tag("-"),
        // take_while1 与 take_while 类似, 但要求至少一个字符符合条件
        take_while1(|c| c != '\r' && c != '\n'),
        tag("\r\n"),
    )(i)?;
    Ok((i, Reply::Err(String::from(resp))))
}
```

### 整数回复

> 整数回复就是一个以 ":" 开头， CRLF 结尾的字符串表示的整数,

整数回复结构与前两种类似, 区别在于中间是整数, 需要将 `take_while1` 的返回值转换为整数.

如果没有进行类型转换解析函数可以这样实现:

```rust
fn parse_int(i: &str) -> IResult<&str, Reply> {
    let (i, int) = delimited(
        tag(":"),
        // 注意负数前缀
        take_while1(|c: char| c.is_digit(10) || c == '-'),
        tag("\r\n"),
    )(i)?;
    // ... 类型转换
    Ok((i, Reply::Int(int)))
}
```

注意到 nom 提供的基本解析工厂函数如 `tag` 创建的解析函数返回值都是 `IResult`, 它与 `Result` 类似, 可以应用 `map` 运算子, 不过这个 `map` 需使用 nom 提供的

```rust
map(take_while1(|c: char| c.is_digit(10) || c == '-'), |int: &str| int.parse::<i64>().unwrap())
```

通过 nom 的 map 函数可以把返回值从 `IResult<&str, &str>` 映射为 `IResult<&str, i64>`,
最后解析函数可以写成

```rust
fn parse_int(i: &str) -> IResult<&str, Reply> {
    let (i, int) = delimited(
        tag(":"),
        map(
            take_while1(|c: char| c.is_digit(10) || c == '-'),
            |int: &str| int.parse::<i64>().unwrap(),
        ),
        tag("\r\n"),
    )(i)?;
    Ok((i, Reply::Int(int)))
}
```

### 批量回复

> 服务器发送的内容中：
> - 第一字节为 "$" 符号
> - 接下来跟着的是表示实际回复长度的数字值
> - 之后跟着一个 CRLF
> - 再后面跟着的是实际回复数据
> - 最末尾是另一个 CRLF

同时批量回复还有特殊情况

> 如果被请求的值不存在， 那么批量回复会将特殊值 -1 用作回复的长度值, 这种回复称为空批量回复（NULL Bulk Reply）

此时协议要求客户端返回空对象, 对于 Rust 则是 `None`, 所以 `BatchReply` 才会被定义为 `BatchReply<Option<String>>`.

所以这个函数的解析可能稍微复杂点, 但方法与上面没有太大差异, 除了新的 `take_while_m_n`,
`take_while_m_n` 与 `take_while` 类似, 不同的是它可以指定消耗输入最小数和最大数m, n.

如果是空回复则尝试匹配 `\r\n`, 如果成功, 直接返回, 否则根据拿到的回复长度, 获取那么多长度的字符, 接着应该碰到 `\r\n`.

```rust
fn parse_batch(i: &str) -> IResult<&str, Reply> {
    let (i, _) = tag("$")(i)?;
    let (i, len) = (take_while1(|c: char| c.is_digit(10) || c == '-'))(i)?;
    if len == "-1" {
        let (i, _) = tag("\r\n")(i)?;
        Ok((i, Reply::Batch(None)))
    } else {
        let len = len.parse::<usize>().unwrap();
        let (i, resp) = delimited(tag("\r\n"), take_while_m_n(len, len, |_| true), tag("\r\n"))(i)?;
        Ok((i, Reply::Batch(Some(String::from(resp)))))
    }
}
```

### 多条批量回复

> 多条批量回复是由多个回复组成的数组， 数组中的每个元素都可以是任意类型的回复， 包括多条批量回复本身。
> 多条批量回复的第一个字节为 "*" ， 后跟一个字符串表示的整数值， 这个值记录了多条批量回复所包含的回复数量， 再后面是一个 CRLF

多条批量回复其实是对上面四种回复的嵌套, 但需要注意"空白多条批量回复"和"无内容多条批量回复"这两种特殊情况.

空白多条回复为 "\*0\r\n", 无内容多条批量回复为 "\*-1\r\n", 在解析时需要对这两种特殊情况进行处理. 在其他情况则可以应用 nom 提供的 `alt` 组合子服用之前的四个解析函数; `alt` 即"可选的", 它接受多个解析函数元组, 依次尝试应用每个函数, 返回第一个成功解析结果或抛出错误.

同时对于重复应用某个解析函数 m 到 n 次这种模式, nom 提供了 `many_m_n` 组合子, 对于 `fn parse_item(&str) -> IResult<&str, Reply>` 这样的函数, `many_m_n(parse_item, 0, 12)` 返回值为 `IResult<&str, Vec<Reply>>`.

理清逻辑后解析多条批量回复的解析函数虽然有些长但还是很清晰的:

```rust
fn parse_multi_batch(i: &str) -> IResult<&str, Reply> {
    let (i, count) = delimited(
        tag("*"),
        take_while1(|c: char| c.is_digit(10) || c == '-'),
        tag("\r\n"),
    )(i)?;
    if count == "-1" {
        let (i, _) = tag("\r\n")(i)?;
        Ok((i, Reply::MultiBatch(None)))
    } else {
        let count = count.parse::<usize>().unwrap();
        let (i, responses) = many_m_n(
            count,
            count,
            alt((parse_single_line, parse_err, parse_int, parse_batch)),
        )(i)?;
        // 做个严格检查, 检查解析到的个数与预期的是否一致
        if responses.len() != count {
            Ok((
                i,
                Reply::BadReply(format!("expect {} items, got {}", count, responses.len())),
            ))
        } else {
            Ok((i, Reply::MultiBatch(Some(responses))))
        }
    }
}
```

最后用 `alt` 做个"汇总"

```rust
fn parse(i: &str) -> IResult<&str, Reply> {
    alt((
        parse_single_line,
        parse_err,
        parse_int,
        parse_batch,
        parse_multi_batch,
    ))(i)
}
```

至此我们我们的解析函数到完成了, 为 `Reply` 实现 `Display` 特性后对 redis 返回的消息应用 `parse` 然后把解析结果打印出来即可验证解析函数正确性. 完整代码在 [rrdis-cli/src/reply.rs](https://github.com/PrivateRookie/rrdis-cli/blob/master/src/reply.rs)

## 汇总

完整代码可以在我的 [rrdis-cli](https://github.com/PrivateRookie/rrdis-cli) 查看.
不知道大家对 nom 的评价如何, 我觉得使用 `nom` 提供的基本函数和一系列组合子从最小元素出发, 搭积木似的构建出更复杂的解析函数, 即降低了开发难度, 熟悉之后代码逻辑还挺清晰的.

整个 rrdis-cli 项目实现 set, get, incr, lrange, rpush 和 ping 这基本命令, 实现其他命令也是非常简单; 并且实现了绝大部分(还有一些特殊错误情况没处理)协议解析, 整个项目代码量如下

```bash
tokei .
-------------------------------------------------------------------------------
 Language            Files        Lines         Code     Comments       Blanks
-------------------------------------------------------------------------------
 Markdown                1            4            4            0            0
 Rust                    3          332          284           20           28
 TOML                    1           15           12            1            2
-------------------------------------------------------------------------------
 Total                   5          351          300           21           30
-------------------------------------------------------------------------------
```

Rust 代码只有 332 行, 挺简洁的, 估计比我用 Python 实现都少.

下一篇使用 nom 写什么还不确定, 随缘更新吧~