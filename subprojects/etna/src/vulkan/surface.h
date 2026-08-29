#pragma once

#include <volk.h>
#include <stdbool.h>
#include "log.h"
#include "instance.h"
#include <Windows.h>

typedef struct {
    etna_vk_instance* inst;
    HWND hwnd;
    HINSTANCE hinstance;
} etna_vk_surface_create_info;

typedef struct {
    etna_log_scope_t* log_scope;
    VkSurfaceKHR surface;
} etna_vk_surface;

etna_vk_surface* etna_vk_create_surface(etna_vk_surface_create_info* info);