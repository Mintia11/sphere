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

etna_vk_cmdbuf_t* etna_vk_alloc_cmdbuffer(etna_vk_cmdpool_t* cmdpool) {
    etna_vk_device_t* device = ETNA_ALLOCATION_GET_PARENT(cmdpool, etna_vk_device_t);

    VkCommandBufferAllocateInfo alloc_info = {0};
    alloc_info.sType = VK_STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO;
    alloc_info.commandPool = cmdpool->pool;
    alloc_info.commandBufferCount = 1;
    alloc_info.level = VK_COMMAND_BUFFER_LEVEL_PRIMARY;

    VkCommandBuffer out = NULL;
    VK_CHECK(cmdpool->log_scope, vkAllocateCommandBuffers(device->device, &alloc_info, &out));

    etna_log_scope_t* log = etna_log_scope_new("cmdbuf", cmdpool->log_scope);
    etna_vk_cmdbuf_t* cmdbuf = ETNA_ALLOC_TYPE(cmdpool, etna_vk_cmdbuf_t);
    cmdbuf->log_scope = log;
    cmdbuf->buf = out;

    return cmdbuf;
}

void etna_vk_submit_cmdbuf(etna_vk_cmdbuf_t* cmdbuf) {
    etna_vk_cmdpool_t* cmdpool = ETNA_ALLOCATION_GET_PARENT(cmdbuf, etna_vk_cmdpool_t);

    VkCommandBufferSubmitInfo cmdbuf_info = {0};
    cmdbuf_info.sType = VK_STRUCTURE_TYPE_COMMAND_BUFFER_SUBMIT_INFO;
    cmdbuf_info.commandBuffer = cmdbuf->buf;
    cmdbuf_info.deviceMask = 1;

    ETNA_VEC(VkSemaphoreSubmitInfo) wait_infos = ETNA_VEC_INIT;
    ETNA_VEC(VkSemaphoreSubmitInfo) signal_infos = ETNA_VEC_INIT;

    ETNA_VEC_FOR_EACH_ENTRY(&cmdbuf->wait_binary, idx) {
        VkSemaphore sema = ETNA_VEC_AT(&cmdbuf->wait_binary, idx);

        VkSemaphoreSubmitInfo info = {0};
        info.sType = VK_STRUCTURE_TYPE_SEMAPHORE_SUBMIT_INFO;
        info.semaphore = sema;
        info.deviceIndex = 1;

        ETNA_VEC_PUSH(&wait_infos, info);
    }

    ETNA_VEC_FOR_EACH_ENTRY(&cmdbuf->signal_binary, idx) {
        VkSemaphore sema = ETNA_VEC_AT(&cmdbuf->signal_binary, idx);

        VkSemaphoreSubmitInfo info = {0};
        info.sType = VK_STRUCTURE_TYPE_SEMAPHORE_SUBMIT_INFO;
        info.semaphore = sema;
        info.deviceIndex = 1;

        ETNA_VEC_PUSH(&signal_infos, info);
    }

    VkSubmitInfo2 submit_info = {0};
    submit_info.sType = VK_STRUCTURE_TYPE_SUBMIT_INFO_2;
    submit_info.commandBufferInfoCount = 1;
    submit_info.pCommandBufferInfos = &cmdbuf_info;
    submit_info.waitSemaphoreInfoCount = wait_infos.length;
    submit_info.pWaitSemaphoreInfos = wait_infos.data;
    submit_info.signalSemaphoreInfoCount = signal_infos.length;
    submit_info.pSignalSemaphoreInfos = signal_infos.data;

    VK_CHECK(cmdpool->log_scope,
             vkQueueSubmit2(ETNA_VEC_AT(&cmdpool->queues, 0), 1, &submit_info, NULL));

    ETNA_VEC_FREE(&signal_infos);
    ETNA_VEC_FREE(&wait_infos);
    ETNA_VEC_FREE(&cmdbuf->signal_binary);
    ETNA_VEC_FREE(&cmdbuf->wait_binary);
    ETNA_FREE(cmdbuf->log_scope);
    ETNA_FREE(cmdbuf);

    if (ETNA_REFCOUNT(cmdbuf) != 0) {
        ETNA_FATAL(cmdpool->log_scope, "tried to free command buffer with %d active references\n",
                   ETNA_REFCOUNT(cmdbuf));
        exit(1);
    }
}