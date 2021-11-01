# 我经常使用的 Rust 小 Crate - [译]

原文链接： [Karol Kuczmarski's Blog – Small Rust crates I (almost) always use](http://xion.io/post/code/rust-little-crates.html#rust-little-crates)

---

因为 Rust 相对贫瘠的标准库，使用 Rust 不可避免地会引入不少第三发依赖。

这些第三方依赖用于解决一些“自带电池”更丰富的依靠内建库就可以解决的问题。
一个好例子就是 Python 的 `re` 模块，这相当于 Rust [regex](http://docs.rs/regex) crate。

正则表达式之类的问题是一类相对大的问题，拥有专门的库一点也不奇怪。对于一门语言，提供一个小库来解决一个很特化的则不那么常见。

就如同，一个函数/类型/宏 之类的问题，或者只比他们大一点。

在这篇博客，我们会快速浏览一系列必需的“小型库”

## either

Rust 有内建的 `Result` 类型，这是 `Ok` 和 `Err` 的集合。它构成了 Rust 中一般错误处理的基础。

从结构上来说，`Result<T, E>` 只是提供了 `T` 和 `E` 的替代。你可能想将这样
一个枚举类用于不同用途来代表错误处理。不幸的是，由于 `Result` 强烈的内在意
义，这种用法不符合 Rust 风格同时也令人疑惑（其实就是 `Result` 从名字到用法
都是高度明确的语义，如果使用在其他地方反而会造成疑惑）

这也是需要 [either](http://docs.rs/either) crate。它包含了下面的 `Either` 类型：

```rust
enum Either<L, R> {
    Left(L),
    Right(L),
}
```

虽然它与 `Result` 同构，但它并不带有强制的错误处理语义。而且它还提供了对称
组合器方法如 `map_left` 和 `right_and_then` 用于链式计算 `Eigther` 包含的值

## lazy_static

因为语言设计，Rust 不允许安全地使用全局可变变量。将全局可变变量引入你的代码
半标准方法是使用 [lazy_static](http://docs.rs/lazy_static) crate

但是，这个 crate 最重要的用法是声明延迟初始化复杂常量：

```rust
lazy_static! {
    static ref TICK_INTERVAL: Duration = Duration::from_secs(7 * 24 * 60 * 60);
}
```

这个技巧并不是完全透明，但直到 Rust 拥有[运行时表达式](https://github.com/rust-lang/rfcs/issues/322)，这就是你所能想到最好的办法

## maplit

为了与上述 crate 更好地配合，且使用与标准库中的 `vec![]` 类似语法，我们可以使用 [maplit](http://docs.rs/maplit)

它通过定义一些非常简单的 `hashmap!` 和 `hashset!` 宏，让你可以通过“字面量”
添加 `HashMap` 和 `HashSet`：

```rust
lazy_static! {
    static ref IMAGE_EXTENSIONS: HashMap<&'static str, ImageFormat> = hashmap!{
        "gif" => ImageFormat::GIF,
        "jpeg" => ImageFormat::JPEG,
        "jpg" => ImageFormat::JPG,
        "png" => ImageFormat::PNG,
    };
}
```

在 `hashmap!` 宏内部，`hashmap!` 会根据传入的字面量调用 `HashMap::insert`，接着返回已包含传入字面量的 `HashMap`。

## try_opt

在 Rust 引入 `?` 运算符之前(**目前已可以使用**)，在处理 `Result` 时传播错误的惯用手法是使用 `try!` 宏。

`try_opt` 为Option类型实现类似的宏，用于传播 `None`，这个宏的使用方法也相当直观：

```rust
fn parse_ipv4(s: &str) -> Option<(u8, u8, u8, u8)> {
    lazy_static! {
        static ref RE: Regex = Regex::new(
            r"^(\d{1,3})\.(\d{1,3})\.(\d{1,3})\.(\d{1,3})$"
        ).unwrap();
    }
    let caps = try_opt!(RE.captures(s));
    let a = try_opt!(caps.get(1)).as_str();
    let b = try_opt!(caps.get(2)).as_str();
    let c = try_opt!(caps.get(3)).as_str();
    let d = try_opt!(caps.get(4)).as_str();
    Some((
        try_opt!(a.parse().ok()),
        try_opt!(b.parse().ok()),
        try_opt!(c.parse().ok()),
        try_opt!(d.parse().ok()),
    ))
}
```

直到 Rust 支持 `?`，`try_opt!` 不失为一个可接受的 workaroud。

## exitcode

基本上每个主流操作系统中的一个常见约定，如果一个进程以不同于0（零）的代码退出，
表示进程发生错误，Linux进一步划分错误代码的空间，并且与BSD一起它还包括sysexits.h头文件包含一些更专业的代码。

许多程序和语言都采用了这些方法。在Rust中，也可以使用那些常见错误的半标准名称。需要做的就是将exitcode crate添加到您的项目依赖中：

```rust
fn main() {
    let options = args::parse().unwrap_or_else(|e| {
        print_args_error(e).unwrap();
        std::process::exit(exitcode::USAGE);
    });
```

除了 `USAGE` 或 `TEMPFAIL` 之类的常量之外，[exitcode](http://docs.rs/exitcode)
还为保存退出代码的整数类型定义了一个 `ExitCode` 别名。除其他外，也可以将它用作顶级函数的返回类型：

```rust
    let code = do_stuff(options);
    std::process::exit(code);
}

fn do_stuff(options: Options) -> exitcode::ExitCode {
    // ...
}
```

## enum-set

在 Java 中，有一种普通 `Set` 的特化接口用于枚举类型：[EnumSet](https://docs.oracle.com/javase/7/docs/api/java/util/EnumSet.html)。它的成员非常紧凑地表示为位而不是散列元素。

[enum_set](http://docs.rs/enum-set) 实现了一个相似（尽管不如那么强）的结构。对于一个 `#[repr(u32)]` 枚举类型：

```rust
#[repr(u32)]
#[derive(Clone, Copy, Debug Eq, Hash, PartialEq)]
enum Weekday {
    Monday, Tuesday, Wednesday, Thursday, Friday, Saturday, Sunday,
}
```

你可以创建一个其成员的 `EnumSet` :

```rust
let mut weekend: EnumSet<Weekday> = EnumSet::new();
weekend.insert(Weekday::Saturday);
weekend.insert(Weekday::Sunday);
```

只要你实现一个简单 trait，这个 trait 声明了怎么将这个枚举值转换为 u32 或怎么从 u32 转换而来：

```rust
impl enum_set::CLike for Weekday {
    fn to_u32(&self) -> u32            { *self as u32 }
    unsafe fn from_u32(v: u32) -> Self { std::mem::transmute(v) }
}
```

这样的优点是具有由单个无符号32位整数表示的集合结构，所有集合操作复杂性都是O（1），这些操作包括成员资格检查，两套联合，它们的交集，差异等等。

## antidote

作为实现“无畏并发”诺言一部分，Rust 在 `std::sync` 模块定义了许多同步原语。
`Mutex`，`RwLock` 和它们的类似机制有一个共同之处就是，如果一个线程在持有它们的情况下恐慌，
它们的锁会变得“中毒”。因此，获取锁定需要处理潜在的`PoisonError`。

然而，对于许多程序来说，锁定中毒甚至不是遥远的，而是一种直接的不可能的情况。
如果你遵循并发资源共享的最佳实践，你将不会持有多个指令的锁，没有解包或任何其他恐慌的机会！（）。
不幸的是，你无法静态地向Rust编译器证明这一点，因此它仍然需要你处理一个不可能发生的PoisonError。

如名所示 [antidote](http://docs.rs/antidote) 这正是其能提供帮助的地方。
在 antidote 中，您可以找到std :: sync提供的所有相同的锁和保护API，
只是没有PoisonError。在许多情况下，这种删除从根本上简化了接口，
例如将Result <Guard，Error>返回类型转换为Guard。

代价显而易见，那就是你需要保证所有持有“免疫性”锁的线程：

1. 完全不会恐慌；或者
2. 如果他们恐慌，不会将保护资源置于不一致的状态

就像之前提到过的那样，实现这一目标的最佳方法是将锁定保护的关键部分保持在最小和绝对可靠的状态。

## matches

模式匹配是Rust最重要的特性之一，但是一些相关的语言结构具有令人尴尬的缺点。例如，if let条件不能与布尔测试结合使用：

```rust
if let Foo(_) = x && y.is_good() {
```

因此需要额外的嵌套或完全不同的方法。

值得庆幸的是，为了帮助解决这种情况，有一个方便的[matches](http://docs.rs/matches) crate 。
除了与它同名的 `matches!` 宏：

```rust
if matches!(x, Foo(_)) && y.is_good() {
```

它也暴露了断言宏 `assert_match!` 和 `debug_assert_match!`，这些宏可用于生产和测试代码。

---