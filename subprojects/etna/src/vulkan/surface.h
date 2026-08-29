#pragma once

#include <volk.h>
#include <stdbool.h>
#include "log.h"
#include "instance.h"
#include <Windows.h>

typedef struct {
    etna_vk_instance_t* inst;
    HWND hwnd;
    HINSTANCE hinstance;
} etna_vk_surface_create_info_t;

typedef struct {
    etna_log_scope_t* log_scope;
    VkSurfaceKHR surface;
} etna_vk_surface_t;

etna_vk_surface_t* etna_vk_create_surface(etna_vk_surface_create_info_t* info);
void etna_vk_destroy_surface(etna_vk_surface_t* surf);