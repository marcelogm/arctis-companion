i18n.gettext(gettext_package, preset: 'glib')

msgfmt = find_program('msgfmt')
test('check-pt_BR', msgfmt, args: ['--check', '--output-file=/dev/null', files('pt_BR.po')])
