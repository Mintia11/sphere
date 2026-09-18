#include "demux.h"
#include <string.h>
#include "alloc.h"
#include "demuxer.h"
#include "ebml.h"
#include "vec.h"

#define CMP(a, b) strncmp(a, b, strlen(b)) == 0

static void matroska_read_seek(void* master_element, FILE* file, ebml_element_t* elem) {
    matroska_seek_t* out = (matroska_seek_t*)master_element;

    enum {
        MATROSKA_ID_SEEK_ID = 0x53AB,
        MATROSKA_ID_SEEK_POSITION = 0x53AC,
    };

    size_t length = ebml_element_length(elem);

    switch (elem->id) {
        case MATROSKA_ID_SEEK_ID:
            // the matroska spec says that this element is a binary EBML ID but i interpret EBML IDs as uint32_t
            out->id = ebml_read_uinteger(file, length);
            break;
        case MATROSKA_ID_SEEK_POSITION:
            out->position = ebml_read_uinteger(file, length);
            break;
        default:
            ETNA_ERROR(NULL, "unknown Matroska seek element: 0x%x\n", elem->id);
            break;
    }
}

static void matroska_read_info(void* master_element, FILE* file, ebml_element_t* elem) {
    matroska_info_t* out = (matroska_info_t*)master_element;

    enum {
        MATROSKA_ID_INFO_TIMESTAMP_SCALE = 0x2AD7B1,
        MATROSKA_ID_INFO_MUXING_APP = 0x4D80,
        MATROSKA_ID_INFO_WRITING_APP = 0x5741,
        MATROSKA_ID_INFO_SEGMENT_UUID = 0x73A4,
        MATROSKA_ID_INFO_DURATION = 0x4489,
    };

    size_t length = ebml_element_length(elem);

    switch (elem->id) {
        case MATROSKA_ID_INFO_TIMESTAMP_SCALE:
            out->timestamp_scale = ebml_read_uinteger(file, length);
            break;
        case MATROSKA_ID_INFO_MUXING_APP:
            out->muxing_app = ebml_read_string(file, length);
            break;
        case MATROSKA_ID_INFO_WRITING_APP:
            out->writing_app = ebml_read_string(file, length);
            break;
        case MATROSKA_ID_INFO_SEGMENT_UUID:
            uint8_t* data = ebml_read_binary(file, length);
            memcpy(&out->segment_uuid, data, sizeof(out->segment_uuid));
            break;
        case MATROSKA_ID_INFO_DURATION:
            out->duration = ebml_read_float(file, length);
            break;
        default:
            ETNA_WARN(NULL, "unknown Matroska info element: 0x%x\n", elem->id);
            break;
    }
}

static void matroska_read_track_entry(void* master_element, FILE* file, ebml_element_t* elem) {
    etna_track_t* out = (etna_track_t*)master_element;

    enum {
        MATROSKA_ID_TRACK_NUMBER = 0xD7,
        MATROSKA_ID_TRACK_TYPE = 0x83,
        MATROSKA_ID_TRACK_CODEC_ID = 0x86,
        MATROSKA_ID_TRACK_CODEC_PRIVATE = 0x63A2,
    };

    enum {
        MATROSKA_TRACK_TYPE_VIDEO = 1,
        MATROSKA_TRACK_TYPE_AUDIO = 2,
        MATROSKA_TRACK_TYPE_SUBTITLE = 17,
    };

    size_t length = ebml_element_length(elem);

    switch (elem->id) {
        case MATROSKA_ID_TRACK_NUMBER:
            out->id = ebml_read_uinteger(file, length);
            break;
        case MATROSKA_ID_TRACK_TYPE:
            uint64_t track_type = ebml_read_uinteger(file, length);
            switch (track_type) {
                case MATROSKA_TRACK_TYPE_VIDEO:
                    out->type = ETNA_TRACK_VIDEO;
                    break;
                case MATROSKA_TRACK_TYPE_AUDIO:
                    out->type = ETNA_TRACK_AUDIO;
                    break;
                case MATROSKA_TRACK_TYPE_SUBTITLE:
                    out->type = ETNA_TRACK_SUBTITLE;
                    break;
                default:
                    ETNA_WARN(NULL, "unknown Matroska track type: %lld\n", track_type);
                    break;
            }

            break;
        case MATROSKA_ID_TRACK_CODEC_ID:
            char* name = ebml_read_string(file, length);

            if (CMP(name, "V_MPEG4/ISO/AVC")) {
                out->codec = ETNA_CODEC_H264;
            }

            ETNA_FREE(name);
            break;
        case MATROSKA_ID_TRACK_CODEC_PRIVATE:
            uint8_t* data = ebml_read_binary(file, length);
            out->codec_data = data;
            out->codec_data_len = length;
            break;
        default:
            ETNA_WARN(NULL, "unknown Matroska track element: 0x%x\n", elem->id);
            break;
    }
}

static void matroska_read_cluster(void* master_element, FILE* file, ebml_element_t* elem) {
    matroska_cluster_t* out = (matroska_cluster_t*)master_element;

    enum {
        MATROSKA_ID_CLUSTER_TIMESTAMP = 0xE7,
        MATROSKA_ID_CLUSTER_SIMPLE_BLOCK = 0xA3,
    };

    size_t length = ebml_element_length(elem);

    switch (elem->id) {
        case MATROSKA_ID_CLUSTER_TIMESTAMP:
            out->timestamp = ebml_read_uinteger(file, length);
            break;
        case MATROSKA_ID_CLUSTER_SIMPLE_BLOCK:
            matroska_block_t block = {0};
            block.track_id = ebml_read_vint(file);
            uint16_t raw = 0;
            if (fread(&raw, 2, 1, file) != 1) {
                ETNA_FATAL(NULL, "failed to read Matroska block relative timestamp");
                exit(1);
            }
            block.relative_timestamp = (int16_t)__builtin_bswap16(raw);

            uint8_t flags = 0;
            if (fread(&flags, 1, 1, file) != 1) {
                ETNA_FATAL(NULL, "failed to read Matroska block flags");
                exit(1);
            }

            if (flags & 0b110) {
                ETNA_FATAL(NULL, "TODO: block lacing");
                exit(1);
            } else {
                matroska_block_data_t data = {0};
                data.data_start = _ftelli64(file);
                data.data_length = elem->data_end - data.data_start;
                ETNA_VEC_PUSH(&block.data, data);
            }

            ETNA_VEC_PUSH(&out->blocks, block);

            break;
        default:
            ETNA_WARN(NULL, "unknown Matroska cluster element: 0x%x\n", elem->id);
            break;
    }
}

static void matroska_read_segment_elem(void* master_element, FILE* file, ebml_element_t* elem) {
    matroska_segment_t* out = (matroska_segment_t*)master_element;

    enum {
        MATROSKA_ID_SEEK_HEAD = 0x114D9B74,
        MATROSKA_ID_SEEK = 0x4DBB,
        MATROSKA_ID_INFO = 0x1549A966,
        MATROSKA_ID_TRACKS = 0x1654AE6B,
        MATROSKA_ID_TRACK_ENTRY = 0xAE,
        MATROSKA_ID_TAGS = 0x1254C367,
        MATROSKA_ID_CLUSTER = 0x1F43B675,
        MATROSKA_ID_CUES = 0x1C53BB6B,
    };

    switch (elem->id) {
        case MATROSKA_ID_SEEK_HEAD:
            ebml_read_master_element(file, out, elem->id, matroska_read_segment_elem, elem);
            break;
        case MATROSKA_ID_SEEK:
            matroska_seek_t seek = {0};
            ebml_read_master_element(file, &seek, elem->id, matroska_read_seek, elem);
            ETNA_VEC_PUSH(&out->seek_head, seek);
            break;
        case MATROSKA_ID_INFO:
            ebml_read_master_element(file, &out->info, elem->id, matroska_read_info, elem);
            break;
        case MATROSKA_ID_TRACKS:
            ebml_read_master_element(file, out, elem->id, matroska_read_segment_elem, elem);
            break;
        case MATROSKA_ID_TRACK_ENTRY:
            etna_track_t track = {0};
            ebml_read_master_element(file, &track, elem->id, matroska_read_track_entry, elem);
            track.timebase.num = 1;
            track.timebase.den = out->info.timestamp_scale;
            ETNA_VEC_PUSH(&out->tracks, track);
            break;
        case MATROSKA_ID_CLUSTER:
            matroska_cluster_t cluster = {0};
            ebml_read_master_element(file, &cluster, elem->id, matroska_read_cluster, elem);
            ETNA_VEC_PUSH(&out->clusters, cluster);
            break;
        default:
            ETNA_WARN(NULL, "unknown Matroska segment element: 0x%x\n", elem->id);
            break;
    }
}

static matroska_segment_t matroska_read_segment(FILE* file) {
    matroska_segment_t out = {0};
    ebml_read_master_element(file, &out, 0x18538067, matroska_read_segment_elem, NULL);
    return out;
}

void get_tracks(struct etna_demuxer* demuxer, etna_track_t* tracks, size_t* track_count) {
    matroska_demuxer_priv_t* priv = (matroska_demuxer_priv_t*)demuxer->priv;
    if (tracks && *track_count > 0) {
        memcpy(tracks, priv->segment.tracks.data, sizeof(etna_track_t) * *track_count);
    }

    *track_count = priv->segment.tracks.length;
}

int read_packet(struct etna_demuxer* demuxer, etna_packet_t* out) {
    matroska_demuxer_priv_t* priv = (matroska_demuxer_priv_t*)demuxer->priv;

    for (;;) {
        if (priv->pending_blocks.length > 0) {
            etna_packet_t packet = ETNA_VEC_AT(&priv->pending_blocks, 0);
            ETNA_VEC_REMOVE(&priv->pending_blocks, 0);
            memcpy(out, &packet, sizeof(etna_packet_t));
            return 0;
        }

        if (priv->cluster_index == -1) {
            ETNA_VEC_FOR_EACH_ENTRY(&priv->segment.clusters, idx) {
                matroska_cluster_t* cluster = &ETNA_VEC_AT(&priv->segment.clusters, idx);
                if (cluster->timestamp >= (uint64_t)priv->current_cluster_timestamp.value) {
                    priv->cluster_index = idx;
                    break;
                }
            }
        }

        if (priv->cluster_index == -1 ||
            priv->cluster_index >= (int64_t)priv->segment.clusters.length) {
            return 1;
        }

        matroska_cluster_t* cluster = &ETNA_VEC_AT(&priv->segment.clusters, priv->cluster_index);
        priv->current_cluster_timestamp.value = cluster->timestamp;

        ETNA_VEC_FOR_EACH_ENTRY(&cluster->blocks, idx) {
            matroska_block_t* block = &ETNA_VEC_AT(&cluster->blocks, idx);
            etna_packet_t packet_template = {0};
            packet_template.track = block->track_id;
            packet_template.pts = packet_template.dts = (etna_timestamp_t){
                .value = cluster->timestamp + block->relative_timestamp,
                .timebase = (etna_timebase_t){.num = 1, .den = priv->segment.info.timestamp_scale}};

            ETNA_VEC_FOR_EACH_ENTRY(&block->data, data_idx) {
                matroska_block_data_t* data = &ETNA_VEC_AT(&block->data, data_idx);
                etna_packet_t packet = packet_template;

                packet.data = ETNA_ALLOC(NULL, data->data_length);
                packet.data_len = data->data_length;
                if (fseek(priv->file, data->data_start, SEEK_SET) != 0) {
                    ETNA_FATAL(NULL, "failed to seek to Matroska block data");
                    exit(1);
                }
                if (fread(packet.data, 1, data->data_length, priv->file) != data->data_length) {
                    ETNA_FATAL(NULL, "failed to read Matroska block data");
                    exit(1);
                }
                ETNA_VEC_PUSH(&priv->pending_blocks, packet);
            }
        }

        if (priv->pending_blocks.length == 0) {
            return 1;
        }
    }
}

etna_timestamp_t duration(struct etna_demuxer* demuxer) {
    matroska_demuxer_priv_t* priv = (matroska_demuxer_priv_t*)demuxer->priv;
    return (etna_timestamp_t){
        .value = priv->segment.info.duration,
        .timebase = (etna_timebase_t){.num = 1, .den = priv->segment.info.timestamp_scale},
    };
}

bool is_supported(FILE* file) {
    ebml_header_t hdr = ebml_read_header(file);
    fseek(file, 0, SEEK_SET);

    return CMP(hdr.doc_type, "matroska");
}

etna_demuxer_t* create(FILE* file) {
    etna_demuxer_t* demuxer = ETNA_ALLOC_TYPE(NULL, etna_demuxer_t);
    matroska_demuxer_priv_t* priv = ETNA_ALLOC_TYPE(demuxer, matroska_demuxer_priv_t);

    demuxer->get_tracks = get_tracks;
    demuxer->read_packet = read_packet;
    demuxer->duration = duration;

    priv->file = file;
    demuxer->priv = priv;
    priv->cluster_index = -1;

    ebml_header_t hdr = ebml_read_header(file);
    if (strncmp(hdr.doc_type, "matroska", strlen("matroska")) != 0) {
        return NULL;
    }

    priv->segment = matroska_read_segment(file);

    ETNA_INFO(NULL, "parsed matroska file:\n");
    ETNA_INFO(NULL, "  Seek:\n");
    ETNA_VEC_FOR_EACH_ENTRY(&priv->segment.seek_head, idx) {
        matroska_seek_t seek = ETNA_VEC_AT(&priv->segment.seek_head, idx);
        ETNA_INFO(NULL, "    0x%x: 0x%llx\n", seek.id, seek.position);
    }
    ETNA_INFO(NULL, "  Info:\n");
    ETNA_INFO(NULL, "    Timestamp Scale: %lld\n", priv->segment.info.timestamp_scale);
    ETNA_INFO(NULL, "    Muxing App: %s\n", priv->segment.info.muxing_app);
    ETNA_INFO(NULL, "    Writing App: %s\n", priv->segment.info.writing_app);
    ETNA_INFO(NULL, "    Duration: %fs\n",
              priv->segment.info.duration * priv->segment.info.timestamp_scale / 1e9);
    ETNA_INFO(NULL, "  Tracks:\n");
    ETNA_VEC_FOR_EACH_ENTRY(&priv->segment.tracks, idx) {
        etna_track_t track = ETNA_VEC_AT(&priv->segment.tracks, idx);
        ETNA_INFO(NULL, "    %d: \n", track.id);

        const char* track_type = "unk";
        const char* codec = "unk";

        switch (track.type) {
            case ETNA_TRACK_VIDEO:
                track_type = "video";
                break;
            case ETNA_TRACK_AUDIO:
                track_type = "audio";
                break;
            case ETNA_TRACK_SUBTITLE:
                track_type = "subtitle";
                break;
            default:
                break;
        }
        switch (track.codec) {
            case ETNA_CODEC_H264:
                codec = "h264 (avc)";
                break;
            default:
                break;
        }

        ETNA_INFO(NULL, "      Type: %s\n", track_type);
        ETNA_INFO(NULL, "      Codec: %s\n", codec);
        ETNA_INFO(NULL, "      Codec Private Data: %lld bytes\n", track.codec_data_len);
    }
    ETNA_INFO(NULL, "  Clusters:\n");
    ETNA_VEC_FOR_EACH_ENTRY(&priv->segment.clusters, idx) {
        matroska_cluster_t cluster = ETNA_VEC_AT(&priv->segment.clusters, idx);
        ETNA_INFO(NULL, "    %d: %d blocks\n", idx, cluster.blocks.length);
    }

    return demuxer;
}

void destroy(etna_demuxer_t* demuxer) {
    matroska_demuxer_priv_t* priv = (matroska_demuxer_priv_t*)demuxer->priv;
    ETNA_VEC_FREE(&priv->segment.seek_head);
    ETNA_FREE(priv->segment.info.muxing_app);
    ETNA_FREE(priv->segment.info.writing_app);
    ETNA_VEC_FOR_EACH_ENTRY(&priv->segment.tracks, idx) {
        etna_track_t track = ETNA_VEC_AT(&priv->segment.tracks, idx);
        ETNA_FREE(track.codec_data);
    }
    ETNA_FREE(demuxer->priv);
    ETNA_FREE(demuxer);
}

etna_demuxer_def_t matroska_demuxer = {
    .is_supported = is_supported, .create = create, .destroy = destroy};

#undef CMP