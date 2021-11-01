# 【使用 Rust 写 Parser】1. 初识 nom

关于 nom 计划写一个系列, 大概有3篇或更多专栏文章(如果我不是太忙或不鸽的话), 从基本概念开始, 到中等难度的 json 解析器, 最后可能会用 nom 5.0 实现简单语言的解析器.

---

## 简介

最近在读书练习用 Rust 写算术表达式解析器被正则表达式弄烦了, 不由得想起那句金句

> Some people, when confronted with a problem, think “I know, I’ll use regular expressions.” Now they have two problems.
> By Jamie Zawinski

虽然最后用正则表达式实现原有需求, 甚至想写篇专栏记录下, 但一个星期后再看代码, 好像比当初写的时候还要晦涩, 算了, Let it Go 吧 : )

经过每天25小时的高强度网上冲浪, 我找到一个在写解析器时比正则表达式要更方便的 crate:
[nom](https://github.com/Geal/nom)

nom, 发音类似大口咀嚼时发出的声音, 比喻这个 crate 会一口一口吞掉你的数据.

## quick start

以 nom README 上16进制颜色值解析器为例, 简要说明下 nom 的一些概念和常见函数

```rust
use nom::{
  IResult,
  bytes::complete::{tag, take_while_m_n},
  combinator::map_res,
  sequence::tuple
};

#[derive(Debug,PartialEq)]
pub struct Color {
  pub red:   u8,
  pub green: u8,
  pub blue:  u8,
}

fn from_hex(input: &str) -> Result<u8, std::num::ParseIntError> {
  u8::from_str_radix(input, 16)
}

fn is_hex_digit(c: char) -> bool {
  c.is_digit(16)
}

fn hex_primary(input: &str) -> IResult<&str, u8> {
  map_res(
    take_while_m_n(2, 2, is_hex_digit),
    from_hex
  )(input)
}

fn hex_color(input: &str) -> IResult<&str, Color> {
  let (input, _) = tag("#")(input)?;
  let (input, (red, green, blue)) = tuple((hex_primary, hex_primary, hex_primary))(input)?;

  Ok((input, Color { red, green, blue }))
}

fn main() {}

#[test]
fn parse_color() {
  assert_eq!(hex_color("#2F14DF"), Ok(("", Color {
    red: 47,
    green: 20,
    blue: 223,
  })));
}
```

对于一个16进制颜色值, 其以 "#" 开头, 接着6个16进制数(0-9和a-f,大小写不敏感), 每2个值构成一组, 从左到右为 RGB 通道值, 可以简写为 `#R(hex, hex)G(hex, hex)B(hex, hex)`.

因此, 解析器逻辑可以概括为, 去掉开头的 "#", 如果随后的6个字符为16进制数, 则分为三组, 并将每组的值从16进制转换为10进制.

匹配某个模式这个需求在解析过程中普遍存在, nom 提供了 `tag`, `tag` 会匹配(只匹配开头)你给出的字符模式, 并返回匹配的模式和余下字符, 如果不匹配, 则返回错误.

```rust
let (input, _) = tag("#")(input)?;
````

`tag("#")` 返回的是一个函数, 所以我们可以用待解析的字符作为参数, 调用 `tag("#")` 的返回值, 函数返回值为 `IResult<Input, Input, Error>`, 其中 `Input` 为函数输入参数类型, 返回值第一个值为去掉匹配模式后的输入值(这里为字符串切片), 第二个值为 pattern, 第三为错误值.

`IResult` 实现了 `Error` trait, 因此可以用 `?` 快速失败, 其相当于 `std::result::Result<Ok(remaining, pattern), Err>`, 在 nom 中绝大多数解析函数返回值都是这种形式.

```rust
use nom::{IResult, bytes::complete::tag};

fn parse(input: &str) -> IResult<&str, &str> {
    tag("#")(input)
}
fn main() {
    let (remain, pattern) = parse("#ffffff").unwrap();
    println!("{}, {}",remain, pattern);
}
```

输出 `ffffff, #`.

接着要对剩下字符做解析, 拿出两个字符, 判断这字符是否是16进制数, 如果是, 则将其转换为10进制, Rust 有 `take_while`, nom 提供了扩展性更好的 `take_while_m_n`, m, n 分为
最少和最多匹配数

因此函数可以这样调用 `take_while_m_n(2, 2, |c: &char| c.is_digit(16))`, 在 Rust 中可以对 `Result` 使用 `map`, nom 也有类似函数 `map_res`, 按下面的方式调用

```rust
map_res(
    take_while_m_n(2, 2, |c: &char| c.is_digit(16)),
    to_decimal
)(input)
```

会先对 `input` 应用 `take_while_m_n(2, 2, |c: &char| c.is_digit(16))`, 如果 Ok 则对结果应用 `to_decimal` 转换为10进制

```rust
fn from_hex(input: &str) -> Result<u8, std::num::ParseIntError> {
  u8::from_str_radix(input, 16)
}

fn is_hex_digit(c: char) -> bool {
  c.is_digit(16)
}

fn hex_primary(input: &str) -> IResult<&str, u8> {
  map_res(
    take_while_m_n(2, 2, is_hex_digit),
    from_hex
  )(input)
}
```

现在要对输入应用三次 `hex_primary`, 用 for 循环? 不 nom 有更趁手的工具 `tuple`, `tuple` 接受一组组合子, 将组合子按顺序应用到输入上, 然后按顺序返回以元组返回解析结果

```rust
tuple((hex_primary, hex_primary, hex_primary))(input)?
```

把上面的函数组合组合起来, 一个16进制颜色值解析器就完成了

```rust
fn hex_color(input: &str) -> IResult<&str, Color> {
  let (input, _) = tag("#")(input)?;
  let (input, (red, green, blue)) = tuple((hex_primary, hex_primary, hex_primary))(input)?;

  Ok((input, Color { red, green, blue }))
}
```

如果你熟悉 nom, 那么这函数功能非常清晰. 它接受一个类型为 `&str` 的输入值, 如果输入值以 "#" 开头, 取余下的字符, 尝试连续应用三次 `hex_primary` 返回元组. 清晰明了.

## 总结

通过上面的小例子展示 nom 的用法和风格, 特别是 `tag`, `map_res`, `tuple` 和 `IResult` 这几个函数或
数据结构的使用, 它们在 nom 被广泛使用, 熟悉它们可以帮我们更快更高效地使用 nom 构建功能丰富更复杂的解析器.
根据作者在 5.0 版本以后的倡议, 以后的例子都尽量采用函数而不是宏, 因为我个人在阅读某些依赖 nom 的项目时,
大量使用宏确实会导致代码易读性下降, 而且 Rust 编辑器或 IDE 对宏的支持都不太好, 这导致这些代码既不好读, 也不容易写或 debug.

下一篇会尝试用 nom 写一个 S 表达式解析器, 这个例子同样来自 nom 文档, 心急的可以直接移步[项目文档](https://github.com/Geal/nom/blob/master/examples/s_expression.rs).
这个例子将展示如何从基本的元素开始, 一步步把 nom 赋予的简单有效的组合子通过递归等方式组合起来, 最后实现 S 表达式解析器.

最后, 在时常抽风的 Github 上冲浪翻文档不易, 如果喜欢, 求赞支持.
