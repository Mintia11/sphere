project(
    'sphere-mono',
    'c',
    version: '0.1.0',
    default_options: ['c_std=c23', 'warning_level=3', 'buildtype=debug'],
)

etna_proj = subproject('etna')
etna_dep = etna_proj.get_variable('etna_dep')

subdir('sphere')