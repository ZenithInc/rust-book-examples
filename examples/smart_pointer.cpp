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