#pragma once

#include <stdint.h>
#include <alloc.h>

#define ETNA_VEC(type)     \
    struct {               \
        type* data;        \
        uint32_t capacity; \
        uint32_t length;   \
    }
#define ETNA_VEC_INIT {.data = NULL, .capacity = 0, .length = 0}

#define ETNA_VEC_FREE(vec)      \
    do {                        \
        ETNA_FREE((vec)->data); \
        (vec)->data = NULL;     \
        (vec)->capacity = 0;    \
        (vec)->length = 0;      \
    } while (0);

#define ETNA_VEC_AT(vec, idx) (vec)->data[idx]

#define ETNA_VEC_REMOVE(vec, idx)                                          \
    do {                                                                   \
        if ((idx) < (vec)->length) {                                       \
            memmove(&(vec)->data[idx], &(vec)->data[(idx) + 1],            \
                    ((vec)->length - (idx) - 1) * sizeof(*((vec)->data))); \
            (vec)->length -= 1;                                            \
        }                                                                  \
    } while (0);

#define ETNA_VEC_RESIZE(vec, new_size)                                                             \
    do {                                                                                           \
        if ((vec)->capacity != new_size) {                                                         \
            if ((vec)->capacity == 0) {                                                            \
                (vec)->capacity = new_size;                                                        \
                (vec)->data = ETNA_ALLOC(NULL, (vec)->capacity * sizeof(*((vec)->data)));          \
            } else if ((vec)->length == (vec)->capacity) {                                         \
                (vec)->capacity = new_size;                                                        \
                (vec)->data = ETNA_REALLOC((vec)->data, (vec)->capacity * sizeof(*((vec)->data))); \
            }                                                                                      \
        }                                                                                          \
    } while (0);

#define ETNA_VEC_PUSH(vec, value)                                              \
    do {                                                                       \
        ETNA_VEC_RESIZE((vec), (vec)->capacity != 0 ? (vec)->capacity * 2 : 4) \
        (vec)->data[(vec)->length] = value;                                    \
        (vec)->length += 1;                                                    \
    } while (0);

#define ETNA_VEC_POP(vec, out_val)                   \
    do {                                             \
        if ((vec)->length > 0) {                     \
            (vec)->length -= 1;                      \
            *(out_val) = (vec)->data[(vec)->length]; \
        }                                            \
    } while (0);

#define ETNA_VEC_FOR_EACH_ENTRY(vec, idx) for (uint32_t idx = 0; idx < (vec)->length; idx++)
