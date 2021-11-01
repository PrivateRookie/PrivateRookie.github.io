# Python Rust 迭代器对比

---

迭代是数据处理的基石，而 Python 中所有集合都可以迭代，这是 Python 让使用者感到非常方便的特征之一。

下面是一些在 Python 中经常使用的迭代模式
```python
# 列表
for i in [1, 2, 3, 4]:
    print(i)

# 字典
di = {'a': 1, 'b': 2, 'c': 3}
# 迭代键
for k in di.keys():
    print(k)
# 迭代键值
for k, v in di.items():
    print('{}: {}'.format(k, v))
```
除了基本数据类型，Python 也支持为自定义的数据类型实现迭代器协议。Python 解释器在需要迭代对象 x 时会自动调用 iter(x)。
内置的 iter 函数有如下作用。

1. 检查对象是否实现了 `__iter__` 方法，如果实现了就调用它，`__iter__` 方法返回一个迭代器
2. 如果没有实现 `__iter__` 方法，但是实现了 `__getitem__` 方法，Python 会创建一个迭代器，尝试按顺序（从索引0）获取元素。
3. 如果上述两个尝试失败，Python 会抛出 TypeError 异常，提示该元素不可迭代。

所以如果我们要让某个对象是可迭代对象，只需要实现 `__iter__`，这个方法要求返回一个迭代器，那什么是迭代器呢？ Python 中标准的迭代器接口有两个方法。

`__next__`

返回下一个可用的元素，如果元素，抛出 StopIteration 异常。

`__iter__`

返回迭代器自身，即 self，以便在应该使用可迭代对象的地方使用迭代器，如 for 循环中。

这里需要说明的一点是，可迭代对象与迭代器是不同的，《流畅的 Python》这样定义**可迭代对象**
> 使用 iter 内置函数可以获取迭代器的对象。如果对象实现了能返回迭代器的 __iter__ 方法，那么对象就是可迭代的。序列都可以迭代；实现了 __getitem__ 方法，而且其参 数是从零开始的索引，这种对象也可以迭代
>
而**迭代器**则定义为
> 迭代器是这样的对象：实现了无参数的 __next__ 方法，返回序列中的下一个元素；如 果没有元素了，那么抛出 StopIteration 异常。Python 中的迭代器还实现了 __iter__ 方 法，因此迭代器也可以迭代。

也就是说每次对可迭代对象调用 iter(x) 都将返回一个**新的**迭代器。

那如果为一个可迭代对象实现 `__next__` 方法，即把这个可迭代对象变成自身的可迭代对象会怎样呢？没人阻止你这样做，但当你真正为这个对象实现这两个方法时，你会发现麻烦不断。举个例子

```python
class MyData:
    def __init__(self, values):
        # 假设 value 为列表
        self.values = values

    def __iter__(self):
        return self

    def __next__(self):
        # ???
        raise NotImplementedError()
```
按照协议 `__next__` 应该返回下一个元素或者抛出 StopIteration，显然我们需要一个属性存储当前迭代位置，所以应该似乎应该这样写

```python
class MyData:
    def __init__(self, values):
        self.values = values
        # 记录当前迭代位置
        self.current = 0

    def __iter__(self):
        # 每次调用重头开始迭代
        self.current = 0
        return self

    def __next__(self):
        if self.current < len(self.values):
            value = self.values[self.current]
            self.current += 1
            return value
        else:
            raise StopIteration
```
但考虑这样一种情况，我们调用2次 iter，交替迭代获得的2个迭代器，预期行为应该是2个迭代器不会干涉，但如果按上述代码实现 MyData 对象行为并不符合预期。

```python
data = MyData([1, 2, 3, 4, 5])

data_iter1 = iter(data)
print(next(data_iter1)) # 结果为1
print(next(data_iter1)) # 结果为2

data_iter2 = iter(data)
print(next(data_iter2)) # 结果为1

print(next(data_iter1)) # 预期为3，但得到2
```
如果把 current 属性变为列表，每次调用 iter 增加一个元素表示新的迭代器当前位置呢？但又会导致 `__next__` 变得非常复杂，因为它必须找到不同迭代器对应当前位置，这样才能保证正确的迭代行为。为什么我们的迭代实现如此复杂呢？根本原因在于 `__iter__` 总是返回自身，换言之，调用 iter 的迭代器都是**一样**，这其实破坏了 **每次调用 iter 返回新的迭代器** 这一设计。

解决难题办法很简单，遵循设计，把可迭代对象和迭代器拆开。

```rust
class MyData:
    def __init__(self, values):
        self.values = values

    def __iter__(self):
        return DataIterator(list(self.values))


class DataIterator:
    def __init__(self, values):
        self.values = values
        self.current = 0

    def __iter__(self):
        return self

    def __next__(self):
        if self.current < len(self.values):
            value = self.values[self.current]
            self.current += 1
            return value
        else:
            raise StopIteration
```
现在 `__iter__` 将会返回新的迭代器，每个迭代器都保存着自身状态，这让我们不必费心费力第维护迭代器状态。

所以，把可迭代对象变成其自身的迭代器是条歧路，反设计的。

在 Rust 中，迭代也遵循着相似的设计，Rust 中实现了 `Iterator` 特性的结构体就被认为是可迭代的。

我们可以像 Python 那样使用 for 循环迭代
```rust
let v1 = vec![1, 2, 3, 4, 5];

for item in v1 {
    println!("{}", item);
}
```

[std::iter::Iterator](https://doc.rust-lang.org/std/iter/trait.Iterator.html) 只要求实现 `next` 方法即可，下面是一个[官方文档](https://doc.rust-lang.org/std/iter/index.html#implementing-iterator)中的例子

```rust
// 首先定义一个结构体，作为“迭代器”
struct Counter {
    count: usize,
}

// 实现静态方法 new，相当于构造函数
// 这个方法不是必须的，但可以让我更加方便
// 地使用 Counter
impl Counter {
    fn new() -> Counter {
        Counter { count: 0 }
    }
}

// 实现 Iterator 特性
impl Iterator for Counter {
    // 确定迭代器的返回值类型
    type Item = usize;

    // 只有 next() 是必须实现的方法
    // Option<usize> 也可以写成 Option<Self::Item>
    fn next(&mut self) -> Option<usize> {
        // 增加计数
        self.count += 1;

        // 到 5 就返回 :)
        if self.count < 6 {
            Some(self.count)
        } else {
            None
        }
    }
}

let mut counter = Counter::new();

let x = counter.next().unwrap();
println!("{}", x);

let x = counter.next().unwrap();
println!("{}", x);

let x = counter.next().unwrap();
println!("{}", x);

let x = counter.next().unwrap();
println!("{}", x);

let x = counter.next().unwrap();
println!("{}", x);
```

与 for 循环使用时，Python 使用 StopIteration 告诉编译是时候定制循环了，在 Rust 则是 None，所以 `next` 方法返回值为 `Option<Self::Item>`。其实使用 for 循环是一种语法糖

```rust
let values = vec![1, 2, 3, 4, 5];

for x in values {
    println!("{}", x);
}
```

去掉语法糖后相当于

```rust
let values = vec![1, 2, 3, 4, 5];
{
    let result = match IntoIterator::into_iter(values) {
        mut iter => loop {
            let next;
            match iter.next() {
                Some(val) => next = val,
                None => break,
            };
            let x = next;
            let () = { println!("{}", x); };
        },
    };
    result
}
```
编译器会对 values 调用 into_iter 方法，获取迭代器，接着匹配迭代器，一次又一次地调用迭代器的 next 方法，直到返回 None，这时候终止循环，迭代结束。

这里又涉及到另一个特性 [std::iter::IntoIterator](https://doc.rust-lang.org/std/iter/trait.IntoIterator.html)，这个特性可以把某些东西变成一个迭代器。

IntoInterator 声明如下：
```rust
pub trait IntoIterator
where
    <Self::IntoIter as Iterator>::Item == Self::Item,
{
    type Item;
    type IntoIter: Iterator;
    fn into_iter(self) -> Self::IntoIter;
}
```

类比于 Python 中的概念，可以做出以下结论：
1. 实现了 IntoIterator 特性的结构体是一个“可迭代对象”
2. 实现了 Iterator 特性的结构体一个“迭代器”
3. for 循环会尝试调用结构的 into_iter 获得一个新的“迭代器”，当迭代器返回 None 时提示迭代结束

基于以上结论，我们可以实现 Python 例子中类似的代码

```rust
#[derive(Clone)]
struct MyData{
    values: Vec<i32>,
}

struct DataIterator {
    current: usize,
    data: Vec<i32>,
}

impl DataIterator {
    fn new(values: Vec<i32>) -> DataIterator {
        DataIterator {
            current: 0,
            data: values
        }
    }
}

impl Iterator for DataIterator {
    type Item = i32;

    fn next(&mut self) -> Option<i32> {
        if self.current < self.data.len() {
            let ret =  Some(self.data[self.current]);
            self.current += 1;
            ret
        } else {
            None
        }
    }
}

impl IntoIterator for MyData {
    type Item = i32;
    type IntoIter = DataIterator;

    fn into_iter(self) -> DataIterator {
        DataIterator::new(self.values)
    }
}

fn main() {
    let data = MyData { values: vec![1, 2, 3, 4] };
    for item in data {
        println!("{}", item);
    }
}
```

## 总结
Rust 不愧是一门多范式的现代编程语言，如果你之前对某个语言有相当深入的了解，在学习 Rust 是总会有“喔，这不是xxx吗”的感觉。虽然之前阅读过 《流畅的Python》，但在可迭代对象与迭代器这一章并没有太多影响，因为在使用 Python 时真正要我实现迭代接口的场景非常少；直到最近学习 Rust，在尝试使用 Rust 的 Iterator 特性为我的结构实现与 for 循环交互时被 Iterator 和 IntoInterator 特性高的有些蒙圈。最后是靠着 Python 和 Rust 相互对比，弄清迭代器与可迭代对象的区别后才感觉自己真正弄懂了迭代这一重要特性。

## 延申阅读

1. [《流畅的Python》]([http://www.ituring.com.cn/book/1564](http://www.ituring.com.cn/book/1564)) - 第14章，可迭代的对象、迭代器和生成器
2. [std::iter::IntoIterator - Rust](https://doc.rust-lang.org/std/iter/trait.IntoIterator.html)
3. [std::iter::Iterator - Rust](https://doc.rust-lang.org/std/iter/trait.Iterator.html)
4. 《Rust 编程之道》 6.3 迭代器