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