#include "device.h"
#include "alloc.h"
#include "instance.h"
#include "common.h"
#include "log.h"
#include "surface.h"
#include "vec.h"
#include "cmdpool.h"

const char* vk_device_exts[] = {
    VK_KHR_SWAPCHAIN_EXTENSION_NAME,
    VK_KHR_SWAPCHAIN_MAINTENANCE_1_EXTENSION_NAME,
    VK_KHR_SWAPCHAIN_MUTABLE_FORMAT_EXTENSION_NAME,
    VK_KHR_INTERNALLY_SYNCHRONIZED_QUEUES_EXTENSION_NAME,
    VK_KHR_VIDEO_QUEUE_EXTENSION_NAME,
    VK_KHR_VIDEO_DECODE_QUEUE_EXTENSION_NAME,
    VK_KHR_VIDEO_DECODE_H264_EXTENSION_NAME,
};

VkPhysicalDevice choose_physical_device(etna_vk_instance_t* inst, etna_vk_surface_t* surf) {
    etna_log_scope_t* log = inst->log_scope;

    uint32_t physical_device_count = 0;
    VK_CHECK(log, vkEnumeratePhysicalDevices(inst->instance, &physical_device_count, NULL));
    VkPhysicalDevice* physical_devices =
        ETNA_CALLOC_TYPE(inst, VkPhysicalDevice, physical_device_count);
    VK_CHECK(log,
             vkEnumeratePhysicalDevices(inst->instance, &physical_device_count, physical_devices));

    ETNA_INFO(log, "vulkan devices:\n");

    for (uint32_t i = 0; i < physical_device_count; i++) {
        VkPhysicalDevice physical_device = physical_devices[i];
        VkPhysicalDeviceProperties2 props = {.sType =
                                                 VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_PROPERTIES_2};
        vkGetPhysicalDeviceProperties2(physical_device, &props);

        ETNA_INFO(log, "    GPU %d: %s\n", i, props.properties.deviceName);

        if (surf) {
            uint32_t queue_family_count = 0;
            vkGetPhysicalDeviceQueueFamilyProperties2(physical_device, &queue_family_count, NULL);

            bool supports_surface = false;
            for (uint32_t queue_family_idx = 0; queue_family_idx < queue_family_count;
                 queue_family_idx++) {
                if (etna_vk_surface_supports_present(surf, physical_device, queue_family_idx)) {
                    supports_surface = true;
                    break;
                }
            }

            if (!supports_surface) {
                ETNA_WARN(log,
                          "    device does not support presentation to the specified surface\n");
                continue;
            }
        }

        if (props.properties.deviceType == VK_PHYSICAL_DEVICE_TYPE_DISCRETE_GPU) {
            ETNA_FREE(physical_devices);
            return physical_device;
        }
    }

    ETNA_WARN(log, "couldn't find a discrete gpu so using the first one");
    VkPhysicalDevice physical_device = physical_devices[0];

    ETNA_FREE(physical_devices);
    return physical_device;
}

uint32_t find_queue_family(VkQueueFamilyProperties2* queue_families, uint32_t queue_family_count,
                           VkQueueFlags flags) {
    for (uint32_t i = 0; i < queue_family_count; i++) {
        if ((queue_families[i].queueFamilyProperties.queueFlags & flags) != 0) {
            return i;
        }
    }

    return -1;
}

etna_vk_device_t* etna_vk_create_device(etna_vk_instance_t* inst, etna_vk_surface_t* surf) {
    etna_log_scope_t* log = etna_log_scope_new("device", inst->log_scope);
    etna_vk_device_t* device = ETNA_ALLOC_TYPE(inst, etna_vk_device_t);
    device->log_scope = log;

    VkPhysicalDevice physical_device = choose_physical_device(inst, surf);
    uint32_t queue_family_count = 0;
    vkGetPhysicalDeviceQueueFamilyProperties2(physical_device, &queue_family_count, NULL);

    VkQueueFamilyProperties2* queue_families =
        ETNA_CALLOC_TYPE(device, VkQueueFamilyProperties2, queue_family_count);
    VkQueueFamilyVideoPropertiesKHR* video_props =
        ETNA_CALLOC_TYPE(queue_families, VkQueueFamilyVideoPropertiesKHR, queue_family_count);
    for (uint32_t i = 0; i < queue_family_count; i++) {
        queue_families[i].sType = VK_STRUCTURE_TYPE_QUEUE_FAMILY_PROPERTIES_2;
        video_props[i].sType = VK_STRUCTURE_TYPE_QUEUE_FAMILY_VIDEO_PROPERTIES_KHR;

        VK_PUSH(&queue_families[i], &video_props[i]);
    }

    vkGetPhysicalDeviceQueueFamilyProperties2(physical_device, &queue_family_count, queue_families);

    ETNA_DEBUG(log, "available queue families:\n");
    for (uint32_t i = 0; i < queue_family_count; i++) {
        ETNA_DEBUG(log, "    family %d: %x (%d queues)\n", i,
                   queue_families[i].queueFamilyProperties.queueFlags,
                   queue_families[i].queueFamilyProperties.queueCount);
    }

    uint32_t graphics_queue =
        find_queue_family(queue_families, queue_family_count, VK_QUEUE_GRAPHICS_BIT);
    uint32_t decode_queue =
        find_queue_family(queue_families, queue_family_count, VK_QUEUE_VIDEO_DECODE_BIT_KHR);

    if (graphics_queue == (uint32_t)-1) {
        ETNA_FATAL(log, "failed to find suitable graphics queue\n");
        exit(1);
    }

    if (decode_queue == (uint32_t)-1) {
        ETNA_FATAL(log, "failed to find suitable decode queue\n");
        exit(1);
    }

    if ((video_props[decode_queue].videoCodecOperations &
         VK_VIDEO_CODEC_OPERATION_DECODE_H264_BIT_KHR) == 0) {
        ETNA_FATAL(log, "decode queue does not support all required codecs\n");
        exit(1);
    }

    ETNA_DEBUG(log, "using graphics queue %d\n", graphics_queue);
    ETNA_DEBUG(log, "using decode queue %d\n", decode_queue);

    uint32_t ext_count = 0;
    VK_CHECK(log, vkEnumerateDeviceExtensionProperties(physical_device, NULL, &ext_count, NULL));
    VkExtensionProperties* exts = ETNA_CALLOC_TYPE(device, VkExtensionProperties, ext_count);
    VK_CHECK(log, vkEnumerateDeviceExtensionProperties(physical_device, NULL, &ext_count, exts));

    ETNA_DEBUG(log, "device extensions:\n");
    for (uint32_t i = 0; i < ext_count; i++) {
        ETNA_DEBUG(log, "    %s\n", exts[i].extensionName);
    }

    ETNA_VEC(const char*) used_exts = ETNA_VEC_INIT;
    for (size_t i = 0; i < sizeof(vk_device_exts) / sizeof(vk_device_exts[0]); i++) {
        const char* ext = vk_device_exts[i];
        bool found = false;
        for (size_t j = 0; j < ext_count; j++) {
            if (strcmp(ext, exts[j].extensionName) == 0) {
                ETNA_VEC_PUSH(&used_exts, ext);
                found = true;
                break;
            }
        }

        if (!found) {
            ETNA_FATAL(log, "could not find required extension %s\n", ext);
            exit(1);
        }
    }

    ETNA_DEBUG(log, "used device extensions:\n");
    ETNA_VEC_FOR_EACH_ENTRY(&used_exts, idx) {
        ETNA_DEBUG(log, "    %s\n", ETNA_VEC_AT(&used_exts, idx));
    }

    ETNA_VEC(VkDeviceQueueCreateInfo) queue_infos = ETNA_VEC_INIT;
    float* queue_priorities = ETNA_CALLOC_TYPE(device, float, 128);
    for (uint32_t i = 0; i < queue_family_count; i++) {
        if (i != graphics_queue && i != decode_queue)
            continue;

        VkDeviceQueueCreateInfo info = {
            .sType = VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO,
            .pNext = NULL,
            .flags = VK_DEVICE_QUEUE_CREATE_INTERNALLY_SYNCHRONIZED_BIT_KHR,
            .queueFamilyIndex = i,
            .queueCount = queue_families[i].queueFamilyProperties.queueCount,
            .pQueuePriorities = queue_priorities,
        };

        ETNA_VEC_PUSH(&queue_infos, info);
    }

    VkPhysicalDeviceSwapchainMaintenance1FeaturesKHR swapchain_maintenance1 = {0};
    swapchain_maintenance1.sType =
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_SWAPCHAIN_MAINTENANCE_1_FEATURES_KHR;
    swapchain_maintenance1.swapchainMaintenance1 = VK_TRUE;

    VkPhysicalDeviceInternallySynchronizedQueuesFeaturesKHR internally_synchronized_queues = {0};
    internally_synchronized_queues.sType =
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_INTERNALLY_SYNCHRONIZED_QUEUES_FEATURES_KHR;
    internally_synchronized_queues.internallySynchronizedQueues = VK_TRUE;

    VkPhysicalDeviceVulkan14Features features14 = {0};
    features14.sType = VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_VULKAN_1_4_FEATURES;

    VkPhysicalDeviceVulkan13Features features13 = {0};
    features13.sType = VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_VULKAN_1_3_FEATURES;
    features13.dynamicRendering = VK_TRUE;
    features13.synchronization2 = VK_TRUE;

    VkPhysicalDeviceVulkan12Features features12 = {0};
    features12.sType = VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_VULKAN_1_2_FEATURES;
    features12.bufferDeviceAddress = VK_TRUE;
    features12.timelineSemaphore = VK_TRUE;

    VkPhysicalDeviceVulkan11Features features11 = {0};
    features11.sType = VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_VULKAN_1_1_FEATURES;
    features11.samplerYcbcrConversion = VK_TRUE;

    VkPhysicalDeviceFeatures2 features = {0};
    features.sType = VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_FEATURES_2;

    VkDeviceCreateInfo create_info = {.sType = VK_STRUCTURE_TYPE_DEVICE_CREATE_INFO,
                                      .pNext = NULL,
                                      .flags = 0,
                                      .queueCreateInfoCount = queue_infos.length,
                                      .pQueueCreateInfos = queue_infos.data,
                                      .enabledLayerCount = 0,
                                      .ppEnabledLayerNames = NULL,
                                      .enabledExtensionCount = used_exts.length,
                                      .ppEnabledExtensionNames = used_exts.data,
                                      .pEnabledFeatures = NULL};

    VK_PUSH(&create_info, &features);
    VK_PUSH(&create_info, &features11);
    VK_PUSH(&create_info, &features12);
    VK_PUSH(&create_info, &features13);
    VK_PUSH(&create_info, &features14);
    VK_PUSH(&create_info, &internally_synchronized_queues);
    VK_PUSH(&create_info, &swapchain_maintenance1);

    VkDevice device_handle = NULL;
    VK_CHECK(log, vkCreateDevice(physical_device, &create_info, VK_ALLOC(device), &device_handle));
    device->physical_device = physical_device;
    device->device = device_handle;

    device->graphics_pool =
        etna_vk_create_cmdpool(device, graphics_queue, &queue_families[graphics_queue]);
    device->decode_pool =
        etna_vk_create_cmdpool(device, decode_queue, &queue_families[decode_queue]);

    ETNA_FREE(video_props);
    ETNA_FREE(queue_families);
    ETNA_FREE(exts);
    ETNA_FREE(queue_priorities);
    ETNA_VEC_FREE(&queue_infos);

    return device;
}

void etna_vk_destroy_device(etna_vk_device_t* device) {
    etna_vk_destroy_cmdpool(device->graphics_pool);
    etna_vk_destroy_cmdpool(device->decode_pool);
    vkDestroyDevice(device->device, VK_ALLOC(device));
    ETNA_FREE(device->log_scope);
    ETNA_FREE(device);

    if (ETNA_REFCOUNT(device) != 0) {
        ETNA_FATAL(NULL, "tried to free device with %d active references\n", ETNA_REFCOUNT(device));
        exit(1);
    }
}