#include "alloc.h"
#include <volk.h>

void* etna_vulkan_allocate(void* pUserData, size_t size, size_t alignment,
                           VkSystemAllocationScope allocationScope) {
    (void)alignment;
    (void)allocationScope;
    return ETNA_ALLOC(pUserData, size);
}

void* etna_vulkan_reallocate(void* pUserData, void* pOriginal, size_t size, size_t alignment,
                             VkSystemAllocationScope allocationScope) {
    (void)pUserData;
    (void)alignment;
    (void)allocationScope;
    return ETNA_REALLOC(pOriginal, size);
}

void etna_vulkan_free(void* pUserData, void* pMemory) {
    (void)pUserData;
    ETNA_FREE(pMemory);
}