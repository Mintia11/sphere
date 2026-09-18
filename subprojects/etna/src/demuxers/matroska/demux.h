#pragma once

#include "demuxer.h"
#include "vec.h"

typedef struct {
    uint32_t id;
    uint64_t position;
} matroska_seek_t;

typedef struct {
    uint64_t timestamp_scale;
    char* muxing_app;
    char* writing_app;
    char segment_uuid[16];
    double duration;
} matroska_info_t;

typedef struct {
    size_t data_start;
    size_t data_length;
} matroska_block_data_t;

typedef struct {
    uint32_t track_id;
    int16_t relative_timestamp;
    ETNA_VEC(matroska_block_data_t) data;
} matroska_block_t;

typedef struct {
    uint64_t timestamp;
    ETNA_VEC(matroska_block_t) blocks;
} matroska_cluster_t;

typedef struct {
    ETNA_VEC(matroska_seek_t) seek_head;
    matroska_info_t info;
    ETNA_VEC(etna_track_t) tracks;
    ETNA_VEC(matroska_cluster_t) clusters;
} matroska_segment_t;

typedef struct {
    FILE* file;
    matroska_segment_t segment;

    int64_t cluster_index;
    etna_timestamp_t current_cluster_timestamp;
    ETNA_VEC(etna_packet_t) pending_blocks;
} matroska_demuxer_priv_t;

extern etna_demuxer_def_t matroska_demuxer;