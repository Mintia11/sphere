#pragma once

#include "instance.h"
#include "log.h"
#include "surface.h"

typedef struct {
    etna_log_scope_t* log_scope;
    VkPhysicalDevice physical_device;
    VkDevice device;
} etna_vk_device_t;

etna_vk_device_t* etna_vk_create_device(etna_vk_instance_t* inst, etna_vk_surface_t* surf);
void etna_vk_destroy_device(etna_vk_device_t* device);