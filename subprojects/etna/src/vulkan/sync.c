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
    sema->value = initial_value;

    return sema;
}

uint64_t etna_vk_semaphore_get_value(etna_vk_semaphore_t* semaphore) {
    return semaphore->value;
}

void etna_vk_destroy_semaphore(etna_vk_semaphore_t* semaphore) {
    ETNA_FREE(semaphore);
}