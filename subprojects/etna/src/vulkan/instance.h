#pragma once

#include <volk.h>
#include <stdbool.h>
#include "log.h"

typedef struct {
    etna_log_scope_t* log_scope;
    VkInstance instance;
    etna_log_scope_t* validation_scope;
    VkDebugUtilsMessengerEXT debug_messenger;
} etna_vk_instance_t;

etna_vk_instance_t* etna_vk_create_instance(bool debug);
void etna_vk_destroy_instance(etna_vk_instance_t* inst);