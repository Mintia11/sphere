#pragma once

#define VK_CHECK(scope, result)                          \
    do {                                                 \
        if ((result) != VK_SUCCESS) {                    \
            ETNA_FATAL(scope, #result " != VK_SUCCESS"); \
        }                                                \
    } while (0)

#define VK_PUSH(into, val)                                         \
    do {                                                           \
        VkBaseOutStructure* current = (VkBaseOutStructure*)(into); \
        while (current->pNext)                                     \
            current = current->pNext;                              \
        current->pNext = (VkBaseOutStructure*)(val);               \
    } while (0)