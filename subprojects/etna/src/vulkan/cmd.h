#pragma once

#include <volk.h>
#include "log.h"
#include "vec.h"

typedef struct {
    etna_log_scope_t* log_scope;
    VkCommandBuffer buf;

    ETNA_VEC(VkSemaphore) wait_binary;
    ETNA_VEC(VkSemaphore) signal_binary;
} etna_vk_cmdbuf_t;

void etna_vk_cmd_begin(etna_vk_cmdbuf_t* cmdbuf);
void etna_vk_cmd_end(etna_vk_cmdbuf_t* cmdbuf);
void etna_vk_cmd_wait_binary(etna_vk_cmdbuf_t* cmdbuf, VkSemaphore semaphore);
void etna_vk_cmd_signal_binary(etna_vk_cmdbuf_t* cmdbuf, VkSemaphore semaphore);