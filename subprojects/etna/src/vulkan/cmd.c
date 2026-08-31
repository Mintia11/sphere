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

void etna_vk_cmd_wait_binary(etna_vk_cmdbuf_t* cmdbuf, VkSemaphore semaphore) {
    ETNA_VEC_PUSH(&cmdbuf->wait_binary, semaphore);
}

void etna_vk_cmd_signal_binary(etna_vk_cmdbuf_t* cmdbuf, VkSemaphore semaphore) {
    ETNA_VEC_PUSH(&cmdbuf->signal_binary, semaphore);
}