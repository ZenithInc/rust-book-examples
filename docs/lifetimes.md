## 深入理解 Rust 中的生命周期 ##

生命周期的存在，是因为 Rust 的编译器还不足够智能，不能百分百识别是否存在悬垂指针。所以需要开发者显式地加上生命周期的标注，告诉编译器：“我能行，我没问题，悬垂指针？不存在的！“

### 不求同生，但愿共死 ###

经常在影视中看到磕头拜把子的时候，三柱高香一句口号：“不求同年同月生，但求同年同月死”。

其实这句话就非常形象地点出了生命周期的核心要义，我们从一个例子说起, 写一个函数比较两个字符串的长度，返回更长的字符串:
```rust
fn longest(x: &str, y: &str) -> &str {
    if x.len() > y.len() { x } else { y }
}

fn main() {
    let x = String::from("Hello");
    let y = String::from("Rust");
    longest(&x, &y);
}
```
如果有其他编程语言的经验，这样的写法非常顺畅自然。但是在 Rust 中，这是编译不通过的:
```rust
$ cargo check
    Checking app v0.1.0 (/home/user/emo)
error[E0106]: missing lifetime specifier
 --> src/main.rs:1:33
  |
1 | fn longest(x: &str, y: &str) -> &str {
  |               ----     ----     ^ expected named lifetime parameter
  |
  = help: this function's return type contains a borrowed value, but the signature does not say whether it is borrowed from `x` or `y`
help: consider introducing a named lifetime parameter
  |
1 | fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
  |           ++++     ++          ++          ++
```
错误的输出分为两部分：第一部分说明错误的原因，期待在返回值 `&str` 上加上生命周期的标注，第二部分告诉我们如何修改，修改前后的对比如下:
```rust
// 修改之前
fn longest(x: &str, y: &str) -> &str {}
// 修改之后
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {}
```
生命周期标注是通过泛型参数的形式来呈现的:

* `longest<'a>`: 声明了一个生命周期参数 `'a`，当然你也可以任意命名，比如 `'k`、`'geek`、`'duck`，只是习惯上使用 `'a`, 如果有多个则 `'a`、`'b`、`'c`；

* `x: &'a str`：表示参数 `x` 的生命周期是 `'a`, 当然 `y` 也一样；

* `-> &'a str`: 表示返回值的生命周期也是 `'a`。

生命周期的标注并不影响程序的运行，只是因为编译器无法确定返回的引用是 `x` 还是 `y`, 也无法确定 `x` 和 `y` 这两兄弟谁活得更久一些，所以需要导演（开发者）告诉它：`x` 和 `y` 和返回值 `&str` 的寿命都是一样的，都是 `'a`，能够达到剧情效果，一同赴死。

但是，聪明的观众可以会有这样的疑问：这个程序，一眼看过去就知道 `x` 和 `y` 的生命周期是一样的，都在 `main` 函数中定义，是前后脚出生的，当 `main` 函数结束之后，也就销毁了。这么明显，为什么编译器不知道？

原因也很简单，因为观众有上帝视角，就像看一些宫斗言情剧一样，从第一集就能推导出结局，必然是傻白甜的女主吃尽苦头苦尽甘来......来做皇后。

**编译器在编译 `longest` 函数的时候，并不会根据上下文中的作用域来确定其生命周期是否是借用安全的**。这样做的成本非常高，编译复杂度上升，编译时间延长。所以，编译器并没有上帝视角，毕竟它不是万能的上帝。

我们来总结一下，不管 `x` 和 `y` 是否真心真实愿意一起赴死，编译器是不知道的，所以需要开发者显示的标注。一旦开发者做了生命周期的标注之后，编译器就会按照开发者的标注，对所有调用的地方进行严格的生命周期检查，从而避免在运行时出现悬浮指针。

比如下面这个例子，即使做了生命周期的标注，也是编译不通过的:
```rust
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    let z = x.to_string() + y;
    if z.len() > y.len() { &z } else { y }
}

fn main() {
    let x = String::from("Hello");
    let y = String::from("Rust");
    longest(&x, &y);
}
```
在这个示例中，我们引入了局部变量 `z`, 它的生命周期是从定义语句开始到函数结束为止。和 `x` 以及 `y` 的生命周期 `'a` 并并不一致，活得更短。编译器根据我们的标注去检查，就不会通过。

所以，对于多个多个引用而言，可以猴年马月生，但必须同年同月死。

### 生命周期省略(Elision) ###

生命周期标注对于开发者而言，是不小的心智负担。因此 Rust 也在编译器根据一些规则，依照以下的规则进行判断，如果符合，可以省略标注:

1. 如果每个输入的引用参数都会获得各自的生命周期参数
```rust
fn longest(x: &i32, y: &i32) {}
// 对于编译器而言，就是如下形式
fn longest<'a, 'b>(x: &'a i32, y: &'b i32) {}
```
2. 如果只有一个输入生命周期参数，那么会赋值给所有的输出引用
```rust
fn single(x: &i32) -> &i32 {}
// 对于编译器而言，就是如下形式
fn single<'a>(x: &'a i32) -> &'a i32 {} 
```
3. 如果有多个输入的生命周期参数，且其中有一个是 `&self` 或者 `&mut self` ，则生命周期的会被赋予所有的输出引用。
```rust
struct Artisan {};

impl Artisan {
  fn dancing(&self) -> &i32 {}
  // 对于编译器而言，就是如下形式
  fn dancing<'a>(&'a self) -> &'a i32 {}
}
```
当面三种情况，包含了打多数场景，满足其一就可以不用去标注生命周期，否则则需要。

> 上面的省略规则只是对函数以及方法签名有效，对于结构体字段、关联类型都不适用。

### 静态生命周期 ###

静态的生命周期使用 `'static` 来表示，其周期是从程序开始知道程序结束为止。标注了 `'static` 后，在整个程序运行期间都是有效的。

比如说，字符串的字面量:
```rust
let s &'static str = 'Hello, World';
```
静态的全局变量:
```rust
static AQUA: &str = '00FFFF';
```
另外还有函数指针和编译期间的常量。

静态生命周期的数据通常存储在只读数据去或者全局数据去，但是不要因为看到 `static` 就以为它是不可变的。Rust 中，也可以将其生命为可变的，放在 `unsafe {}` 块中，自行保证并发安全:
```rust
static CONFIG: OnceCell<Arc<Config>> = OnceCell::new();
```
这个 CONFIG 就是一个静态的，而且是可变的，线程安全的，但是保障线程安全是通过 CAS(Compare Exchange Swap) 来实现的。

### 结构体和生命周期 ###

在标准库中，有一个 `Cow<'a, B>` 的枚举，其主要作用是实现延迟拷贝(Copy-on-Write)。例如我有一个修正 URL Path 的方法，如下:
```rust
use std::borrow::Cow;

fn normalize_path<'a>(path: &'a str) -> Cow<'a, str> {
    if path.contains('\\') {
        let s = path.replace('\\', "/");
        // 写时替换
        Cow::Owned(s)
    } else {
        // 不需要替换，返回 &'a str，零拷贝
        Cow::Borrowed(path)
    }
}

fn main() {
    let path1 = r"resources\human.jpg";
    let path2 = "resources/women.jpg";

    let path1 = normalize_path(path1);
    let path2 = normalize_path(path2);
    // resources/human.jpg, resources/women.jpg
    println!("{path1}, {path2}");
}
```
我们接着来看 `Cow<'a, B>` 的源码:
```rust
pub enum Cow<'a, B>
where
    B: 'a + ToOwned + ?Sized,
{
    Borrowed(&'a B),
    Owned(<B as ToOwned>::Owned),
}
```
`Cow::Borrowed(&'a B)` 表示“借用一个生命周期 `'a` 内有效的数据，这就意味着 B 或者其内部的应用必须活的至少和 `'a` 一样长，这样才不会导致悬垂引用。具体到上面这个例子中，就是替换后的 `s` 还是没有替换的 `path` 的生命周期都必须比 `'a` 要长。

### 方差（Variance） ###

方差有**协变（Convariance）**、**逆变（Contravariant）**和**不变（Invariant）**三种。协变和逆变都描述子类型系统的，而不变指的是不允许子类型系统。

#### 协变 ####

下面这个简单的例子，非常有助于我们理解什么是协变:
```rust
static GLOBAL: i32 = 123;

fn needs_short<'short>(x: &'short i32) {
    println!("{}", x);
}

fn main() {
    needs_short(&GLOBAL); // 'static 的生命周期是最长的
    let a = &GLOBAL;
    // 同时，这个方法也可以传入生命周期更短的
    // 只要符合生命周期的规则：实参的生命周期大于等于形参
    needs_short(a);  
}
```
以这个例子来说，`need_short<'short>(x: &'short i32)` 要求传入的是一个 `&'short i32` 的类型，但是实际传入的是 `&'static i32` 类型。对于 Rust 这种遵循显式哲学的强类型语言来说，这是两种不同的类型。所以，同样的逻辑，你可能需要写两个方法，只是为了匹配不同的生命周期。

但是这个例子是可以编译通过的，这就是因为协变：

> **如果一个复合类型`F<T>`在其类型参数`T`上是协变的，那么`T1` 是 `T2` 的子类型时, `F<T1>` 也是 `F<T2>` 的子类型**。

这句话不是很好理解，要么多读几遍，要么放弃去理解这句话。我们以上面这个示例来解释：

* `&'short i32` 是引用类型，也是一种复合类型（还包括泛型, 例如`Vec<T>`、`Cow<T>`）；
* 参数 `T` 值得就是 `i32`，`T1` 就是 `&'static i32`, 而 `T2` 就是 `&'short i32`；
* 如果 `T1` 的生命周期 `static` 比 `T2`中的生命周期 `short` 长，表示为 `'static: 'short`；
* 那么 `T1` 就是 `T2` 的子类型，那么 `T1` 就可以安全地代替 `T2`，因为 `T1` 的生命周期更长；
* 而 `F<T1>` 就是 `F<T2>` 的子类型；

> 我没写错，你也没看错，生命周期更长的是子类型，你可以类比生面向对象中的继承，子类型能够完成更多的事情，或者想着“青出于蓝而胜于蓝”

我们上文中，提到了 `Cow<T>` 也是一个协变的例子，你可以传入一个 `Cow<'long, B>` 或者 `Cow<'short, B>`。生命周期通过在编译期间的借用检查，防止了在运行时出现悬垂指针。而协变在对生命周期和类型系统的增强，可以让统一接口、代码复用、类型更加灵活。

#### 逆变 ####

在理解了协变之后，逆变就会好理解一些，其实和协变相反。

> **如果一个符合类型 `F<T>` 在其类型参数 `T` 上是逆变的，那么当 `T1` 是 `T2` 的子类型时，`F<T2>` 反而是 `F<T1>` 的子类型。 **

关于 `T` 子类型的判断还是和协变是一致的，生命周期更长的是子类型，但是整个类型 `F<T>` 的判断就和协变是相反的。这种情况在 Rust 中基本上是少见的，主要体现在**函数指针的逆变**上，又分为参数类型的逆变和返回类型的逆变。下面这个例子是参数类型的逆变:
```rust
trait Animal {}

struct Dog;
impl Animal for Dog {}

fn feed_dog<F>(f: F)
where
    F: Fn(&Dog),
{
    let dog = Dog;
    f(&dog);
}

fn feed_animal(_: &dyn Animal) {
    println!("Feeding an animal");
}

fn main() {
    // 允许传入父类型
    // 编译不通过，类型不匹配
    feed_dog(feed_animal(d));
    // 如果要编译通过，改成如下形式:
    feed_dog(|d: &Dog| feed_animal(d));
}
```
逆变理论内涵在一个生命周期更长的高阶函数内，你可以引用一个生命周期更短的函数指针，这样做理论上也是安全的。所以逆变允许接收子类型参数的函数指针，赋值给接收更加宽泛（超类型）参数的函数指针。

### 总结 ###

生命周期在 Rust 中非常重要，和所有权、借用三足鼎立，缺一不可。故而，这篇文档花了那么长的篇幅来说明它，就是希望读者能够更加深入地理解生命周期。但尽管如此，仍然没有办法覆盖它的每一个知识点，只能挑重点来说，其他的后期再完善。