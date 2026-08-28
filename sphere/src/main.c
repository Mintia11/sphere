#include <stdint.h>
#include <mutex.h>
#include <alloc.h>
#include <log.h>
#include <vulkan/instance.h>
#include <SDL3/SDL.h>
#include <Windows.h>
#include <vulkan/surface.h>

int main() {
    etna_allocator_init_global();
    etna_logger_init_global();

    SDL_Init(SDL_INIT_VIDEO);
    SDL_Window* window =
        SDL_CreateWindow("Sphere", 1366, 768, SDL_WINDOW_VULKAN | SDL_WINDOW_RESIZABLE);

    etna_log_scope_t* main_log = etna_log_scope_new("main", NULL);

    ETNA_INFO(main_log, "Hello world!\n");
    etna_vk_instance* inst = etna_vk_create_instance(true);

    SDL_PropertiesID props = SDL_GetWindowProperties(window);
    HWND hwnd = (HWND)SDL_GetPointerProperty(props, SDL_PROP_WINDOW_WIN32_HWND_POINTER, NULL);
    HINSTANCE hinstance =
        (HINSTANCE)SDL_GetPointerProperty(props, SDL_PROP_WINDOW_WIN32_INSTANCE_POINTER, NULL);

    etna_vk_surface* surf = etna_vk_create_surface(
        &(etna_vk_surface_create_info){.hwnd = hwnd, .inst = inst, .hinstance = hinstance});

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
    }
}
