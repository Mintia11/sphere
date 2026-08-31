#pragma once

#include "device.h"
#include "log.h"
#include "surface.h"
#include "vec.h"
#include <volk.h>

typedef struct {
    etna_log_scope_t* log_scope;
    etna_vk_surface_t* surface;

    VkSurfaceFormat2KHR* supported_formats;
    uint32_t supported_formats_count;
    VkSurfaceCapabilities2KHR caps;

    VkSwapchainKHR swapchain;
    bool recreate;
    ETNA_VEC(VkImage) images;
    ETNA_VEC(VkImageView) image_views;
    ETNA_VEC(VkSemaphore) acquire_semaphores;
    ETNA_VEC(VkSemaphore) release_semaphores;
    ETNA_VEC(VkFence) present_fences;
    uint32_t image_idx;

    VkFormat selected_image_format;
    VkColorSpaceKHR selected_color_space;
    VkPresentModeKHR selected_present_mode;
} etna_vk_swapchain_t;

typedef struct {
    VkImage image;
    VkImageView image_view;
    VkSemaphore acquire_semaphore;
    VkSemaphore release_semaphore;
} etna_vk_frame_t;

etna_vk_swapchain_t* etna_vk_create_swapchain(etna_vk_device_t* device, etna_vk_surface_t* surface,
                                              VkFormat image_format, VkColorSpaceKHR color_space,
                                              VkPresentModeKHR present_mode);
void etna_vk_destroy_swapchain(etna_vk_swapchain_t* swapchain);
void etna_vk_swapchain_change_format(etna_vk_swapchain_t* swapchain, VkFormat image_format,
                                     VkColorSpaceKHR color_space);
void etna_vk_swapchain_start_frame(etna_vk_swapchain_t* swapchain, etna_vk_frame_t* frame);
void etna_vk_swapchain_end_frame(etna_vk_swapchain_t* swapchain);