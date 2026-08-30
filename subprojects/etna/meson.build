project(
    'etna',
    'c',
    version: '0.1.0',
    default_options: ['c_std=c11', 'warning_level=3'],
)

cc = meson.get_compiler('c')
vulkan_dep = dependency('vulkan')

cmake = import('cmake')

volk_options = cmake.subproject_options()
volk_options.add_cmake_defines(
    {
        'VOLK_STATIC_DEFINES': 'VK_USE_PLATFORM_WIN32_KHR',
    },
)

volk_proj = cmake.subproject('volk', options: volk_options)
volk_dep = volk_proj.dependency('volk')
volk_includes_dep = volk_dep.partial_dependency(compile_args: true, includes: true)

if get_option('buildtype') == 'debug'
    if cc.get_id() in ['clang', 'gcc']
        add_project_arguments('-fsanitize=address,undefined', language: 'c')
        add_project_link_arguments('-fsanitize=address,undefined', language: 'c')
    elif cc.get_id() == 'msvc'
        add_project_arguments('/fsanitize=address', language: 'c')
    endif
endif

etna_sources = files(
    'src/alloc.c',
    'src/log.c',
    'src/vulkan/alloc.c',
    'src/vulkan/cmdpool.c',
    'src/vulkan/device.c',
    'src/vulkan/instance.c',
    'src/vulkan/surface.c',
    'src/vulkan/swapchain.c',
)

etna_inc = include_directories('src')

etna_lib = static_library(
    'etna',
    etna_sources,
    dependencies: [vulkan_dep, volk_dep],
    include_directories: etna_inc,
    install: false,
)

etna_dep = declare_dependency(
    link_with: etna_lib,
    include_directories: etna_inc,
    dependencies: [vulkan_dep],
)