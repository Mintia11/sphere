#pragma once

#include <stdatomic.h>

typedef struct {
    atomic_uint_fast32_t next_ticket;
    atomic_uint_fast32_t now_serving;
} etna_mutex_t;

#define ETNA_MUTEX_INIT (etna_mutex_t){.next_ticket = 0, .now_serving = 0}

static inline void etna_mutex_lock(etna_mutex_t* m) {
    uint_fast32_t my_ticket = atomic_fetch_add_explicit(&m->next_ticket, 1, memory_order_relaxed);

    for (;;) {
        uint_fast32_t serving = atomic_load_explicit(&m->now_serving, memory_order_acquire);
        if (serving == my_ticket) {
            return;
        }
    }
}

static inline void etna_mutex_unlock(etna_mutex_t* m) {
    uint_fast32_t current = atomic_load_explicit(&m->now_serving, memory_order_relaxed);
    atomic_store_explicit(&m->now_serving, current + 1, memory_order_release);
}