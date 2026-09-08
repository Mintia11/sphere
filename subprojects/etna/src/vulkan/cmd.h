#pragma once

#include <volk.h>
#include "log.h"
#include "vec.h"
#include "image.h"

typedef struct {
    VkSemaphore semaphore;
    VkPipelineStageFlags stage_mask;
} etna_vk_cmdbuf_semaphore_t;

typedef struct {
    etna_log_scope_t* log_scope;
    VkCommandBuffer buf;

    ETNA_VEC(etna_vk_cmdbuf_semaphore_t) wait_binary;
    ETNA_VEC(etna_vk_cmdbuf_semaphore_t) signal_binary;
    uint64_t timeline_value;
} etna_vk_cmdbuf_t;

void etna_vk_cmd_begin(etna_vk_cmdbuf_t* cmdbuf);
void etna_vk_cmd_end(etna_vk_cmdbuf_t* cmdbuf);
void etna_vk_cmd_wait_binary(etna_vk_cmdbuf_t* cmdbuf, VkSemaphore semaphore,
                             VkPipelineStageFlags stage_mask);
void etna_vk_cmd_signal_binary(etna_vk_cmdbuf_t* cmdbuf, VkSemaphore semaphore,
                               VkPipelineStageFlags stage_mask);
void etna_vk_cmd_image_barrier(etna_vk_cmdbuf_t* cmdbuf, etna_vk_image_t* image,
                               VkImageLayout new_layout, VkAccessFlags new_access,
                               VkPipelineStageFlags new_stage);