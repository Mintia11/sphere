#include "cmd.h"
#include "common.h"

void etna_vk_cmd_begin(etna_vk_cmdbuf_t* cmdbuf) {
    VkCommandBufferBeginInfo info = {0};
    info.sType = VK_STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO;

    VK_CHECK(cmdbuf->log_scope, vkBeginCommandBuffer(cmdbuf->buf, &info));
}

void etna_vk_cmd_end(etna_vk_cmdbuf_t* cmdbuf) {
    VK_CHECK(cmdbuf->log_scope, vkEndCommandBuffer(cmdbuf->buf));
}

void etna_vk_cmd_wait_binary(etna_vk_cmdbuf_t* cmdbuf, VkSemaphore semaphore,
                             VkPipelineStageFlags stage_mask) {
    etna_vk_cmdbuf_semaphore_t sema = {.semaphore = semaphore, .stage_mask = stage_mask};
    ETNA_VEC_PUSH(&cmdbuf->wait_binary, sema);
}

void etna_vk_cmd_signal_binary(etna_vk_cmdbuf_t* cmdbuf, VkSemaphore semaphore,
                               VkPipelineStageFlags stage_mask) {
    etna_vk_cmdbuf_semaphore_t sema = {.semaphore = semaphore, .stage_mask = stage_mask};
    ETNA_VEC_PUSH(&cmdbuf->signal_binary, sema);
}

void etna_vk_cmd_image_barrier(etna_vk_cmdbuf_t* cmdbuf, etna_vk_image_t* image,
                               VkImageLayout new_layout, VkAccessFlags new_access,
                               VkPipelineStageFlags new_stage) {
    VkImageSubresourceRange subresource_range = {0};
    subresource_range.aspectMask = VK_IMAGE_ASPECT_COLOR_BIT;
    subresource_range.baseMipLevel = 0;
    subresource_range.levelCount = 1;
    subresource_range.baseArrayLayer = 0;
    subresource_range.layerCount = 1;

    VkImageMemoryBarrier2 barrier = {0};
    barrier.sType = VK_STRUCTURE_TYPE_IMAGE_MEMORY_BARRIER_2;
    barrier.srcStageMask = image->current_stage;
    barrier.srcAccessMask = image->current_access;
    barrier.dstStageMask = new_stage;
    barrier.dstAccessMask = new_access;
    barrier.oldLayout = image->current_layout;
    barrier.newLayout = new_layout;
    barrier.image = image->image;
    barrier.subresourceRange = subresource_range;

    VkDependencyInfo dep_info = {0};
    dep_info.sType = VK_STRUCTURE_TYPE_DEPENDENCY_INFO;
    dep_info.imageMemoryBarrierCount = 1;
    dep_info.pImageMemoryBarriers = &barrier;

    vkCmdPipelineBarrier2(cmdbuf->buf, &dep_info);

    image->current_stage = new_stage;
    image->current_access = new_access;
    image->current_layout = new_layout;
}