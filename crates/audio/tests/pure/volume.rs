use arctis_audio::volume::{Volume, VolumeChange, VolumeMirror, props_pod, volume_change};
use pipewire::spa::pod::Pod;

use crate::support::{BLUEZ, Scene};

fn volume(channels: &[f32], mute: bool) -> Volume {
    Volume {
        channels: channels.to_vec(),
        mute,
    }
}

fn change(channels: Option<&[f32]>, mute: Option<bool>) -> VolumeChange {
    VolumeChange {
        channels: channels.map(<[f32]>::to_vec),
        mute,
    }
}

fn read(bytes: &[u8]) -> VolumeChange {
    volume_change(Pod::from_bytes(bytes).expect("a pod"))
}

#[test]
fn a_props_pod_reads_back_as_the_same_change() {
    let bytes = props_pod(&volume(&[0.25, 0.5], true));
    assert_eq!(read(&bytes), change(Some(&[0.25, 0.5]), Some(true)));
}

#[test]
fn a_change_without_volume_or_mute_is_empty() {
    assert!(VolumeChange::default().is_empty());
    assert!(!read(&props_pod(&volume(&[], true))).is_empty());
}

#[test]
fn the_mirror_forwards_each_new_volume_once() {
    let mut mirror = VolumeMirror::default();
    assert_eq!(
        mirror.apply(change(Some(&[1.0, 1.0]), Some(false))),
        Some(volume(&[1.0, 1.0], false))
    );
    assert_eq!(mirror.apply(change(Some(&[1.0, 1.0]), Some(false))), None);
    assert_eq!(
        mirror.apply(change(None, Some(true))),
        Some(volume(&[1.0, 1.0], true))
    );
    assert_eq!(
        mirror.apply(change(Some(&[0.5, 0.5]), None)),
        Some(volume(&[0.5, 0.5], true))
    );
    assert_eq!(mirror.apply(change(None, None)), None);
}

#[test]
fn mute_alone_waits_for_channel_volumes() {
    let mut mirror = VolumeMirror::default();
    assert_eq!(mirror.apply(change(None, Some(true))), None);
    assert_eq!(
        mirror.apply(change(Some(&[0.75, 0.75]), None)),
        Some(volume(&[0.75, 0.75], true))
    );
}

#[test]
fn a_change_replaces_only_what_it_carries() {
    let base = volume(&[0.5, 0.5], false);
    assert_eq!(
        base.merged(&change(None, Some(true))),
        volume(&[0.5, 0.5], true)
    );
    assert_eq!(
        base.merged(&change(Some(&[0.25, 0.25]), None)),
        volume(&[0.25, 0.25], false)
    );
}

#[test]
fn a_sink_knows_its_volume_once_channel_volumes_arrive() {
    let mut graph = Scene::default().bluez(4).build();
    assert_eq!(graph.volume_of(BLUEZ), None);
    graph.change_volume(4, &change(None, Some(true)));
    assert_eq!(graph.volume_of(BLUEZ), None);
    graph.change_volume(4, &change(Some(&[0.3, 0.3]), None));
    assert_eq!(graph.volume_of(BLUEZ), Some(volume(&[0.3, 0.3], true)));
    assert_eq!(graph.volume_of("absent"), None);
}
