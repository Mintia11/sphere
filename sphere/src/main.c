#include <stdint.h>
#include <mutex.h>
#include <alloc.h>
#include <log.h>
#include <vulkan/instance.h>
#include <SDL3/SDL.h>
#include <Windows.h>
#include <vulkan/surface.h>
#include <vulkan/device.h>
#include <vulkan/swapchain.h>
#include <vulkan/cmdpool.h>
#include <vulkan/cmd.h>

int main() {
    etna_allocator_init_global();
    etna_logger_init_global();

    SDL_Init(SDL_INIT_VIDEO);
    SDL_Window* window =
        SDL_CreateWindow("Sphere", 1366, 768, SDL_WINDOW_VULKAN | SDL_WINDOW_RESIZABLE);

    etna_log_scope_t* main_log = etna_log_scope_new("main", NULL);

    ETNA_INFO(main_log, "Hello world!\n");
    etna_vk_instance_t* inst = etna_vk_create_instance(true);

    SDL_PropertiesID props = SDL_GetWindowProperties(window);
    HWND hwnd = (HWND)SDL_GetPointerProperty(props, SDL_PROP_WINDOW_WIN32_HWND_POINTER, NULL);
    HINSTANCE hinstance =
        (HINSTANCE)SDL_GetPointerProperty(props, SDL_PROP_WINDOW_WIN32_INSTANCE_POINTER, NULL);

    etna_vk_surface_t* surf = etna_vk_create_surface(
        &(etna_vk_surface_create_info_t){.hwnd = hwnd, .inst = inst, .hinstance = hinstance});

    etna_vk_device_t* device = etna_vk_create_device(inst, surf);
    etna_vk_swapchain_t* swapchain =
        etna_vk_create_swapchain(device, surf, VK_FORMAT_R8G8B8A8_UNORM,
                                 VK_COLOR_SPACE_SRGB_NONLINEAR_KHR, VK_PRESENT_MODE_MAILBOX_KHR);

    bool is_running = true;
    SDL_Event event = {0};
    while (is_running) {
        while (SDL_PollEvent(&event)) {
            switch (event.type) {
                case SDL_EVENT_QUIT:
                    is_running = false;
                    break;
                default:
                    break;
            }
        }

        etna_vk_cmdbuf_t* cmdbuf = etna_vk_alloc_cmdbuffer(device->graphics_pool);

        etna_vk_frame_t frame = {0};
        etna_vk_swapchain_start_frame(swapchain, &frame);
        etna_vk_cmd_begin(cmdbuf);
        etna_vk_cmd_end(cmdbuf);

        etna_vk_cmd_wait_binary(cmdbuf, frame.acquire_semaphore);
        etna_vk_cmd_signal_binary(cmdbuf, frame.release_semaphore);
        etna_vk_submit_cmdbuf(cmdbuf);

        etna_vk_swapchain_end_frame(swapchain);
    }

    etna_vk_destroy_swapchain(swapchain);
    etna_vk_destroy_device(device);
    etna_vk_destroy_surface(surf);
    etna_vk_destroy_instance(inst);
}
