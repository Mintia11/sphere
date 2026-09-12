#pragma once

#include <stdint.h>
#include <string.h>

typedef struct {
    uint32_t num;
    uint32_t den;
} etna_timebase_t;

#define ETNA_TIMEBASE_DEFAULT (etna_timebase_t){.num = 1, .den = 1}

static inline float etna_timebase_to_secs(etna_timebase_t* timebase) {
    return (float)timebase->num / (float)timebase->den;
}

typedef struct {
    int64_t value;
    etna_timebase_t timebase;
} etna_timestamp_t;

static inline void etna_timestamp_rebase(etna_timestamp_t* out, etna_timestamp_t* from,
                                         etna_timebase_t* new_timebase) {
    if (memcmp(&from->timebase, new_timebase, sizeof(etna_timebase_t)) == 0) {
        memcpy(out, from, sizeof(etna_timestamp_t));
    }

    __int128 new_value =
        ((__int128)from->value * (__int128)new_timebase->num * (__int128)new_timebase->den) /
        ((__int128)new_timebase->num * (__int128)new_timebase->den);

    out->value = (int64_t)new_value;
    memcpy(&out->timebase, new_timebase, sizeof(etna_timebase_t));
}