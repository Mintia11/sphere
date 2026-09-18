#pragma once

#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include "alloc.h"
#include "log.h"

typedef struct {
    uint32_t id;
    size_t data_start;
    size_t data_end;
} ebml_element_t;

static inline size_t ebml_element_length(ebml_element_t* elem) {
    return elem->data_end - elem->data_start;
}

static inline uint32_t ebml_read_id(FILE* file) {
    uint8_t first = 0;
    if (fread(&first, 1, 1, file) != 1) {
        ETNA_FATAL(NULL, "failed to read first byte of an EBML id");
        exit(1);
    }

    if (first == 0) {
        ETNA_FATAL(NULL, "invalue EBML id leading byte");
        exit(1);
    }

    size_t length = (__builtin_clz(first) - 24) + 1;
    if (length > 4) {
        ETNA_FATAL(NULL, "invalid EBML id length: expected 0-4 found %lld", length);
        exit(1);
    }

    uint8_t buf[4] = {0};
    buf[4 - length] = first;

    if (fread(&buf[4 - length + 1], 1, length - 1, file) != (length - 1)) {
        ETNA_FATAL(NULL, "failed to read rest of an EBML id");
        exit(1);
    }

    uint32_t value = 0;
    memcpy(&value, buf, sizeof(value));
    return __builtin_bswap32(value);
}

static inline uint64_t ebml_read_vint(FILE* file) {
    uint8_t first = 0;
    if (fread(&first, 1, 1, file) != 1) {
        ETNA_FATAL(NULL, "failed to read first byte of an EBML vint");
        exit(1);
    }

    if (first == 0) {
        ETNA_FATAL(NULL, "invalue EBML vint leading byte");
        exit(1);
    }

    size_t length = (__builtin_clz(first) - 24) + 1;
    if (length > 8) {
        ETNA_FATAL(NULL, "invalid EBML vint length: expected 0-8 found %lld", length);
        exit(1);
    }

    uint8_t mask = length == 8 ? 0 : 0xFF >> (length & 0x7);

    uint8_t buf[8] = {0};
    buf[8 - length] = first & mask;

    if (fread(&buf[8 - length + 1], 1, length - 1, file) != (length - 1)) {
        ETNA_FATAL(NULL, "failed to read rest of an EBML vint");
        exit(1);
    }

    uint64_t value = 0;
    memcpy(&value, buf, sizeof(value));
    return __builtin_bswap64(value);
}

static inline uint64_t ebml_read_uinteger(FILE* file, size_t length) {
    if (length > 8) {
        ETNA_FATAL(NULL, "invalid EBML uinteger length: expected 0-8 found %lld", length);
        exit(1);
    }

    uint8_t buf[8] = {0};
    if (fread(&buf[8 - length], 1, length, file) != length) {
        ETNA_FATAL(NULL, "failed to read EBML uinteger");
        exit(1);
    }

    uint64_t value = 0;
    memcpy(&value, buf, sizeof(value));
    return __builtin_bswap64(value);
}

static inline int64_t ebml_read_integer(FILE* file, size_t length) {
    if (length > 8) {
        ETNA_FATAL(NULL, "invalid EBML integer length: expected 0-8 found %lld", length);
        exit(1);
    }

    if (length == 0) {
        return 0;
    }

    uint8_t buf[8] = {0};
    if (fread(&buf[8 - length], 1, length, file) != length) {
        ETNA_FATAL(NULL, "failed to read EBML integer");
        exit(1);
    }

    uint64_t value = 0;
    memcpy(&value, buf, sizeof(value));
    value = __builtin_bswap64(value);

    int shift = (8 - (int)length) * 8;
    return (int64_t)(value << shift) >> shift;
}

static inline char* ebml_read_string(FILE* file, size_t length) {
    char* buf = ETNA_ALLOC(NULL, length + 1);
    buf[length] = '\0';

    if (fread(buf, 1, length, file) != length) {
        ETNA_FATAL(NULL, "failed to read EBML string");
        exit(1);
    }

    return buf;
}

static inline double ebml_read_float(FILE* file, size_t length) {
    switch (length) {
        case 0:
            return 0.0;
        case 4: {
            uint32_t raw = ebml_read_uinteger(file, length);
            float value = 0;
            memcpy(&value, &raw, sizeof(value));
            return value;
        }
        case 8: {
            uint64_t raw = ebml_read_uinteger(file, length);
            double value = 0;
            memcpy(&value, &raw, sizeof(value));
            return value;
        }
        default:
            ETNA_FATAL(NULL, "invalid EBML float length: expected 0,4,8 found %lld", length);
            exit(1);
    }
}

static inline uint8_t* ebml_read_binary(FILE* file, size_t length) {
    uint8_t* buf = ETNA_ALLOC(NULL, length);
    if (fread(buf, 1, length, file) != length) {
        ETNA_FATAL(NULL, "failed to read EBML binary");
        exit(1);
    }

    return buf;
}

static inline void ebml_read_master_element(FILE* file, void* out_element, uint32_t master_id,
                                            void (*read_element)(void* master_element, FILE* file,
                                                                 ebml_element_t* elem),
                                            ebml_element_t* elem) {
#define EBML_READ_ELEMENT(elem)                       \
    do {                                              \
        (elem)->id = ebml_read_id(file);              \
        size_t size = ebml_read_vint(file);           \
        (elem)->data_start = _ftelli64(file);         \
        (elem)->data_end = (elem)->data_start + size; \
    } while (0)

    bool did_allocate = false;
    if (!elem) {
        elem = ETNA_ALLOC_TYPE(NULL, ebml_element_t);
        did_allocate = true;

        EBML_READ_ELEMENT(elem);
    }

    if (elem->id != master_id) {
        ETNA_FATAL(NULL, "wrong id found for master element: expected 0x%x, found 0x%x\n",
                   master_id, elem->id);
        exit(1);
    }

    if ((size_t)_ftelli64(file) != elem->data_start) {
        _fseeki64(file, elem->data_start, SEEK_SET);
    }

    while ((size_t)_ftelli64(file) < elem->data_end) {
        ebml_element_t child = {0};
        EBML_READ_ELEMENT(&child);

        if (child.id == 0xEC || child.id == 0xBF) {
            goto skip;
        }

        read_element(out_element, file, &child);

    skip:
        _fseeki64(file, child.data_end, SEEK_SET);
    }

    if ((size_t)_ftelli64(file) != elem->data_end) {
        ETNA_FATAL(NULL, "BUG: didn't read the full data of an EBML master element");
        exit(1);
    }

    if (did_allocate) {
        ETNA_FREE(elem);
    }

#undef EBML_READ_ELEMENT
}

typedef struct {
    uint64_t version;
    uint64_t read_version;
    size_t max_id_length;
    size_t max_size_length;
    char* doc_type;
    uint64_t doc_type_version;
    uint64_t doc_type_read_version;
} ebml_header_t;

static inline void ebml_read_header_element(void* master_element, FILE* file,
                                            ebml_element_t* elem) {
    ebml_header_t* out = (ebml_header_t*)master_element;

    enum {
        EBML_ID_VERSION = 0x4286,
        EBML_ID_READ_VERSION = 0x42F7,
        EBML_ID_MAX_ID_LENGTH = 0x42F2,
        EBML_ID_MAX_SIZE_LENGTH = 0x42F3,
        EBML_ID_DOC_TYPE = 0x4282,
        EBML_ID_DOC_TYPE_VERSION = 0x4287,
        EBML_ID_DOC_TYPE_READ_VERSION = 0x4285,
    };

    size_t length = ebml_element_length(elem);

    switch (elem->id) {
        case EBML_ID_VERSION:
            out->version = ebml_read_uinteger(file, length);
            break;
        case EBML_ID_READ_VERSION:
            out->read_version = ebml_read_uinteger(file, length);
            break;
        case EBML_ID_MAX_ID_LENGTH:
            out->max_id_length = (size_t)ebml_read_uinteger(file, length);
            break;
        case EBML_ID_MAX_SIZE_LENGTH:
            out->max_size_length = (size_t)ebml_read_uinteger(file, length);
            break;
        case EBML_ID_DOC_TYPE:
            out->doc_type = ebml_read_string(file, length);
            break;
        case EBML_ID_DOC_TYPE_VERSION:
            out->doc_type_version = ebml_read_uinteger(file, length);
            break;
        case EBML_ID_DOC_TYPE_READ_VERSION:
            out->doc_type_read_version = ebml_read_uinteger(file, length);
            break;
        default:
            ETNA_WARN(NULL, "unknown EBML header element: 0x%x\n", elem->id);
            break;
    }
}

static inline ebml_header_t ebml_read_header(FILE* file) {
    ebml_header_t out = {0};
    ebml_read_master_element(file, &out, 0x1A45DFA3, ebml_read_header_element, NULL);
    return out;
}