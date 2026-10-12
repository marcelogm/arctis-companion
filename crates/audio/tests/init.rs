#[test]
fn initialising_pipewire_twice_is_harmless() {
    arctis_audio::init();
    arctis_audio::init();
}
