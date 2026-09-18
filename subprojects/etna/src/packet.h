#pragma once

#include <stdint.h>
#include <stdbool.h>
#include "time.h"

typedef struct {
    uint32_t track;
    etna_timestamp_t pts;
    etna_timestamp_t dts;

    uint8_t* data;
    size_t data_len;

    bool is_keyframe;
    bool is_discard;
    bool is_corrupt;
} etna_packet_t;