#pragma once

#include "device.h"
#include <volk.h>

typedef struct {
    VkSemaphore semaphore;
    uint64_t value;
} etna_vk_semaphore_t;

etna_vk_semaphore_t* etna_vk_create_semaphore(etna_vk_device_t* device, uint64_t initial_value);
uint64_t etna_vk_semaphore_get_value(etna_vk_semaphore_t* semaphore);
void etna_vk_destroy_semaphore(etna_vk_semaphore_t* semaphore);