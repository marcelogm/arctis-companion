use arctis_core::eq::library;
use arctis_core::eq::reference::{Overlays, Target};
use arctis_core::testkit::english::english;
use arctis_core::view::eq_graph::{ColorRole, LineStyle, Weight, graph_with};
use arctis_core::view::eq_legend::{
    ENTRY_GAP, MAX_ROWS, SWATCH_GAP, SWATCH_WIDTH, Slot, entry_width, legend, natural_width, rows,
    slots,
};

use crate::support::id;

const WIDTHS: [f32; 4] = [60.0, 110.0, 120.0, 70.0];

fn labels(overlays: Overlays) -> Vec<String> {
    let flat = library::find(&id("flat")).expect("flat");
    graph_with(&flat, overlays)
        .legend
        .iter()
        .map(|entry| english(&entry.label))
        .collect()
}

#[test]
fn the_legend_names_the_headset_first_and_the_equalizer_last() {
    let shown = Overlays {
        headset: true,
        target: Some(Target::DiffuseField),
    };
    assert_eq!(
        labels(shown),
        [
            "Headset",
            "Headset with EQ",
            "Diffuse field (B&K 5128)",
            "Equalizer"
        ]
    );
    assert_eq!(
        labels(Overlays::NONE.with_target_setting("harman")),
        ["Harman (B&K 5128)", "Equalizer"]
    );
    assert!(labels(Overlays::NONE).is_empty());
}

#[test]
fn legend_entries_draw_like_their_series() {
    let flat = library::find(&id("flat")).expect("flat");
    let model = graph_with(&flat, Overlays::default());
    let looks: Vec<_> = model
        .legend
        .iter()
        .map(|entry| (entry.style, entry.role, entry.weight, entry.filled))
        .collect();
    assert_eq!(
        looks,
        [
            (
                LineStyle::Solid,
                ColorRole::Foreground,
                Weight::Faint,
                false
            ),
            (LineStyle::Solid, ColorRole::Accent, Weight::Bold, false),
            (
                LineStyle::Dashed,
                ColorRole::Foreground,
                Weight::Regular,
                false
            ),
            (LineStyle::Solid, ColorRole::Accent, Weight::Faint, true),
        ]
    );
    assert_eq!(legend(&model.series), model.legend);
}

#[test]
fn an_entry_is_a_swatch_a_gap_and_its_label() {
    assert!((entry_width(50.0) - (SWATCH_WIDTH + SWATCH_GAP + 50.0)).abs() < 1e-3);
    let one_row: f32 = WIDTHS.iter().copied().map(entry_width).sum::<f32>() + 3.0 * ENTRY_GAP;
    assert!((natural_width(&WIDTHS) - one_row).abs() < 1e-3);
    assert!(natural_width(&[]).abs() < f32::EPSILON);
}

#[test]
fn a_wide_graph_fits_the_legend_on_one_row() {
    let placed = slots(&WIDTHS, 1_000.0).expect("fits");
    assert_eq!(rows(&placed), 1);
    assert_eq!(placed[0], Slot { x: 0.0, row: 0 });
    let second = entry_width(WIDTHS[0]) + ENTRY_GAP;
    assert!((placed[1].x - second).abs() < 1e-3);
}

#[test]
fn a_narrow_graph_wraps_the_legend_onto_a_second_row() {
    let placed = slots(&WIDTHS, 300.0).expect("fits on two rows");
    let rows_used: Vec<usize> = placed.iter().map(|slot| slot.row).collect();
    assert_eq!(rows_used, [0, 0, 1, 1]);
    assert!(placed[2].x.abs() < f32::EPSILON);
    assert_eq!(rows(&placed), 2);
    for slot in &placed {
        assert!(slot.row < MAX_ROWS);
    }
}

#[test]
fn a_legend_that_needs_more_rows_or_a_wider_entry_is_hidden() {
    assert_eq!(slots(&WIDTHS, 170.0), None);
    assert_eq!(slots(&WIDTHS, 100.0), None);
    assert_eq!(slots(&[], 100.0), Some(Vec::new()));
    assert_eq!(rows(&[]), 0);
}
