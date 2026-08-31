#pragma once

#include "cmd.h"
#include "device.h"
#include "log.h"
#include "vec.h"
#include <volk.h>

typedef struct etna_vk_cmdpool {
    etna_log_scope_t* log_scope;
    uint32_t queue_family_idx;
    VkCommandPool pool;
    ETNA_VEC(VkQueue) queues;
    ETNA_VEC(VkSemaphore) semaphores;
} etna_vk_cmdpool_t;

etna_vk_cmdpool_t* etna_vk_create_cmdpool(etna_vk_device_t* device, uint32_t queue_family_idx,
                                          VkQueueFamilyProperties2* props);
void etna_vk_destroy_cmdpool(etna_vk_cmdpool_t* cmdpool);
etna_vk_cmdbuf_t* etna_vk_alloc_cmdbuffer(etna_vk_cmdpool_t* cmdpool);
void etna_vk_submit_cmdbuf(etna_vk_cmdbuf_t* cmdbuf);