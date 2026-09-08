#include "sync.h"
#include "alloc.h"
#include "common.h"

etna_vk_semaphore_t* etna_vk_create_semaphore(etna_vk_device_t* device, uint64_t initial_value) {
    VkSemaphoreTypeCreateInfo type_info = {
        .sType = VK_STRUCTURE_TYPE_SEMAPHORE_TYPE_CREATE_INFO,
        .pNext = NULL,
        .semaphoreType = VK_SEMAPHORE_TYPE_TIMELINE,
        .initialValue = initial_value,
    };

    VkSemaphoreCreateInfo create_info = {
        .sType = VK_STRUCTURE_TYPE_SEMAPHORE_CREATE_INFO,
        .pNext = &type_info,
        .flags = 0,
    };

    VkSemaphore semaphore = NULL;
    VK_CHECK(device->log_scope,
             vkCreateSemaphore(device->device, &create_info, VK_ALLOC(device), &semaphore));

    etna_vk_semaphore_t* sema = ETNA_ALLOC_TYPE(device, etna_vk_semaphore_t);
    sema->semaphore = semaphore;
    sema->target_value = 0;

    return sema;
}

uint64_t etna_vk_semaphore_get_value(etna_vk_semaphore_t* semaphore) {
    etna_vk_device_t* device = ETNA_ALLOCATION_GET_PARENT(semaphore, etna_vk_device_t);
    uint64_t value = 0;
    VK_CHECK(device->log_scope,
             vkGetSemaphoreCounterValue(device->device, semaphore->semaphore, &value));
    return value;
}

void etna_vk_destroy_semaphore(etna_vk_semaphore_t* semaphore) {
    etna_vk_device_t* device = ETNA_ALLOCATION_GET_PARENT(semaphore, etna_vk_device_t);

    vkDestroySemaphore(device->device, semaphore->semaphore, VK_ALLOC(semaphore));
    ETNA_FREE(semaphore);
}