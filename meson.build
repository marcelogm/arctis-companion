project(
  'arctis-companion',
  version: '0.1.0',
  meson_version: '>= 1.5.0',
  license: 'GPL-3.0-or-later',
)

gnome = import('gnome')

app_id = 'io.github.marcelogm.ArctisCompanion'
bindir = get_option('prefix') / get_option('bindir')
datadir = get_option('prefix') / get_option('datadir')

subdir('data')

cargo = find_program('cargo')
cargo_target_dir = meson.project_build_root() / 'target'

custom_target(
  'cargo-build',
  build_by_default: true,
  build_always_stale: true,
  output: meson.project_name(),
  console: true,
  install: true,
  install_dir: bindir,
  command: [
    cargo, 'build', '--release',
    '--package', 'arctis-app',
    '--manifest-path', meson.project_source_root() / 'Cargo.toml',
    '--target-dir', cargo_target_dir,
    '&&',
    'cp', cargo_target_dir / 'release' / meson.project_name(), '@OUTPUT@',
  ],
)

gnome.post_install(
  glib_compile_schemas: true,
  gtk_update_icon_cache: true,
  update_desktop_database: true,
)
