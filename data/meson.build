icons_dir = datadir / 'icons' / 'hicolor'

install_data(app_id + '.desktop', install_dir: datadir / 'applications')
install_data(app_id + '.metainfo.xml', install_dir: datadir / 'metainfo')
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

desktop_file_validate = find_program('desktop-file-validate', required: false)
if desktop_file_validate.found()
  test('validate-desktop', desktop_file_validate, args: [files(app_id + '.desktop')])
endif

appstreamcli = find_program('appstreamcli', required: false)
if appstreamcli.found()
  test('validate-metainfo', appstreamcli, args: ['validate', '--no-net', '--explain', files(app_id + '.metainfo.xml')])
endif

glib_compile_schemas = find_program('glib-compile-schemas')
test('validate-schema', glib_compile_schemas, args: ['--strict', '--dry-run', meson.current_source_dir()])
