#pragma once

#include "device.h"
#include "log.h"
#include "surface.h"
#include <volk.h>

typedef struct {
    etna_log_scope_t* log_scope;
    etna_vk_surface_t* surface;

    VkSurfaceFormat2KHR* supported_formats;
    uint32_t supported_formats_count;
    VkSurfaceCapabilities2KHR caps;

    VkSwapchainKHR swapchain;
    bool recreate;

    VkFormat selected_image_format;
    VkColorSpaceKHR selected_color_space;
    VkPresentModeKHR selected_present_mode;
} etna_vk_swapchain_t;

etna_vk_swapchain_t* etna_vk_create_swapchain(etna_vk_device_t* device, etna_vk_surface_t* surface,
                                              VkFormat image_format, VkColorSpaceKHR color_space,
                                              VkPresentModeKHR present_mode);
void etna_vk_destroy_swapchain(etna_vk_swapchain_t* swapchain);
void etna_vk_swapchain_change_format(etna_vk_swapchain_t* swapchain, VkFormat image_format,
                                     VkColorSpaceKHR color_space);