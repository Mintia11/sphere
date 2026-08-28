#include <stdint.h>
#include <mutex.h>
#include <alloc.h>
#include <Volk/volk.h>
#include "log.h"
#include "vec.h"

int main() {
    etna_allocator_init_global();
    etna_logger_init_global();

    etna_log_scope_t* main_log = etna_log_scope_new("main", NULL);

    ETNA_INFO(main_log, "Hello world!\n");
    void* mem = ETNA_ALLOC_TYPE(NULL, int);
    ETNA_FREE(mem);

    ETNA_VEC(uint32_t) ciao = ETNA_VEC_INIT;
    ETNA_VEC_PUSH(&ciao, 1);
    ETNA_VEC_PUSH(&ciao, 2);
    ETNA_VEC_PUSH(&ciao, 3);
    ETNA_VEC_PUSH(&ciao, 4);
    ETNA_VEC_PUSH(&ciao, 5);

    etna_log_scope_t* loop_log = etna_log_scope_new("loop", main_log);

    ETNA_VEC_FOR_EACH_ENTRY(&ciao, idx) {
        ETNA_INFO(loop_log, "%d\n", ETNA_VEC_AT(&ciao, idx));
    }

    ETNA_FREE(main);
}
