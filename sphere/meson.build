player_sources = files(
    'src/main.c',
)

sdl_dep = dependency('sdl3', default_options: ['default_library=static'])

executable(
    'sphere',
    player_sources,
    dependencies: [etna_dep, sdl_dep],
    include_directories: include_directories('src'),
)