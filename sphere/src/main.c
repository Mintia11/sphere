#include <stdio.h>
#include <mutex.h>
#include <alloc.h>

int main() {
    etna_allocator_init_global();

    printf("Hello world!\n");
    void* mem = ETNA_ALLOC_TYPE(main, int);
}
