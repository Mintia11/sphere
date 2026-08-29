#include "surface.h"
#include "alloc.h"
#include "common.h"
#include "instance.h"

etna_vk_surface_t* etna_vk_create_surface(etna_vk_surface_create_info_t* info) {
    etna_vk_instance_t* inst = info->inst;
    etna_vk_surface_t* surface = ETNA_ALLOC_TYPE(inst, etna_vk_surface_t);
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

void etna_vk_destroy_surface(etna_vk_surface_t* surf) {
    etna_vk_instance_t* inst = ETNA_ALLOCATION_GET_PARENT(surf, etna_vk_instance_t);
    vkDestroySurfaceKHR(inst->instance, surf->surface, VK_ALLOC(surf));
    ETNA_FREE(surf->log_scope);
    ETNA_FREE(surf);
}