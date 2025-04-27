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