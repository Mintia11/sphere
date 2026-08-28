#pragma once

#include <mutex.h>
#include <linked_list.h>
#include <stdatomic.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include "log.h"

struct etna_allocator;

extern struct etna_allocator* etna_global_alloc;

typedef struct etna_alloc {
    struct etna_alloc* parent;
    atomic_uint_fast32_t refcount;
    char sentinel[3];
    bool is_large;
    uint8_t data[];
} etna_alloc_t;

typedef struct etna_alloc_bucket_header {
    size_t size;
    union {
        void** first_free;
        struct etna_alloc_bucket_header* parent;
    };
    struct etna_allocator* allocator;
} etna_alloc_bucket_header_t;

typedef struct etna_allocator {
    etna_mutex_t mtx;
    etna_log_scope_t* scope;
    size_t bucket_count;
    size_t* bucket_sizes;
    etna_alloc_bucket_header_t* buckets[];
} etna_allocator_t;

#define ETNA_MIN_ALIGN 16
#define ETNA_ALLOC_SENTINEL_VALUE "etn"
#define ETNA_FREE_SENTINEL_VALUE "fre"

#define ETNA_ALIGN_UP(value, align) (((value) + (align) - 1) & ~((align) - 1))
#define ETNA_ALLOCATION_GET(data) (etna_alloc_t*)(data - (void*)ETNA_MIN_ALIGN)
#define ETNA_ALLOCATION_GET_BUCKET(data) (etna_alloc_bucket_header_t*)((uintptr_t)data & ~0xFFFF)

#define ETNA_ALLOC(parent, size) etna_allocator_allocate(etna_global_alloc, parent, size)
#define ETNA_ALLOC_TYPE(parent, type) \
    etna_allocator_allocate(etna_global_alloc, parent, sizeof(type))
#define ETNA_CALLOC_TYPE(parent, type, count) \
    etna_allocator_allocate(etna_global_alloc, parent, sizeof(type) * (count))
#define ETNA_REALLOC(data, new_size) etna_allocator_realloc(etna_global_alloc, data, new_size)
#define ETNA_FREE(data) etna_allocator_free(etna_global_alloc, data)

void etna_allocator_init_global();
etna_allocator_t* etna_allocator_new(const size_t* bucket_sizes, const size_t bucket_count);
void etna_allocator_init(etna_allocator_t* alloc, const size_t* bucket_sizes,
                         const size_t bucket_count);
void* etna_allocator_allocate(etna_allocator_t* alloc, const void* parent, size_t size);
void* etna_allocator_realloc(etna_allocator_t* alloc, void* data, const size_t new_size);
int etna_allocator_free(etna_allocator_t* alloc, void* data);