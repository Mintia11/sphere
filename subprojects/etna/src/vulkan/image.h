#pragma once

#include <volk.h>

typedef struct {
    VkImage image;
    VkImageView view;
    VkExtent2D extent;
    VkFormat format;
    VkImageLayout current_layout;
    VkAccessFlags current_access;
    VkPipelineStageFlags current_stage;
} etna_vk_image_t;