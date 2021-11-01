# 捋捋 Rust 中的 impl Trait 和 dyn Trait

---

## 缘起

一切都要从年末换工作碰上疫情, 在家闲着无聊又读了几首诗, 突然想写一个可以浏览和背诵诗词的 TUI 程序说起.
我选择了 [Cursive](https://github.com/gyscos/cursive) 这个 Rust TUI 库. 在实现时有这么一个函数, 它会根据参数的不同返回某个组件(如 Button, TextView 等).
在 `Cursive` 中, 每个组件都实现了 `View` 这个 trait, 最初这个函数只会返回某个确定的组件, 所以函数签名可以这样写

```rust
fn some_fn(param: SomeType) -> Button
```

随着开发进度增加, 这个函数需要返回 Button, TextView 等组件中的一个, 我下意识地写出了类似于下面的代码

```rust
fn some_fn(param1: i32, param2: i32) -> impl View {
    if param1 > param2 {
        // do something...
        return Button {};
    } else {
        // do something...
        return TextView {};
    }
}
```

可惜 Rust 编译器一如既往地打脸, Rust 编译器报错如下

```bash
  --> src\main.rs:19:16
   |
13 | fn some_fn(param1: i32, param2: i32) -> impl View {
   |                                         --------- expected because this return type...
...
16 |         return Button {};
   |                --------- ...is found to be `Button` here
...
19 |         return TextView {};
   |                ^^^^^^^^^^^ expected struct `Button`, found struct `TextView`

error: aborting due to previous error

For more information about this error, try `rustc --explain E0308`.
```

从编译器报错信息看函数返回值虽然是 `impl View` 但其从 `if` 分支推断返回值类型为 `Button` 就不再接受 `else` 分支返回的 `TextView`. 这与 Rust 要求 `if else` 两个分支的返回值类型相同的特性一致.
那能不能让函数返回多种类型呢? Rust 之所以要求函数不能返回多种类型是因为 Rust 在需要在
编译期确定返回值占用的内存大小, 显然不同类型的返回值其内存大小不一定相同.
既然如此, 把返回值装箱, 返回一个胖指针, 这样我们的返回值大小可以确定了, 这样也许就可以了吧.
尝试把函数修改成如下形式:

```rust
fn some_fn(param1: i32, param2: i32) -> Box<View> {
    if param1 > param2 {
        // do something...
        return Box::new(Button {});
    } else {
        // do something...
        return Box::new(TextView {});
    }
}
```

现在代码通过编译了, 但如果使用 Rust 2018, 你会发现编译器会抛出警告:

```bash
warning: trait objects without an explicit `dyn` are deprecated
  --> src\main.rs:13:45
   |
13 | fn some_fn(param1: i32, param2: i32) -> Box<View> {
   |                                             ^^^^ help: use `dyn`: `dyn View`
   |
   = note: `#[warn(bare_trait_objects)]` on by default
```

编译器告诉我们使用 trait object 时不使用 `dyn` 的形式已经被废弃了, 并且还
贴心的提示我们把 `Box<View>` 改成 `Box<dyn View>`, 按编译器的提示修改代码, 此时代码
no warning, no error, 完美.

但 `impl Trait` 和 `Box<dyn Trait>` 除了允许多种返回值类型的之外还有什么区别吗? `trait object` 又是什么? 为什么 `Box<Trait>` 形式的返回值会被废弃而引入了新的 `dyn` 关键字呢?

## 埋坑

`impl Trait` 和 `dyn Trait` 在 Rust 分别被称为静态分发和动态分发. 在第一版的 Rust Book 这样解释分发(dispatch)

> When code involves polymorphism, there needs to be a mechanism to determine which specific version is actually run. This is called ‘dispatch’. There are two major forms of dispatch: static dispatch and dynamic dispatch. While Rust favors static dispatch, it also supports dynamic dispatch through a mechanism called ‘trait objects’.

即当代码涉及多态时, 需要某种机制决定实际调用类型. Rust 的 Trait 可以看作某些具有通过特性类型的集合, 以上面代码为例, 在写代码时我们不关心具体类型, 但在编译或运行时必须确定 `Button` 还是 `TextView`.
静态分发, 正如静态类型语言的"静态"一词说明的, 在编译期就确定了具体调用类型. Rust 编译器会通过单态化(Monomorphization) 将泛型函数展开.

假设 `Foo` 和 `Bar` 都实现了 `Noop` 特性, Rust 会把函数

```rust
fn x(...) -> impl Noop
```
展开为

```rust
fn x_for_foo(...) -> Foo
fn x_for_bar(...) -> Bar
```

(仅作原理说明, 不保证编译会这样展开函数名).

通过单态化, 编译器消除了泛型, 而且没有性能损耗, 这也是 Rust 提倡的形式, 缺点是过多展开可能会导致编译生成的二级制文件体积过大, 这时候可能需要重构代码.

静态分发虽然有很高的性能, 但在文章开头其另一个缺点也有所体现, 那就是无法让函数返回多种类型, 因此 Rust 也支持通过 trait object 实现动态分发. 既然 Trait 是具有某种特性的类型的集合, 那我们可以把 Trait 也看作某种类型, 但它是"抽象的", 就像OOP中的抽象类或基类, 不能直接实例化.

Rust 的 trait object 使用了与 c++ 类似的 `vtable` 实现, trait object 含有1个指向实际类型的 `data` 指针, 和一个指向实际类型实现 trait 函数的 vtable, 以此实现动态分发. 更加详细的介绍可以在 [Exploring Dynamic Dispatch in Rust](https://alschwalm.com/blog/static/2017/03/07/exploring-dynamic-dispatch-in-rust/)看到.
既然 trait object 在实现时可以确定大小, 那为什么不用 `fn x() -> Trait` 的形式呢? 虽然 trait object 在实现上可以确定大小, 但在逻辑上, 因为 Trait 代表类型的集合, 其大小无法确定. 允许 `fn x() -> Trait` 会导致语义上的不和谐.
那 `fn x() -> &Trait` 呢? 当然可以! 但鉴于这种场景下都是在函数中创建然后返回该值的引用, 显然需要加上生命周期:

```rust
fn some_fn(param1: i32, param2: i32) -> &'static View {
    if param1 > param2 {
        // do something...
        return &Button {};
    } else {
        // do something...
        return &TextView {};
    }
}
```

我不喜欢添加额外的生命周期说明, 想必各位也一样. 所以我们可以用拥有所有权的 `Box` 智能指针避免烦人的生命周期说明. 至此 `Box<Trait>` 终于出现了.
那么问题来了, 为什么编译器会提示 `Box<Trait>` 会被废弃, 特地引入了 `dyn` 关键字呢?
答案可以在 [RFC-2113](https://github.com/rust-lang/rfcs/blob/master/text/2113-dyn-trait-syntax.md) 中找到.

RFC-2113 明确说明了引入 `dyn` 的原因, 即语义模糊, 令人困惑, 原因在于没有 `dyn` 让 Trait 和 trait objects 看起来完全一样, RFC 列举了３个例子说明.

第一个例子, 加入你看到下面的代码, 你直到作者要干什么吗?

```rust
impl SomeTrait for AnotherTrait impl<T> SomeTrait for T where T: Another
```

你看懂了吗? 说实话我也看不懂 : ) PASS

第二个例子, `impl MyTrait {}` 是正确的语法, 不过这样会让人以为这会在 Trait 上添加默认实现, 扩展方法或其他 Trait 自身的一些操作.
实际上这是在 trait object 上添加方法.

如在下面代码说明的, Trait 默认实现的正确定义方法是在定义 Trait 时指定, 而不应该在 `impl Trait {}` 语句块中.

```rust
trait Foo {
    fn default_impl(&self) {
        println!("correct impl!");
    }
}

impl Foo {
    fn trait_object() {
        println!("trait object impl");
    }
}

struct Bar {}

impl Foo for Bar {}

fn main() {
    let b = Bar{};
    b.default_impl();
    // b.trait_object();
    Foo::trait_object();
}
```

`Bar` 在实现了 `Foo` 后可以通过 `b.default_impl` 调用, 无需额外实现, 但 `b.trait_object` 则不行, 因为 `trait_object` 方法是 `Foo` 的
trait object 上的方法.

如果是 Rust 2018 编译器应该还会显示一条警告, 告诉我们应该使用 `impl dyn Foo {}`

第三个例子则以函数类型和函数 trait 作对比, 两者差别只在于首字母是否大写(Fn代表函数trait object, fn则是函数类型), 难免会把两者弄混.

更加详细的说明可以移步 [RFC-2113](https://github.com/rust-lang/rfcs/blob/master/text/2113-dyn-trait-syntax.md).

## 总结

`impl trait` 和 `dyn trait` 区别在于静态分发于动态分发, 静态分发性能
好, 但大量使用有可能造成二进制文件膨胀; 动态分发以 trait object 的概念通过虚表实现, 会带来一些运行时开销. 又因 trait object 与 Trait 在不引入 `dyn` 的情况下经常导致语义混淆, 所以 Rust 特地引入 `dyn` 关键字, 在 Rust 2018 中已经稳定.

## 引用

以下是本文参考的资料

- [Rust Edition Guide](https://doc.rust-lang.org/nightly/edition-guide/rust-2018/trait-system/impl-trait-for-returning-complex-types-with-ease.html#argument-position)
- [impl trait 社区跟踪](https://github.com/rust-lang/rust/issues/34511)
- [RFC-2113](https://github.com/rust-lang/rfcs/blob/master/text/2113-dyn-trait-syntax.md)
- [Trait and Trait Object](https://joshleeb.com/posts/rust-traits-and-trait-objects/)
- [Dynamic vs Static Dispatch](https://lukasatkinson.de/2016/dynamic-vs-static-dispatch/)
- [Exploring Dynamic Dispatch in Rust](https://alschwalm.com/blog/static/2017/03/07/exploring-dynamic-dispatch-in-rust/)