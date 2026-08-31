#include "cmdpool.h"
#include <stdlib.h>
#include "alloc.h"
#include "common.h"
#include "vec.h"

etna_vk_cmdpool_t* etna_vk_create_cmdpool(etna_vk_device_t* device, uint32_t queue_family_idx,
                                          VkQueueFamilyProperties2* props) {
    uint32_t queue_count = props->queueFamilyProperties.queueCount;
    VkQueueFlags flags = props->queueFamilyProperties.queueFlags;

    const char* pool_name = NULL;
    if ((flags & VK_QUEUE_GRAPHICS_BIT) != 0) {
        pool_name = "graphics_pool";
    } else if ((flags & VK_QUEUE_VIDEO_DECODE_BIT_KHR) != 0) {
        pool_name = "decode_pool";
    }

    etna_log_scope_t* log = etna_log_scope_new(pool_name, device->log_scope);
    etna_vk_cmdpool_t* cmdpool = ETNA_ALLOC_TYPE(device, etna_vk_cmdpool_t);
    cmdpool->log_scope = log;

    VkSemaphoreTypeCreateInfo type_info = {
        .sType = VK_STRUCTURE_TYPE_SEMAPHORE_TYPE_CREATE_INFO,
        .pNext = NULL,
        .semaphoreType = VK_SEMAPHORE_TYPE_TIMELINE,
        .initialValue = 0,
    };

    VkSemaphoreCreateInfo create_info = {
        .sType = VK_STRUCTURE_TYPE_SEMAPHORE_CREATE_INFO,
        .pNext = NULL,
        .flags = 0,
    };

    VK_PUSH(&create_info, &type_info);

    for (uint32_t i = 0; i < 1; i++) {
        VkQueue queue = NULL;
        VkDeviceQueueInfo2 queue_info = {
            .sType = VK_STRUCTURE_TYPE_DEVICE_QUEUE_INFO_2,
            .pNext = NULL,
            .flags = VK_DEVICE_QUEUE_CREATE_INTERNALLY_SYNCHRONIZED_BIT_KHR,
            .queueFamilyIndex = queue_family_idx,
            .queueIndex = i,
        };
        vkGetDeviceQueue2(device->device, &queue_info, &queue);
        ETNA_VEC_PUSH(&cmdpool->queues, queue);

        VkSemaphore semaphore;
        VK_CHECK(log,
                 vkCreateSemaphore(device->device, &create_info, VK_ALLOC(cmdpool), &semaphore));
        ETNA_VEC_PUSH(&cmdpool->semaphores, semaphore);
    }

    VkCommandPoolCreateInfo cmdpool_info = {
        .sType = VK_STRUCTURE_TYPE_COMMAND_POOL_CREATE_INFO,
        .pNext = NULL,
        .flags =
            VK_COMMAND_POOL_CREATE_TRANSIENT_BIT | VK_COMMAND_POOL_CREATE_RESET_COMMAND_BUFFER_BIT,
        .queueFamilyIndex = queue_family_idx};

    VkCommandPool handle = NULL;
    VK_CHECK(log, vkCreateCommandPool(device->device, &cmdpool_info, VK_ALLOC(cmdpool), &handle));
    cmdpool->pool = handle;

    return cmdpool;
}

void etna_vk_destroy_cmdpool(etna_vk_cmdpool_t* cmdpool) {
    etna_vk_device_t* device = ETNA_ALLOCATION_GET_PARENT(cmdpool, etna_vk_device_t);

    ETNA_FREE(cmdpool->log_scope);
    ETNA_VEC_FOR_EACH_ENTRY(&cmdpool->semaphores, idx) {
        vkDestroySemaphore(device->device, ETNA_VEC_AT(&cmdpool->semaphores, idx),
                           VK_ALLOC(cmdpool));
    }
    ETNA_VEC_FREE(&cmdpool->semaphores);
    ETNA_VEC_FREE(&cmdpool->queues);
    vkDestroyCommandPool(device->device, cmdpool->pool, VK_ALLOC(cmdpool));
    ETNA_FREE(cmdpool);

    if (ETNA_REFCOUNT(cmdpool) != 0) {
        ETNA_FATAL(NULL, "tried to free command pool with %d active references\n",
                   ETNA_REFCOUNT(cmdpool));
        exit(1);
    }
}