#include "instance.h"
#include <stdint.h>
#include <string.h>
#include "alloc.h"
#include "common.h"
#include "log.h"
#include "vec.h"

const char* vk_instance_exts[] = {
    VK_KHR_SURFACE_EXTENSION_NAME,
    VK_EXT_SWAPCHAIN_COLOR_SPACE_EXTENSION_NAME,
    VK_KHR_GET_SURFACE_CAPABILITIES_2_EXTENSION_NAME,
    VK_KHR_SURFACE_MAINTENANCE_1_EXTENSION_NAME,
    VK_KHR_WIN32_SURFACE_EXTENSION_NAME,
};

static VkBool32 VKAPI_PTR etna_vk_debug_utils_callback(
    VkDebugUtilsMessageSeverityFlagBitsEXT vk_severity, VkDebugUtilsMessageTypeFlagsEXT msg_type,
    const VkDebugUtilsMessengerCallbackDataEXT* data, void* priv) {
    etna_log_scope_t* vk_scope = priv;

    etna_log_severity_t severity;
    switch (vk_severity) {
        case VK_DEBUG_UTILS_MESSAGE_SEVERITY_ERROR_BIT_EXT:
            severity = ETNA_LOG_ERROR;
            break;
        case VK_DEBUG_UTILS_MESSAGE_SEVERITY_WARNING_BIT_EXT:
            severity = ETNA_LOG_WARN;
            break;
        case VK_DEBUG_UTILS_MESSAGE_SEVERITY_INFO_BIT_EXT:
            severity = ETNA_LOG_DEBUG;
            break;
        case VK_DEBUG_UTILS_MESSAGE_SEVERITY_VERBOSE_BIT_EXT:
            severity = ETNA_LOG_TRACE;
            break;
        default:
            severity = ETNA_LOG_INFO;
            break;
    }

    etna_log_message(etna_global_logger, vk_scope, severity, "%s\n", data->pMessage);

    if ((severity & VK_DEBUG_UTILS_MESSAGE_SEVERITY_ERROR_BIT_EXT) &&
        (msg_type & VK_DEBUG_UTILS_MESSAGE_TYPE_VALIDATION_BIT_EXT)) {
        ETNA_FATAL(vk_scope, "validation error");
        return true;
    }

    return false;
}

etna_vk_instance* etna_vk_create_instance(bool debug) {
    etna_vk_instance* inst = ETNA_ALLOC_TYPE(NULL, etna_vk_instance);
    etna_log_scope_t* log = etna_log_scope_new("vk", NULL);
    inst->log_scope = log;

    volkInitialize();

    uint32_t api_version = VK_API_VERSION_1_0;
    VK_CHECK(log, vkEnumerateInstanceVersion(&api_version));

    ETNA_DEBUG(log, "available instance version: %d.%d.%d\n", VK_API_VERSION_MAJOR(api_version),
               VK_API_VERSION_MINOR(api_version), VK_API_VERSION_PATCH(api_version));

    VkInstanceCreateInfo info = {
        .sType = VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO,
        .pApplicationInfo =
            &(VkApplicationInfo){
                .sType = VK_STRUCTURE_TYPE_APPLICATION_INFO,
                .apiVersion = api_version,
            },
    };

    uint32_t layer_count = 0;
    VK_CHECK(log, vkEnumerateInstanceLayerProperties(&layer_count, NULL));
    VkLayerProperties* layers = ETNA_CALLOC_TYPE(inst, VkLayerProperties, layer_count);
    VK_CHECK(log, vkEnumerateInstanceLayerProperties(&layer_count, layers));

    ETNA_DEBUG(log, "available instance layers:\n");
    for (uint32_t i = 0; i < layer_count; i++) {
        ETNA_DEBUG(log, "    %s (v%d.%d.%d)\n", layers[i].layerName,
                   VK_API_VERSION_MAJOR(api_version), VK_API_VERSION_MINOR(api_version),
                   VK_API_VERSION_PATCH(api_version));
    }

    ETNA_VEC(const char*) used_layers = ETNA_VEC_INIT;
    if (debug) {
        bool debug_layers_enabled = false;
        for (uint32_t i = 0; i < layer_count; i++) {
            const char* layer = layers[i].layerName;

            if (strcmp(layer, "VK_LAYER_KHRONOS_validation") != 0) {
                continue;
            }

            ETNA_VEC_PUSH(&used_layers, layer);
            debug_layers_enabled = true;
        }

        if (!debug_layers_enabled) {
            ETNA_WARN(log, "api validation layers requested, but couldn't be found");
        }
    }

    uint32_t global_ext_count = 0;
    VK_CHECK(log, vkEnumerateInstanceExtensionProperties(NULL, &global_ext_count, NULL));
    VkExtensionProperties* global_ext =
        ETNA_CALLOC_TYPE(inst, VkExtensionProperties, global_ext_count);
    VK_CHECK(log, vkEnumerateInstanceExtensionProperties(NULL, &global_ext_count, global_ext));

    typedef struct {
        VkExtensionProperties* exts;
        uint32_t extension_count;
        const char* layer_name;
    } layer_ext_prop_t;
    ETNA_VEC(layer_ext_prop_t) layer_ext = ETNA_VEC_INIT;

    for (uint32_t i = 0; i < layer_count; i++) {
        uint32_t layer_ext_count = 0;
        VK_CHECK(log, vkEnumerateInstanceExtensionProperties(layers[i].layerName, &layer_ext_count,
                                                             NULL));
        VkExtensionProperties* exts =
            ETNA_CALLOC_TYPE(inst, VkExtensionProperties, layer_ext_count);
        VK_CHECK(log, vkEnumerateInstanceExtensionProperties(layers[i].layerName, &layer_ext_count,
                                                             exts));

        layer_ext_prop_t prop = {
            .exts = exts, .extension_count = layer_ext_count, .layer_name = layers[i].layerName};
        ETNA_VEC_PUSH(&layer_ext, prop);
    }

    ETNA_VEC_FOR_EACH_ENTRY(&layer_ext, idx) {
        for (uint32_t i = 0; i < global_ext_count; i++) {
            for (uint32_t j = 0; j < ETNA_VEC_AT(&layer_ext, idx).extension_count; j++) {
                const char* ext_name = ETNA_VEC_AT(&layer_ext, idx).exts[j].extensionName;
                if (strcmp(ext_name, global_ext[i].extensionName) == 0) {
                    memset(ETNA_VEC_AT(&layer_ext, idx).exts[j].extensionName, 0,
                           VK_MAX_EXTENSION_NAME_SIZE);
                }
            }
        }
    }

    ETNA_DEBUG(log, "available instance extensions:\n");
    for (uint32_t i = 0; i < global_ext_count; i++) {
        ETNA_DEBUG(log, "    %s\n", global_ext[i].extensionName);
    }
    ETNA_VEC_FOR_EACH_ENTRY(&layer_ext, idx) {
        for (uint32_t i = 0; i < ETNA_VEC_AT(&layer_ext, idx).extension_count; i++) {
            const char* ext_name = ETNA_VEC_AT(&layer_ext, idx).exts[i].extensionName;
            if (ext_name[0] != 0) {
                ETNA_DEBUG(log, "    %s (via %s)\n", ext_name,
                           ETNA_VEC_AT(&layer_ext, idx).layer_name);
            }
        }
    }

    ETNA_VEC(const char*) used_exts = ETNA_VEC_INIT;

    for (size_t i = 0; i < sizeof(vk_instance_exts) / sizeof(vk_instance_exts[0]); i++) {
        const char* ext = vk_instance_exts[i];
        for (size_t j = 0; j < global_ext_count; j++) {
            if (strcmp(ext, global_ext[j].extensionName) == 0) {
                ETNA_VEC_PUSH(&used_exts, ext);
                break;
            }
        }
    }

    etna_log_scope_t* validation = etna_log_scope_new("validation", log);

    const VkDebugUtilsMessengerCreateInfoEXT debug_info = {
        .sType = VK_STRUCTURE_TYPE_DEBUG_UTILS_MESSENGER_CREATE_INFO_EXT,
        .messageSeverity = VK_DEBUG_UTILS_MESSAGE_SEVERITY_VERBOSE_BIT_EXT |
                           VK_DEBUG_UTILS_MESSAGE_SEVERITY_INFO_BIT_EXT |
                           VK_DEBUG_UTILS_MESSAGE_SEVERITY_WARNING_BIT_EXT |
                           VK_DEBUG_UTILS_MESSAGE_SEVERITY_ERROR_BIT_EXT,
        .messageType = VK_DEBUG_UTILS_MESSAGE_TYPE_GENERAL_BIT_EXT |
                       VK_DEBUG_UTILS_MESSAGE_TYPE_VALIDATION_BIT_EXT |
                       VK_DEBUG_UTILS_MESSAGE_TYPE_PERFORMANCE_BIT_EXT,
        .pfnUserCallback = etna_vk_debug_utils_callback,
        .pUserData = (void*)validation,
    };

    if (debug) {
        const char* ext = VK_EXT_DEBUG_UTILS_EXTENSION_NAME;
        bool found = false;
        for (size_t j = 0; j < global_ext_count; j++) {
            if (strcmp(ext, global_ext[j].extensionName) == 0) {
                ETNA_VEC_PUSH(&used_exts, ext);
                VK_PUSH(&info, &debug_info);
                found = true;
                break;
            }
        }

        if (!found) {
            ETNA_WARN(log, "debug utils extensions were requested but couldn't be found");
        }
    }

#define ENABLE_BOOL(name)                                                                       \
    {                                                                                           \
        "VK_LAYER_KHRONOS_validation", name, VK_LAYER_SETTING_TYPE_BOOL32_EXT, 1, &(VkBool32) { \
            VK_TRUE                                                                             \
        }                                                                                       \
    }

    const VkLayerSettingEXT debug_settings[] = {
        ENABLE_BOOL("validate_best_practices"),
        ENABLE_BOOL("legacy_detection"),
        ENABLE_BOOL("validate_sync"),
        ENABLE_BOOL("syncval_shader_accesses_heuristic"),
        ENABLE_BOOL("syncval_submit_time_validation"),
    };

    VkLayerSettingsCreateInfoEXT layer_settings = {
        .sType = VK_STRUCTURE_TYPE_LAYER_SETTINGS_CREATE_INFO_EXT,
        .settingCount = sizeof(debug_settings) / sizeof(debug_settings[0]),
        .pSettings = debug_settings,
    };

    if (debug) {
        const char* ext = VK_EXT_LAYER_SETTINGS_EXTENSION_NAME;
        bool found = false;
        ETNA_VEC_FOR_EACH_ENTRY(&layer_ext, idx) {
            for (uint32_t i = 0; i < ETNA_VEC_AT(&layer_ext, idx).extension_count; i++) {
                const char* ext_name = ETNA_VEC_AT(&layer_ext, idx).exts[i].extensionName;
                if (strcmp(ext, ext_name) == 0) {
                    found = true;
                    ETNA_VEC_PUSH(&used_exts, ext);
                    VK_PUSH(&info, &layer_settings);
                    goto layer_settings_found;
                }
            }
        }

    layer_settings_found:
        if (!found) {
            ETNA_WARN(log, "couldn't enable extra validation settings");
        }
    }

    ETNA_DEBUG(log, "creating instance with:\n");
    ETNA_DEBUG(log, "enabled instance layers:\n");
    ETNA_VEC_FOR_EACH_ENTRY(&used_layers, idx) {
        ETNA_DEBUG(log, "    %s\n", ETNA_VEC_AT(&used_layers, idx));
    }

    ETNA_DEBUG(log, "enabled instance extensions:\n");
    ETNA_VEC_FOR_EACH_ENTRY(&used_exts, idx) {
        ETNA_DEBUG(log, "    %s\n", ETNA_VEC_AT(&used_exts, idx));
    }

    info.enabledLayerCount = used_layers.length;
    info.ppEnabledLayerNames = used_layers.data;
    info.enabledExtensionCount = used_exts.length;
    info.ppEnabledExtensionNames = used_exts.data;

    VkInstance instance = NULL;
    VK_CHECK(log, vkCreateInstance(&info, VK_ALLOC(inst), &instance));
    volkLoadInstance(instance);
    inst->instance = instance;

    if (debug) {
        VkDebugUtilsMessengerEXT debug_utils_messenger;
        vkCreateDebugUtilsMessengerEXT(instance, &debug_info, VK_ALLOC(instance),
                                       &debug_utils_messenger);
    }

    ETNA_VEC_FREE(&used_exts);
    ETNA_VEC_FOR_EACH_ENTRY(&layer_ext, idx) {
        ETNA_FREE(ETNA_VEC_AT(&layer_ext, idx).exts);
    }
    ETNA_VEC_FREE(&layer_ext);
    ETNA_VEC_FREE(&used_layers);
    ETNA_FREE(layers);

    return inst;
}