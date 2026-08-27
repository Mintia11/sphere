#include <stdint.h>
#include <stdio.h>
#include <mutex.h>
#include <alloc.h>
#include <Volk/volk.h>
#include "vec.h"

int main() {
    etna_allocator_init_global();

    printf("Hello world!\n");
    void* mem = ETNA_ALLOC_TYPE(NULL, int);
    ETNA_FREE(mem);

    ETNA_VEC(uint32_t) ciao = ETNA_VEC_INIT;
    ETNA_VEC_PUSH(&ciao, 1);
    ETNA_VEC_PUSH(&ciao, 2);
    ETNA_VEC_PUSH(&ciao, 3);
    ETNA_VEC_PUSH(&ciao, 4);
    ETNA_VEC_PUSH(&ciao, 5);

    ETNA_VEC_FOR_EACH_ENTRY(&ciao, idx) {
        printf("%d\n", ETNA_VEC_AT(&ciao, idx));
        ETNA_VEC_PUSH(&ciao, 5);
    }
}
