#pragma once

#include <volk.h>
#include <vulkan/vk_enum_string_helper.h>

#define VK_CHECK(scope, result)                                                  \
    do {                                                                         \
        VkResult r = (result);                                                   \
        if (r != VK_SUCCESS) {                                                   \
            ETNA_FATAL(scope, #result "(%s) != VK_SUCCESS", string_VkResult(r)); \
        }                                                                        \
    } while (0)

#define VK_PUSH(into, val)                                         \
    do {                                                           \
        VkBaseOutStructure* current = (VkBaseOutStructure*)(into); \
        while (current->pNext)                                     \
            current = current->pNext;                              \
        current->pNext = (VkBaseOutStructure*)(val);               \
    } while (0)

extern void* etna_vulkan_allocate(void* pUserData, size_t size, size_t alignment,
                                  VkSystemAllocationScope allocationScope);
extern void* etna_vulkan_reallocate(void* pUserData, void* pOriginal, size_t size, size_t alignment,
                                    VkSystemAllocationScope allocationScope);
extern void etna_vulkan_free(void* pUserData, void* pMemory);

#define VK_ALLOC(parent)                                                       \
    &(VkAllocationCallbacks) {                                                 \
        .pUserData = parent, .pfnAllocation = etna_vulkan_allocate,            \
        .pfnReallocation = etna_vulkan_reallocate, .pfnFree = etna_vulkan_free \
    }