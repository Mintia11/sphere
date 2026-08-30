#include "surface.h"
#include "alloc.h"
#include "common.h"
#include "instance.h"
#include "log.h"

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

    if (ETNA_REFCOUNT(surf) != 0) {
        ETNA_FATAL(NULL, "tried to free surface with %d active references\n", ETNA_REFCOUNT(surf));
        exit(1);
    }
}

bool etna_vk_surface_supports_present(etna_vk_surface_t* surf, VkPhysicalDevice phys_dev,
                                      uint32_t queue_family_idx) {
    VkBool32 out = false;
    VK_CHECK(surf->log_scope,
             vkGetPhysicalDeviceSurfaceSupportKHR(phys_dev, queue_family_idx, surf->surface, &out));
    return out != 0;
}