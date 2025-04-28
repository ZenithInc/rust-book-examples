# 不要害怕 Rust 中的指针 #

指针类型是强大的，但是它的强大和灵活也导致了非常容易产生意想不到的危险。即使是富有经验的程序员，也没有办法完全避免出现因为指针导致的安全问题。既然，这个问题光靠人脑的算力无法解决，就从规则的设计上去避免，这也是 Rust 这门语言所追求的，通过遵循一定的规则来实现性能和安全的平衡。

这篇文章描述了 Rust 中的各种指针，希望通过这篇文章能够让我们灵活运用指针，从害怕指针转向拥抱指针。

## 指针的历史演进 ##

指针（Pointer）最早作为编程语言中的一等公民出现在 C 语言和 Pascal 语言中，其本质是一个保存内存地址的变量。指针的强大之处在于可以直接操作内存，极大地提升了程序的灵活性和性能，例如在系统底层实现链表、树、图等复杂数据结构，或用于回调和内存映射等场景。然而，指针同时也是一把双刃剑——悬垂指针、野指针、缓冲区溢出、内存泄漏、数据竞争等问题层出不穷，可以说是许多安全漏洞的根源。

随着编程语言的发展，现代语言如 C++、Rust、Go 等都继承了指针的概念，但普遍对指针的使用做了限制，以减少甚至避免由此带来的安全隐患。

在 Java 中，虽然没有显式的指针类型，取而代之的是引用（Reference），开发者无法直接操作内存地址，这大大提升了安全性和稳定性。但 Java 程序员对著名的 NPE（NullPointerException，空指针异常）并不陌生，这是由于引用可以为 null，导致运行时异常。为缓解这一问题，Java 后续引入了 Optional 类型，鼓励开发者显式地进行判空处理，从而减少 NPE 的发生。

更进一步，Kotlin、Swift、Rust 等语言在类型系统层面引入了“可空类型”，从根本上减少或避免了空指针异常。例如，Kotlin 通过 String? 表示可空类型，Rust 则通过 Option<T> 明确区分有值和无值的情况。

## Rust 中的指针 ##

在 Rust 中，既继承了 C 语言中指针的强大之处，同时也通过对指针类型的进一步划分结合所有权机制，有效规避了传统指针带来的内存安全问题。

Rust 将指针划分成裸指针（Raw Pointer）、引用（Reference）、智能指针（Smart Pointer）：

* 裸指针：包括不可变形式（`*const T`）和可变形式（`*mut T`）。这类指针和 C/C++ 中地指针相似，但是不受所有权和借用规则保护，所以是不安全的，必须包含在 `unsafe` 代码块中使用。

* 引用：包含不可变引用（`&T`）和可变引用（`&mut T`），受到所有权和借用规则的保护，常用且非常安全。

* 智能指针：比如 `Box<T>`、`Rc<T>`、`Arc<T>`、`RefCell<T>`、`Pin<T>`、`Mutex<T>` 等，是对资源和所有权的再次封装，解决了在一些复杂场景下引用受所有权和借用规则的约束不够灵活的问题，适用于更复杂的内存管理场景。智能指针属于结构体类型，实现了 `Deref` 和 `DerefMut` trait。

此外，Rust 还有如函数指针、`NonNull<T>` 这类特殊的指针类型。在了解了 Rust 中指针的分类之后，我们将会继续深入每一种指针的用法和使用场景。

## 所有权回顾 ##

在 Rust 的编码中，最常见的也是最安全的就是引用类型的指针，这是因为是基于 Rust 的所有权和借用规则。所以，我们在讲具体的指针之前，需要先来回顾一下所有权和借用规则。

所有权指的是，变量的使用需要遵循下面三大规则:

* 所有值都有一个所有者（Owner）：程序中每一个值有且只有一个所有者变量；
* 同一时刻，只能有一个所有者：当所有权被转移（Move）之后，原来的所有者不再拥有该值，避免多方同时操作同一数据，防止数据竞争和双重释放；
* 所有者离开作用域时，值将被释放：资源的释放由作用域自动管理，避免悬垂指针和内存泄漏问题。

### 案例: 悬垂指针 ###

我们以悬垂指针为例，我通过 C 语言和 Rust 语言的对比来说明这些规则的设立目的，我们来看一下 C 语言中是如何产生这个问题的:
```c
#include <stdio.h>
#include <stdlib.h>

int* create_point() {
    int x = 67;
    return &x;
}

int main() {
    int* ptr = create_point();
    printf("%d\n", *ptr);
    return 0;
}
```
使用 GCC 编译，编译器可能会给出如下警告, 表明函数返回了一个本地变量的地址，会导致悬垂指针的出现：
```
$ gcc examples/dangling_pointers.c -o target/dangling_pointer
$ ./target/dangling_pointer
examples/dangling_pointers.c:6:12: warning: function returns address of local variable [-Wreturn-local-addr]
    6 |     return &x;
      |            ^~
```
同样的例子，Rust 根本不会编译通过。但是 GCC 仍然可以编译通过并运行，而且 GCC 也没办法找出所有的这类问题，只是因为上面我们的示例程序比较简单。
```rust
fn create_pointer() -> &i32 {
    let x = 42;
    &x

fn main() {
    let ptr = create_pointer();
    println!("{}", ptr);
}
```
这个例子将会编译不通过，Rust 会给出如下错误以及修复意见：：
```
help: consider using the `'static` lifetime, but this is uncommon unless you're returning a borrowed value from a `const` or a `static`
  |
1 | fn create_pointer() -> &'static i32 {
  |                         +++++++
help: instead, you are more likely to want to return an owned value
  |
1 - fn create_pointer() -> &i32 {
1 + fn create_pointer() -> i32 {
  |
```
要么加上生命周期的类型标注（只适用于返回外部传入的变量的引用，不适用于局部变量的引用），要么返回一个 `i32` 类型，实现了 Copy trait，所以不存在引用问题。相比 C 语言，Rust 在编译器就消灭了悬垂指针的问题。

### 案例：指针重复释放 ###

接着，我们在用 C 语言写一个指针重复释放的案例:
```c
#include <stdio.h>
#include <stdlib.h>

void release(int *p) {
    // 释放指针指向的内存空间
    free(p);
}

int main() {
    // 分配内存空间
    int *p = (int *)malloc(sizeof(int));
    release(p);
    release(p); // 重复释放
    return 0;
}
```
然后我们编译并执行这个程序:
```
$ gcc examples/double_free.c -o target/release_pointers
$ ./target/double_free 
free(): double free detected in tcache 2
Aborted (core dumped)
```
可以看到，编译器并不会在编译期间给出任何警告或者错误，当程序运行的时候，才给出错误指明存在重复释放的问题。这只是一个简单的例子，在实际的更复杂的场景中，出现错误的地方可能已经不是事故的第一案发现场，要想找到案发现场就会消耗很多的时间。

在 C++11 标准中，正式引入了智能指针的概念，来解决这类重复释放的问题。我们来看 C++ 的示例:
```c++
#include <memory>
#include <iostream>

void release(std::unique_ptr<int> p) {
    // 当 p 离开作用域的时候，自动释放所指向的内存
}

int main() {
    auto p = std::make_unique<int>(0);
    release(std::move(p));  // 第一次释放
    // 编译时不会出错误，p 已经为空，不会重复释放
    release(std::move(p));
    return 0;
}
```
然后编译运行，不会有任何问题:
```
$ g++ examples/smart_pointer.cpp -o target/smart_pointer -std=c++14    
$ ./target/smart_pointer
```

但是 C++ 的智能指针并没有完全解决内存泄漏、数据竞争、悬垂指针、重复释放等内存安全问题，只是极大的减少了这类问题的发生。所以，Rust 参考了 C++ 的智能指针，但是从语言层面做了更严格的规范。在编译 期做强制检查，避免了这方面的运行时错误，防止循环引用，区分线程安全和线程不安全，避免数据竞争，让程序在编译期就做到安全，更实现了零成本抽象。

### Rust 案例 ###

```rust
fn release(_p: Box<i32>) {}

fn main() {
    let ptr = Box::new(0);
    release(ptr);
    release(ptr);
}
```
这个程序无法编译通过，错误如下:
```
$ cargo check
    Checking app v0.1.0 (/home/user/emo)
error[E0382]: use of moved value: `ptr`
 --> src/main.rs:6:13
  |
4 |     let ptr = Box::new(0);
  |         --- move occurs because `ptr` has type `Box<i32>`, which does not implement the `Copy` trait
5 |     release(ptr);
  |             --- value moved here
6 |     release(ptr);
  |             ^^^ value used here after move
  |
note: consider changing this parameter type in function `release` to borrow instead if owning the value isn't necessary
 --> src/main.rs:1:16
  |
1 | fn release(_p: Box<i32>) {}
  |    -------     ^^^^^^^^ this parameter takes ownership of the value
  |    |
  |    in this function
help: consider cloning the value if the performance cost is acceptable
  |
5 |     release(ptr.clone());
  |                ++++++++

For more information about this error, try `rustc --explain E0382`.
```
这个错误表明引用的所有权已经转移，但是你可以通过调用 `clone()` 方法，增加引用计数的方式来传递多个引用。

### GC 语言存在的问题 ###

至于 Java、Go 等语言，则采用了另一种内存管理策略——垃圾回收（Garbage Collection, GC）。这种机制极大地简化了开发者的工作，使其无需手动管理内存的分配与释放，从而减少了因资源回收不当导致的内存泄漏和悬垂指针等问题。然而，GC 机制本身会带来一定的运行时开销。在 Java 中，垃圾回收器以独立线程的方式周期性地扫描和回收不再使用的对象，但具体的回收时机和耗时并不可控，可能导致应用在关键时刻发生停顿（如“Stop the World”），在高性能或低延迟场景下成为瓶颈。

相比之下，Rust 通过所有权和生命周期等静态检查机制，在编译期就完成了绝大多数资源管理相关的安全校验，避免了运行时垃圾回收的开销。相比 C/C++ 提升了开发效率，相比 Java/Go 提升了性能，做到了开发效率和性能的平衡，尤其适用于对资源控制和响应时间要求极高的系统级开发场景。

## 引用和借用 ##

上文中，我们通过对比 C 和 C++ 的程序，理解了所有权机制设立的目标就是为了避免各类内存安全的问题，比如悬垂指针、重复释放等。在这个基础上我们再来看什么是引用和借用。先看下面这个例子:
```rust
struct User;

fn song(_user: User) {}

fn dancing(_user: User) {}

fn main() {
    let user = User{};
    song(user);   // 所有权已经转移
    dancing(user);  // 编译错误
}
```
如果你希望一个用户既能唱歌，又能跳舞，这个程序是无法实现的。因为当他去唱歌的时候，所有权就已经转移了，`user` 变量已经失效。根据以往的编程经验，如果一个变量传递给一个方法之后就不能用了，那也太不方便了，甚至写不了程序。

如果你想让这个程序通过，你就需要通过引用传递:
```rust
struct User;

fn song(_user: &User) {}

fn dancing(_user: &User) {}

fn main() {
    let user = User{};
    song(&user);   // 所有权未转移，传递的是不可变引用
    dancing(&user);  // 同一时间，可以存在多个不可变引用
}
```
对于调用者来说，传递给函数的是 `&user`，即 `user` 的引用，对于 `song` 和 `dancing` 两个方法来说就是借用，借过来用一用。引用和借用其实是同一个意思，引用是一个名词，表示数据的访问方式；借用是一个动词，表示临时使用数据的行为。更严谨的说，引用（Reference）是语言层面的类型，借用（Borrowing）是行为和语义。

引用分为不可变引用和可变引用，借用的规则如下:

* 可以存在多个不可变引用
* 只能存在一个可变引用
* 不能同时存在可变引用和不可变引用，从而避免数据竞争和悬垂引用

Rust 编译器会自动检查引用的生命周期，防止引用失效导致悬垂指针。这三条规则的设立为内存和并发安全提供了基础，这是 Rust 保证内存安全和并发安全的核心机制之一。

## 智能指针 ##

看到这里，你应该已经了解了指针的发展脉络、所有权的意义，以及 Rust 中引用和借用的基本用法。在绝大多数场景下，引用和借用已经足够安全高效，但在一些更复杂的场景下，它们就显得不够灵活了。比如自引用的数据结构（如链表、树）、回调管理、状态统计、线程间资源共享等。

### 写一个链表 ###

有些教程会劝初学者不要使用 Rust 来编写数据结构比如树、图，因为在不了解所有权机制的情况下，编写这些包含自引用的数据结构是非常痛苦的，很难通过编译器这关。那么接下来我们就来写一个简单的链表吧，通过链表我们来讲解为什么需要智能指针，解决了什么问题，有哪些智能指针。

下面就是一个链表的例子:
```rust
struct Node {
    value: i32,
    next: Option<Node>,
}

struct List {
    head: Option<Node>,
}

impl List {
    fn new() -> Self {
        List { head: None }
    }

    fn push(&mut self, value: i32) {
        let node = Node {
            value,
            next: self.head,
        };
        self.head = Some(&node);
    }
}
```
如果这个例子放在其他语言中，并没有太大问题。但是 Rust 中，编译是不会通过的，我们来测试一下:
```rust
fn main() {
    let mut list = List::new();
    list.push(1);
    list.push(2);
}
```
我们编译之后出现如下错误:
```
$ cargo check
    Checking app v0.1.0 (/home/user/emo)
error[E0072]: recursive type `Node` has infinite size
 --> src/main.rs:1:1
  |
1 | struct Node {
  | ^^^^^^^^^^^
2 |     value: i32,
3 |     next: Option<Node>,
  |                  ---- recursive without indirection
  |
help: insert some indirection (e.g., a `Box`, `Rc`, or `&`) to break the cycle
  |
3 |     next: Option<Box<Node>>,
  |                  ++++    +

```
Rust 要求所有的类型都能够在编译期间确定大小（Sized），而我们写的 `next: Option<Node>`，Node 又包含 Node，形成无限递归，就无法确定大小。所以你看到了这个错误，但是 Rust 编译器也很智能地给出了解决方案，将 Node 装入 Box。

Box 的作用是将 Node 放到堆上，通过指针来引用。那么结构体里只存指针的大小，就可以通过编译:
```rust
struct Node {
    value: i32,
    next: Option<Box<Node>>,
}

struct List {
    head: Option<Box<Node>>,
}

impl List {
    fn new() -> Self {
        List { head: None }
    }

    fn push(&mut self, value: i32) {
        let node = Node {
            value,
            next: self.head.take(), // take 是将原来的 head 置空并取出旧值
        };
        self.head = Some(Box::new(node));
    }
}
```
这看上去像是一个魔法，用 Box 将 Node 包装成一个智能指针，然后它就从原本栈帧上的一个局部变量转移到了堆上，栈上只保留一个指针。具体是怎么实现的呢？我们来分析一下 Box 的源码。

### Box 源码分析 ###

李白说，欲穷千里目，更上一层楼。但对于编程而言，想要看到更多，可能要更下一层去查看源码。Box 的结构如下:
```rust
pub struct Box<
    T: ?Sized,
    A: Allocator = Global,
>(Unique<T>, A);
```
Box 包含两个成员：

* `Unique<T>` 就是一个非空指针，等价与 `NonNull<T>`；
* `A` 是内存分配器，默认情况下就是 `malloc/free`；

透过 Bootstrap 模式（编译器在没有完成标准库编译前，使用 Rust 自举的模式），`Box::new(val)`时做的事情如下:

```rust
pub fn try_new_uninit_in(alloc: A) -> Result<Box<mem::MaybeUninit<T>, A>, AllocError>
where
  A: Allocator,
{
  let ptr = if T::IS_ZST {
    NonNull::dangling()
  } else {
    let layout = Layout::new::<mem::MaybeUninit<T>>();
    alloc.allocate(layout)?.cast()
  };
  unsafe { Ok(Box::from_raw_in(ptr.as_ptr(), alloc)) }
}
```
大致流程如下:

1. 计算 `Layout::new::<T>()`，根据 T 的内存布局信息来计算出将要分配的内存大小以及对齐
2. 用分配器 `A::allocate(layout)` 申请一块足够大的堆内存
3. 将返回的原始指针 `ptr: *mut T` 包转成 `Unique<T>`
4. 把值写到之前申请的内存中
5. 返回 `Box(Unique(ptr), alloc)`

通过 Box 源码，我们发现 Box 将类型从栈上转移到了堆上，这个数据的所有者是 Box。而 Box 的大小在编译时是可以确定的，就是一个指针的大小（`NonNull<Node<T>>`），从而符合 Rust 关于在编译时需要固定大小（Sized）的要求。

> 你可能有疑问，为什么在要编译时明确数据的大小？这是因为 Rust 在编译的时候需要为每个函数确定固定的栈帧的大小，从而高效分配局部变量。

所以，智能指针的一个作用就是管理堆上的数据，因为其实现了自动析构（Drop），所以在作用域结束的时候，会清理资源，从而避免了悬垂指针、多次释放等内存问题。

另外智能指针也实现了 `Deref`、`DerefMut`，可以像操作普通引用那样使用 `*box` 或者 `box.field`。

其他的智能指针，例如 `Rc<T>`、`Mutex<T>`、`RefCell<T>` 也都是相同的道理。

## 总结 ##

Rust 的指针体系看起来复杂，其实是从众多其他语言的发展中遇到的问题，演进而来。结合了所有权和借用规则，通过智能指针在栈上分配资源并自动析构，从而避免了很多悬垂指针、多次释放、内存泄漏、数据竞争等问题。如果从这些问题的发生着眼，去思考这些规则的设立目的，就会好理解很多。