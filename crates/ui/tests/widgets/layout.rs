use adw::prelude::*;
use arctis_core::companion::routing::Route;
use arctis_core::eq::FilterKind;
use arctis_core::messages::device::Phase;
use arctis_core::testkit::snapshots::{snapshot, with};
use arctis_ui::PermissionDialog;

use crate::app_presets_page::rendered_page;
use crate::battery_history::{shown_battery_page, synthetic_week, with_history};
use crate::support::{
    custom_preset, descendants, filter, named, one_peak, open_editor, open_page, open_presets,
    rendered_window, rich_snapshot, ruled, seen_app, shown, standalone_editor, with_preset,
};

const NARROWEST: i32 = 360;

fn minimum_width(widget: &impl IsA<gtk::Widget>) -> i32 {
    widget.measure(gtk::Orientation::Horizontal, -1).0
}

#[gtk::test]
fn the_headset_page_fits_the_narrowest_window() {
    let (window, _) = rendered_window(rich_snapshot());
    let content = window.content().expect("window content");
    assert!(
        minimum_width(&content) <= NARROWEST,
        "{}",
        minimum_width(&content)
    );
}

#[gtk::test]
fn the_status_pages_fit_the_narrowest_window() {
    for phase in [Phase::Absent, Phase::PermissionDenied] {
        let (window, _) = rendered_window(with(phase, Route::Dongle));
        let content = window.content().expect("window content");
        assert!(minimum_width(&content) <= NARROWEST, "{phase:?}");
    }
}

#[gtk::test]
fn the_dialogs_fit_the_narrowest_window() {
    let (window, _) = rendered_window(snapshot());
    assert!(minimum_width(&window.preferences_dialog()) <= NARROWEST);
    assert!(minimum_width(&PermissionDialog::default()) <= NARROWEST);
}

fn assert_never_scrolls_horizontally(root: &impl IsA<gtk::Widget>, what: &str) {
    let scrolled: Vec<gtk::ScrolledWindow> = descendants(root)
        .into_iter()
        .filter(shown)
        .filter_map(|widget| widget.downcast::<gtk::ScrolledWindow>().ok())
        .collect();
    assert!(!scrolled.is_empty(), "{what} scrolls vertically");
    for view in scrolled {
        assert_eq!(view.hscrollbar_policy(), gtk::PolicyType::Never);
    }
}

#[gtk::test]
fn the_app_presets_page_and_dialog_fit_the_narrowest_window() {
    let mut s = ruled(&[("org.mozilla.firefox", "footsteps"), ("mpv", "gone")]);
    s.presets.seen_apps = vec![seen_app("Spotify", &["com.spotify.Client", "spotify"])];
    let (page, _, _) = rendered_page(&s);
    assert!(
        minimum_width(&page) <= NARROWEST,
        "{}",
        minimum_width(&page)
    );
    let dialog = page.add_rule_dialog();
    assert!(
        minimum_width(&dialog) <= NARROWEST,
        "{}",
        minimum_width(&dialog)
    );
}
fn content_width(window: &arctis_ui::Window) -> i32 {
    minimum_width(&window.content().expect("window content"))
}

fn widest(window: &arctis_ui::Window) -> String {
    descendants(window)
        .iter()
        .filter(|widget| shown(widget))
        .map(|widget| (minimum_width(widget), widget.type_().name()))
        .filter(|(width, _)| *width > NARROWEST - 60)
        .map(|(width, name)| format!("{name} {width}"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn shown_scrolled_windows(window: &arctis_ui::Window) -> Vec<gtk::ScrolledWindow> {
    descendants(window)
        .into_iter()
        .filter(shown)
        .filter_map(|widget| widget.downcast::<gtk::ScrolledWindow>().ok())
        .collect()
}

#[gtk::test]
fn the_preset_pages_fit_the_narrowest_window() {
    let mine = custom_preset(&[filter(FilterKind::Peaking, 250, 4, 1_000)]);
    let (window, _) = rendered_window(with_preset(&mine));
    open_presets(&window);
    assert!(
        content_width(&window) <= NARROWEST,
        "{}",
        content_width(&window)
    );
    for preset in ["mine", "flat"] {
        open_editor(&window, preset);
        assert!(
            content_width(&window) <= NARROWEST,
            "{preset}: {} ({})",
            content_width(&window),
            widest(&window)
        );
    }
}

#[gtk::test]
fn the_editor_never_scrolls_horizontally() {
    let (window, _) = rendered_window(rich_snapshot());
    open_editor(&window, "flat");
    let scrolled = shown_scrolled_windows(&window);
    assert!(!scrolled.is_empty(), "the band list scrolls vertically");
    for view in scrolled {
        assert_eq!(view.hscrollbar_policy(), gtk::PolicyType::Never);
    }
}

#[gtk::test]
fn nothing_shown_scrolls_horizontally() {
    let (window, _) = rendered_window(rich_snapshot());
    assert_never_scrolls_horizontally(&window, "the headset page");
}

#[gtk::test]
fn the_battery_page_fits_the_narrowest_window() {
    for s in [snapshot(), with_history(synthetic_week())] {
        let page = shown_battery_page(s);
        assert!(
            minimum_width(&page) <= NARROWEST,
            "{}",
            minimum_width(&page)
        );
    }
}

#[gtk::test]
fn the_opened_battery_page_fits_the_narrowest_window() {
    let (window, _) = rendered_window(with_history(synthetic_week()));
    open_page(&window, "battery");
    assert!(
        content_width(&window) <= NARROWEST,
        "{}",
        content_width(&window)
    );
}

#[gtk::test]
fn the_battery_page_never_scrolls_horizontally() {
    let page = shown_battery_page(with_history(synthetic_week()));
    assert_never_scrolls_horizontally(&page, "the battery page");
}

const CONTENT_MAX: f32 = 720.0;

fn bounds_in(widget: &gtk::Widget, page: &impl IsA<gtk::Widget>) -> gtk::graphene::Rect {
    widget.compute_bounds(page).expect("an allocated widget")
}

fn allocated_editor(width: i32) -> arctis_ui::EqEditorPage {
    let editor = standalone_editor(with_preset(&one_peak()), "mine");
    let page = editor.page;
    page.measure(gtk::Orientation::Horizontal, -1);
    page.measure(gtk::Orientation::Vertical, width);
    page.allocate(width, 1000, -1, None);
    page
}

#[gtk::test]
fn a_wide_editor_keeps_one_centered_column() {
    let page = allocated_editor(1920);
    let graph = bounds_in(&named::<gtk::Widget>(&page, "response-graph"), &page);
    let bands = bounds_in(&named::<gtk::Widget>(&page, "bands"), &page);
    for (what, rect) in [("graph", &graph), ("bands", &bands)] {
        assert!(
            rect.width() <= CONTENT_MAX,
            "{what} is {} wide",
            rect.width()
        );
        let center = rect.x() + rect.width() / 2.0;
        assert!(
            (center - 960.0).abs() <= 1.0,
            "{what} is centered at {center}"
        );
    }
    assert!(
        graph.y() + graph.height() <= bands.y(),
        "the graph sits above the bands"
    );
}

#[gtk::test]
fn a_narrow_editor_spends_its_width_on_the_graph_and_the_bands() {
    let page = allocated_editor(440);
    for name in ["response-graph", "bands"] {
        let rect = bounds_in(&named::<gtk::Widget>(&page, name), &page);
        assert!(rect.width() >= 400.0, "{name} is {} wide", rect.width());
    }
}
