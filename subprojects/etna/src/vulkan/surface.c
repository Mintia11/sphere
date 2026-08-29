#include "surface.h"
#include "alloc.h"
#include "common.h"
#include "instance.h"

etna_vk_surface* etna_vk_create_surface(etna_vk_surface_create_info* info) {
    etna_vk_instance* inst = info->inst;
    etna_vk_surface* surface = ETNA_ALLOC_TYPE(inst, etna_vk_surface);
    etna_log_scope_t* log = etna_log_scope_new("surface", inst->log_scope);
    surface->log_scope = log;

    VkWin32SurfaceCreateInfoKHR create_info = {
        .sType = VK_STRUCTURE_TYPE_WIN32_SURFACE_CREATE_INFO_KHR,
        .hinstance = info->hinstance,
        .hwnd = info->hwnd};

    VkSurfaceKHR surf = NULL;
    VK_CHECK(log, vkCreateWin32SurfaceKHR(inst->instance, &create_info, VK_ALLOC(surface), &surf));
    surface->surface = surf;

    return surface;
}