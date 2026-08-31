#include "swapchain.h"
#include "alloc.h"
#include "common.h"
#include "vec.h"
#include "cmdpool.h"

void recreate_swapchain(etna_vk_swapchain_t* swapchain);

etna_vk_swapchain_t* etna_vk_create_swapchain(etna_vk_device_t* device, etna_vk_surface_t* surface,
                                              VkFormat image_format, VkColorSpaceKHR color_space,
                                              VkPresentModeKHR present_mode) {
    etna_vk_swapchain_t* swapchain = ETNA_ALLOC_TYPE(device, etna_vk_swapchain_t);
    etna_log_scope_t* log = etna_log_scope_new("swapchain", device->log_scope);
    swapchain->log_scope = log;
    ETNA_ADDREF(surface);
    swapchain->surface = surface;

    uint32_t present_mode_count = 0;
    VK_CHECK(log, vkGetPhysicalDeviceSurfacePresentModesKHR(
                      device->physical_device, surface->surface, &present_mode_count, NULL));
    VkPresentModeKHR* present_modes =
        ETNA_CALLOC_TYPE(swapchain, VkPresentModeKHR, present_mode_count);
    VK_CHECK(log,
             vkGetPhysicalDeviceSurfacePresentModesKHR(device->physical_device, surface->surface,
                                                       &present_mode_count, present_modes));

    bool present_mode_supported = false;
    for (uint32_t i = 0; i < present_mode_count; i++)
        if (present_modes[i] == present_mode)
            present_mode_supported = true;

    if (!present_mode_supported) {
        ETNA_WARN(log,
                  "specified present mode isn't supported by current device, switching to "
                  "VK_PRESENT_MODE_FIFO_KHR\n");
        present_mode = VK_PRESENT_MODE_FIFO_KHR;
    }

    VkPhysicalDeviceSurfaceInfo2KHR surface_info = {0};
    surface_info.sType = VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_SURFACE_INFO_2_KHR;
    surface_info.surface = surface->surface;

    VkSurfacePresentModeKHR present_mode_info = {0};
    present_mode_info.sType = VK_STRUCTURE_TYPE_SURFACE_PRESENT_MODE_KHR;
    present_mode_info.presentMode = present_mode;

    VK_PUSH(&surface_info, &present_mode_info);

    uint32_t supported_format_count = 0;
    VK_CHECK(log, vkGetPhysicalDeviceSurfaceFormats2KHR(device->physical_device, &surface_info,
                                                        &supported_format_count, NULL));
    VkSurfaceFormat2KHR* supported_formats =
        ETNA_CALLOC_TYPE(swapchain, VkSurfaceFormat2KHR, supported_format_count);
    for (uint32_t i = 0; i < supported_format_count; i++) {
        supported_formats[i].sType = VK_STRUCTURE_TYPE_SURFACE_FORMAT_2_KHR;
    }
    VK_CHECK(log,
             vkGetPhysicalDeviceSurfaceFormats2KHR(device->physical_device, &surface_info,
                                                   &supported_format_count, supported_formats));

    ETNA_DEBUG(log, "supported formats:\n");
    bool format_supported = false;
    for (uint32_t i = 0; i < supported_format_count; i++) {
        VkSurfaceFormat2KHR format = supported_formats[i];
        ETNA_DEBUG(log, "    %s (%s)\n", string_VkFormat(format.surfaceFormat.format),
                   string_VkColorSpaceKHR(format.surfaceFormat.colorSpace));

        if (format.surfaceFormat.format == image_format &&
            format.surfaceFormat.colorSpace == color_space)
            format_supported = true;
    }

    if (!format_supported) {
        ETNA_WARN(log,
                  "chosen format: %s (%s) isn't supported, downgrading to R8G8B8A8_UNORM "
                  "(SRGB_NONLINEAR)",
                  string_VkFormat(image_format), string_VkColorSpaceKHR(color_space));

        image_format = VK_FORMAT_R8G8B8A8_UNORM;
        color_space = VK_COLOR_SPACE_SRGB_NONLINEAR_KHR;
    }

    VkSurfaceCapabilities2KHR surface_caps = {0};
    surface_caps.sType = VK_STRUCTURE_TYPE_SURFACE_CAPABILITIES_2_KHR;

    VK_CHECK(log, vkGetPhysicalDeviceSurfaceCapabilities2KHR(device->physical_device, &surface_info,
                                                             &surface_caps));

    swapchain->caps = surface_caps;
    swapchain->selected_color_space = color_space;
    swapchain->selected_image_format = image_format;
    swapchain->selected_present_mode = present_mode;
    swapchain->supported_formats = supported_formats;
    swapchain->supported_formats_count = supported_format_count;
    swapchain->recreate = true;

    recreate_swapchain(swapchain);

    ETNA_FREE(present_modes);

    return swapchain;
}

void etna_vk_destroy_swapchain(etna_vk_swapchain_t* swapchain) {
    etna_vk_device_t* device = ETNA_ALLOCATION_GET_PARENT(swapchain, etna_vk_device_t);
    VK_CHECK(swapchain->log_scope, vkDeviceWaitIdle(device->device));
    ETNA_VEC_FOR_EACH_ENTRY(&swapchain->image_views, idx) {
        vkDestroyImageView(device->device, ETNA_VEC_AT(&swapchain->image_views, idx),
                           VK_ALLOC(swapchain));
    }
    ETNA_VEC_FOR_EACH_ENTRY(&swapchain->acquire_semaphores, idx) {
        vkDestroySemaphore(device->device, ETNA_VEC_AT(&swapchain->acquire_semaphores, idx),
                           VK_ALLOC(swapchain));
    }
    ETNA_VEC_FOR_EACH_ENTRY(&swapchain->release_semaphores, idx) {
        vkDestroySemaphore(device->device, ETNA_VEC_AT(&swapchain->release_semaphores, idx),
                           VK_ALLOC(swapchain));
    }
    ETNA_VEC_FOR_EACH_ENTRY(&swapchain->present_fences, idx) {
        vkDestroyFence(device->device, ETNA_VEC_AT(&swapchain->present_fences, idx),
                       VK_ALLOC(swapchain));
    }
    ETNA_VEC_FREE(&swapchain->image_views);
    ETNA_VEC_FREE(&swapchain->acquire_semaphores);
    ETNA_VEC_FREE(&swapchain->release_semaphores);
    ETNA_VEC_FREE(&swapchain->present_fences);
    ETNA_VEC_FREE(&swapchain->images);
    ETNA_FREE(swapchain->supported_formats);
    ETNA_FREE(swapchain->surface);  // decrease it's refcount
    ETNA_FREE(swapchain->log_scope);
    vkDestroySwapchainKHR(device->device, swapchain->swapchain, VK_ALLOC(swapchain));
    ETNA_FREE(swapchain);

    if (ETNA_REFCOUNT(swapchain) != 0) {
        ETNA_FATAL(NULL, "tried to free swapchain with %d active references\n",
                   ETNA_REFCOUNT(swapchain));
        exit(1);
    }
}

void etna_vk_swapchain_change_format(etna_vk_swapchain_t* swapchain, VkFormat image_format,
                                     VkColorSpaceKHR color_space) {
    swapchain->recreate = true;

    bool format_supported = false;
    for (uint32_t i = 0; i < swapchain->supported_formats_count; i++) {
        VkSurfaceFormat2KHR format = swapchain->supported_formats[i];
        if (format.surfaceFormat.format == image_format &&
            format.surfaceFormat.colorSpace == color_space)
            format_supported = true;
    }

    if (!format_supported) {
        ETNA_WARN(swapchain->log_scope,
                  "chosen format: %s (%s) isn't supported, downgrading to R8G8B8A8_UNORM "
                  "(SRGB_NONLINEAR)",
                  string_VkFormat(image_format), string_VkColorSpaceKHR(color_space));

        image_format = VK_FORMAT_R8G8B8A8_UNORM;
        color_space = VK_COLOR_SPACE_SRGB_NONLINEAR_KHR;
    }

    swapchain->selected_color_space = color_space;
    swapchain->selected_image_format = image_format;
}

void recreate_swapchain(etna_vk_swapchain_t* swapchain) {
    VkSwapchainCreateInfoKHR create_info = {0};
    create_info.sType = VK_STRUCTURE_TYPE_SWAPCHAIN_CREATE_INFO_KHR;
    create_info.surface = swapchain->surface->surface;
    create_info.minImageCount = swapchain->caps.surfaceCapabilities.maxImageCount >= 3
                                    ? 3
                                    : swapchain->caps.surfaceCapabilities.minImageCount;
    create_info.imageFormat = swapchain->selected_image_format;
    create_info.imageColorSpace = swapchain->selected_color_space;
    create_info.imageExtent = swapchain->caps.surfaceCapabilities.currentExtent;
    create_info.imageArrayLayers = 1;
    create_info.imageUsage = VK_IMAGE_USAGE_TRANSFER_DST_BIT | VK_IMAGE_USAGE_COLOR_ATTACHMENT_BIT;
    create_info.imageSharingMode = VK_SHARING_MODE_EXCLUSIVE;
    create_info.preTransform = VK_SURFACE_TRANSFORM_IDENTITY_BIT_KHR;
    create_info.compositeAlpha = VK_COMPOSITE_ALPHA_OPAQUE_BIT_KHR;
    create_info.presentMode = swapchain->selected_present_mode;
    create_info.clipped = VK_FALSE;
    create_info.oldSwapchain = swapchain->swapchain;

    VkSwapchainPresentModesCreateInfoKHR present_modes = {0};
    present_modes.sType = VK_STRUCTURE_TYPE_SWAPCHAIN_PRESENT_MODES_CREATE_INFO_KHR;
    present_modes.presentModeCount = 1;
    present_modes.pPresentModes = &swapchain->selected_present_mode;

    VK_PUSH(&create_info, &present_modes);

    etna_vk_device_t* device = ETNA_ALLOCATION_GET_PARENT(swapchain, etna_vk_device_t);
    VK_CHECK(swapchain->log_scope,
             vkCreateSwapchainKHR(device->device, &create_info, VK_ALLOC(swapchain),
                                  &swapchain->swapchain));

    swapchain->recreate = false;

    uint32_t image_count = 0;
    VK_CHECK(swapchain->log_scope,
             vkGetSwapchainImagesKHR(device->device, swapchain->swapchain, &image_count, NULL));
    ETNA_VEC_RESIZE(&swapchain->images, image_count);
    VK_CHECK(swapchain->log_scope, vkGetSwapchainImagesKHR(device->device, swapchain->swapchain,
                                                           &image_count, swapchain->images.data));

    ETNA_VEC_FOR_EACH_ENTRY(&swapchain->image_views, idx) {
        vkDestroyImageView(device->device, ETNA_VEC_AT(&swapchain->image_views, idx),
                           VK_ALLOC(swapchain));
    }
    ETNA_VEC_FOR_EACH_ENTRY(&swapchain->acquire_semaphores, idx) {
        vkDestroySemaphore(device->device, ETNA_VEC_AT(&swapchain->acquire_semaphores, idx),
                           VK_ALLOC(swapchain));
    }
    ETNA_VEC_FOR_EACH_ENTRY(&swapchain->release_semaphores, idx) {
        vkDestroySemaphore(device->device, ETNA_VEC_AT(&swapchain->release_semaphores, idx),
                           VK_ALLOC(swapchain));
    }
    ETNA_VEC_FOR_EACH_ENTRY(&swapchain->present_fences, idx) {
        vkDestroyFence(device->device, ETNA_VEC_AT(&swapchain->present_fences, idx),
                       VK_ALLOC(swapchain));
    }

    ETNA_VEC_FREE(&swapchain->image_views);
    ETNA_VEC_FREE(&swapchain->acquire_semaphores);
    ETNA_VEC_FREE(&swapchain->release_semaphores);
    ETNA_VEC_FREE(&swapchain->present_fences);

    VkImageViewCreateInfo view_info = {0};
    view_info.sType = VK_STRUCTURE_TYPE_IMAGE_VIEW_CREATE_INFO;
    view_info.viewType = VK_IMAGE_VIEW_TYPE_2D;
    view_info.format = swapchain->selected_image_format;
    view_info.components.r = VK_COMPONENT_SWIZZLE_IDENTITY;
    view_info.components.g = VK_COMPONENT_SWIZZLE_IDENTITY;
    view_info.components.b = VK_COMPONENT_SWIZZLE_IDENTITY;
    view_info.components.a = VK_COMPONENT_SWIZZLE_IDENTITY;
    view_info.subresourceRange.aspectMask = VK_IMAGE_ASPECT_COLOR_BIT;
    view_info.subresourceRange.baseArrayLayer = 0;
    view_info.subresourceRange.baseMipLevel = 0;
    view_info.subresourceRange.layerCount = 1;
    view_info.subresourceRange.levelCount = 1;

    VkSemaphoreCreateInfo semaphore_info = {
        .sType = VK_STRUCTURE_TYPE_SEMAPHORE_CREATE_INFO,
        .pNext = NULL,
        .flags = 0,
    };
    VkFenceCreateInfo fence_info = {
        .sType = VK_STRUCTURE_TYPE_FENCE_CREATE_INFO,
        .pNext = NULL,
        .flags = VK_FENCE_CREATE_SIGNALED_BIT,
    };

    for (uint32_t i = 0; i < image_count; i++) {
        VkImageView view;
        VkSemaphore acquire_semaphore;
        VkSemaphore release_semaphore;
        VkFence present_fence;

        view_info.image = ETNA_VEC_AT(&swapchain->images, i);
        VK_CHECK(swapchain->log_scope,
                 vkCreateImageView(device->device, &view_info, VK_ALLOC(swapchain), &view));
        VK_CHECK(swapchain->log_scope, vkCreateSemaphore(device->device, &semaphore_info,
                                                         VK_ALLOC(swapchain), &acquire_semaphore));
        VK_CHECK(swapchain->log_scope, vkCreateSemaphore(device->device, &semaphore_info,
                                                         VK_ALLOC(swapchain), &release_semaphore));
        VK_CHECK(swapchain->log_scope,
                 vkCreateFence(device->device, &fence_info, VK_ALLOC(swapchain), &present_fence));

        ETNA_VEC_PUSH(&swapchain->image_views, view);
        ETNA_VEC_PUSH(&swapchain->acquire_semaphores, acquire_semaphore);
        ETNA_VEC_PUSH(&swapchain->release_semaphores, release_semaphore);
        ETNA_VEC_PUSH(&swapchain->present_fences, present_fence);
    }
}

void etna_vk_swapchain_start_frame(etna_vk_swapchain_t* swapchain, etna_vk_frame_t* frame) {
    etna_vk_device_t* device = ETNA_ALLOCATION_GET_PARENT(swapchain, etna_vk_device_t);

    if (swapchain->recreate) {
        recreate_swapchain(swapchain);
    }

    VkAcquireNextImageInfoKHR acquire_info = {0};
    acquire_info.sType = VK_STRUCTURE_TYPE_ACQUIRE_NEXT_IMAGE_INFO_KHR;
    acquire_info.semaphore = ETNA_VEC_AT(&swapchain->acquire_semaphores, swapchain->image_idx);
    acquire_info.timeout = ~0;
    acquire_info.swapchain = swapchain->swapchain;

    VkResult res = vkAcquireNextImage2KHR(device->device, &acquire_info, &swapchain->image_idx);
    switch (res) {
        case VK_SUCCESS:
            frame->image = ETNA_VEC_AT(&swapchain->images, swapchain->image_idx);
            frame->image_view = ETNA_VEC_AT(&swapchain->image_views, swapchain->image_idx);
            frame->acquire_semaphore =
                ETNA_VEC_AT(&swapchain->acquire_semaphores, swapchain->image_idx);
            frame->release_semaphore =
                ETNA_VEC_AT(&swapchain->release_semaphores, swapchain->image_idx);
            return;
        case VK_ERROR_OUT_OF_DATE_KHR:
            swapchain->recreate = true;
            etna_vk_swapchain_start_frame(swapchain, frame);
            return;
        default:
            VK_CHECK(swapchain->log_scope, res);
            return;
    }
}

void etna_vk_swapchain_end_frame(etna_vk_swapchain_t* swapchain) {
    etna_vk_device_t* device = ETNA_ALLOCATION_GET_PARENT(swapchain, etna_vk_device_t);
    if (swapchain->recreate) {
        return;
    }

    vkResetFences(device->device, 1,
                  &ETNA_VEC_AT(&swapchain->present_fences, swapchain->image_idx));

    VkSwapchainPresentFenceInfoKHR fence_info = {
        .sType = VK_STRUCTURE_TYPE_SWAPCHAIN_PRESENT_FENCE_INFO_KHR,
        .swapchainCount = 1,
        .pFences = &ETNA_VEC_AT(&swapchain->present_fences, swapchain->image_idx),
    };

    VkPresentInfoKHR present_info = {
        .sType = VK_STRUCTURE_TYPE_PRESENT_INFO_KHR,
        .waitSemaphoreCount = 1,
        .pWaitSemaphores = &ETNA_VEC_AT(&swapchain->release_semaphores, swapchain->image_idx),
        .swapchainCount = 1,
        .pSwapchains = &swapchain->swapchain,
        .pImageIndices = &swapchain->image_idx,
    };

    VK_PUSH(&present_info, &fence_info);

    VkResult res = vkQueuePresentKHR(ETNA_VEC_AT(&device->graphics_pool->queues, 0), &present_info);
    switch (res) {
        case VK_SUCCESS:
            return;
        case VK_ERROR_OUT_OF_DATE_KHR:
            swapchain->recreate = true;
            return;
        default:
            VK_CHECK(swapchain->log_scope, res);
            return;
    }
}