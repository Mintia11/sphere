#include "alloc.h"
#include <alloc.h>
#include <Windows.h>
#include <mutex.h>
#include <stdatomic.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

const size_t etna_default_bucket_sizes[] = {32, 64, 128, 256, 512, 1024, 2048};
struct etna_allocator* etna_global_alloc = 0;

void etna_allocator_init_global() {
    etna_allocator_t* alloc =
        etna_allocator_new(etna_default_bucket_sizes, sizeof(etna_default_bucket_sizes) /
                                                          sizeof(etna_default_bucket_sizes[0]));
    etna_global_alloc = alloc;
}

etna_allocator_t* etna_allocator_new(const size_t* bucket_sizes, const size_t bucket_count) {
    size_t allocator_size =
        sizeof(etna_allocator_t) + (bucket_count * sizeof(etna_alloc_bucket_header_t*));
    etna_allocator_t* allocator = (etna_allocator_t*)malloc(allocator_size);
    memset(allocator, 0, allocator_size);

    etna_allocator_init(allocator, bucket_sizes, bucket_count);
    return allocator;
}

void etna_allocator_init(etna_allocator_t* alloc, const size_t* bucket_sizes,
                         const size_t bucket_count) {
    alloc->mtx = ETNA_MUTEX_INIT;

    size_t* bucket_sizes_alloc = (size_t*)malloc(bucket_count * sizeof(size_t));
    memcpy(bucket_sizes_alloc, bucket_sizes, bucket_count * sizeof(size_t));

    alloc->bucket_sizes = bucket_sizes_alloc;
    alloc->bucket_count = bucket_count;
}

static void init_bucket(etna_allocator_t* alloc, const size_t bucket_idx, const size_t size,
                        etna_alloc_bucket_header_t* link_into) {
    void* mem = VirtualAlloc(NULL, 64 * 1024, MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE);
    if (!mem) {
        printf("fatal: failed to allocate memory for bucket id: %lld of size %lld\n", bucket_idx,
               size);
        exit(1);
    }

    etna_alloc_bucket_header_t* hdr = (etna_alloc_bucket_header_t*)mem;
    hdr->allocator = alloc;
    hdr->size = size;

    size_t entry_count = ((64 * 1024) / size);
    size_t hdr_size = ETNA_ALIGN_UP(sizeof(etna_alloc_bucket_header_t), size);

    void** last_entry = NULL;
    if (link_into) {
        last_entry = link_into->first_free;
    }

    for (size_t i = (hdr_size / size); i < entry_count; i++) {
        void** entry = (void**)(mem + (size * i));
        *entry = last_entry;
        last_entry = entry;
    }

    if (link_into) {
        link_into->first_free = last_entry;
        hdr->parent = link_into;
    } else {
        hdr->first_free = last_entry;
        alloc->buckets[bucket_idx] = hdr;
    }
}

void* etna_allocator_allocate(etna_allocator_t* alloc, const void* parent, size_t size) {
    etna_alloc_t* parent_alloc = NULL;
    if (parent) {
        parent_alloc = ETNA_ALLOCATION_GET(parent);
        if (parent_alloc->sentinel != ETNA_ALLOC_SENTINEL_VALUE) {
            printf("fatal: tried to link an allocation of size %lld to a corrupt allocation at %p",
                   size, parent);
            exit(1);
        }
    }

    size = ETNA_ALIGN_UP(size + sizeof(etna_alloc_t), ETNA_MIN_ALIGN);

    etna_mutex_lock(&alloc->mtx);

    int bucket_idx = -1;
    for (size_t i = 0; i < alloc->bucket_count; i++) {
        if (alloc->bucket_sizes[i] >= size) {
            bucket_idx = i;
            break;
        }
    }

    if (bucket_idx == -1) {
        printf("todo: implement large allocations\n");
        exit(1);
    }

    if (!alloc->buckets[bucket_idx]) {
        init_bucket(alloc, bucket_idx, alloc->bucket_sizes[bucket_idx], NULL);
    }

    if (!alloc->buckets[bucket_idx]->first_free) {
        init_bucket(alloc, bucket_idx, alloc->bucket_sizes[bucket_idx], alloc->buckets[bucket_idx]);
    }

    etna_alloc_t* our_allocation = (etna_alloc_t*)alloc->buckets[bucket_idx]->first_free;
    alloc->buckets[bucket_idx]->first_free = *alloc->buckets[bucket_idx]->first_free;

    our_allocation->parent = parent_alloc;
    our_allocation->refcount = 1;
    our_allocation->sentinel = ETNA_ALLOC_SENTINEL_VALUE;

    if (parent_alloc) {
        atomic_fetch_add_explicit(&parent_alloc->refcount, 1, memory_order_acquire);
    }

    etna_mutex_unlock(&alloc->mtx);

    return &our_allocation->data;
}

int etna_allocator_free(etna_allocator_t* alloc, void* data) {
    etna_alloc_t* allocation = ETNA_ALLOCATION_GET(data);

    if (atomic_load_explicit(&allocation->refcount, memory_order_seq_cst) != 1) {
        return atomic_load_explicit(&allocation->refcount, memory_order_seq_cst);
    }

    if (allocation->sentinel == ETNA_FREE_SENTINEL_VALUE) {
        printf("fatal: tried to free allocation at %p twice", data);
        exit(1);
    } else if (allocation->sentinel != ETNA_ALLOC_SENTINEL_VALUE) {
        printf("fatal: tried to free an allocation at %p not tracked by this allocator", data);
        exit(1);
    }

    etna_mutex_lock(&alloc->mtx);
    if (allocation->parent) {
        atomic_fetch_sub_explicit(&allocation->parent->refcount, 1, memory_order_release);
    }

    allocation->sentinel = ETNA_FREE_SENTINEL_VALUE;

    etna_alloc_bucket_header_t* bucket = ETNA_ALLOCATION_GET_BUCKET(data);
    while ((bucket->first_free != 0) && ((uintptr_t)bucket->first_free & ~0xFFFF) == 0)
        bucket = bucket->parent;

    *(void**)allocation = bucket->first_free;
    bucket->first_free = (void**)allocation;

    etna_mutex_unlock(&alloc->mtx);
    return 0;
}