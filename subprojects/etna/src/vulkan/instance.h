#pragma once

#include <Volk/volk.h>
#include <stdbool.h>
#include "log.h"

typedef struct {
    etna_log_scope_t* vk_scope;
    VkInstance instance;
} etna_vk_instance;

etna_vk_instance* etna_vk_create_instance(bool debug);