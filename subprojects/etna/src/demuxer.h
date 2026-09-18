#pragma once

#include <stdint.h>
#include <stdio.h>
#include <stdbool.h>
#include "time.h"
#include "packet.h"

typedef enum {
    ETNA_TRACK_UNK,
    ETNA_TRACK_VIDEO,
    ETNA_TRACK_AUDIO,
    ETNA_TRACK_SUBTITLE
} etna_track_type;
typedef enum { ETNA_CODEC_UNK, ETNA_CODEC_H264 } etna_track_codec;

typedef struct {
    uint32_t id;
    etna_track_type type;
    etna_track_codec codec;
    etna_timebase_t timebase;

    uint8_t* codec_data;
    size_t codec_data_len;
} etna_track_t;

typedef struct etna_demuxer {
    void* priv;

    void (*get_tracks)(struct etna_demuxer* demuxer, etna_track_t* tracks, size_t* track_count);
    int (*read_packet)(struct etna_demuxer* demuxer, etna_packet_t* out);
    etna_timestamp_t (*duration)(struct etna_demuxer* demuxer);
} etna_demuxer_t;

typedef struct {
    bool (*is_supported)(FILE* file);
    etna_demuxer_t* (*create)(FILE* file);
    void (*destroy)(etna_demuxer_t* demuxer);
} etna_demuxer_def_t;

static inline etna_demuxer_t* etna_get_demuxer(FILE* file, const etna_demuxer_def_t** demuxers,
                                               const size_t demuxer_count) {
    for (size_t i = 0; i < demuxer_count; i++) {
        const etna_demuxer_def_t* demuxer = demuxers[i];
        if (demuxer->is_supported(file)) {
            return demuxer->create(file);
        }
    }

    return NULL;
}