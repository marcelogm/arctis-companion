icons_dir = datadir / 'icons' / 'hicolor'

install_data(app_id + '.gschema.xml', install_dir: datadir / 'glib-2.0' / 'schemas')
install_data('dbus' / (app_id + '.Headset1.xml'), install_dir: datadir / 'dbus-1' / 'interfaces')
install_data(
  'icons' / 'hicolor' / 'scalable' / 'apps' / (app_id + '.svg'),
  install_dir: icons_dir / 'scalable' / 'apps',
)
install_data(
  'icons' / 'hicolor' / 'symbolic' / 'apps' / (app_id + '-symbolic.svg'),
  install_dir: icons_dir / 'symbolic' / 'apps',
)

configure_file(
  input: app_id + '.service.in',
  output: app_id + '.service',
  configuration: {'bindir': bindir},
  install_dir: datadir / 'dbus-1' / 'services',
)

desktop_file = i18n.merge_file(
  input: app_id + '.desktop.in',
  output: app_id + '.desktop',
  type: 'desktop',
  po_dir: meson.project_source_root() / 'po',
  install: true,
  install_dir: datadir / 'applications',
)

metainfo_file = i18n.merge_file(
  input: app_id + '.metainfo.xml.in',
  output: app_id + '.metainfo.xml',
  po_dir: meson.project_source_root() / 'po',
  install: true,
  install_dir: datadir / 'metainfo',
)

desktop_file_validate = find_program('desktop-file-validate', required: false)
if desktop_file_validate.found()
  test('validate-desktop', desktop_file_validate, args: [desktop_file])
endif

appstreamcli = find_program('appstreamcli', required: false)
if appstreamcli.found()
  test('validate-metainfo', appstreamcli, args: ['validate', '--no-net', '--explain', metainfo_file])
endif

glib_compile_schemas = find_program('glib-compile-schemas')
test('validate-schema', glib_compile_schemas, args: ['--strict', '--dry-run', meson.current_source_dir()])
