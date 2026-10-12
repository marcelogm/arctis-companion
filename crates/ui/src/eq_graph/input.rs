use adw::prelude::*;
use adw::subclass::prelude::*;
use arctis_core::l10n::msgid;
use arctis_core::view::eq_plot::{Nudge, nudged, scrolled_q};
use gtk::{gdk, gio, glib};

use super::{EqGraph, GraphAction};
use crate::eq_handle::{EqHandle, HANDLE_SIZE, pixels};
use crate::text::translated;

const DOUBLE_CLICK: i32 = 2;
const GRAB_CURSOR: &str = "grab";
const GRABBING_CURSOR: &str = "grabbing";

enum KeyMeaning {
    Nudge(Nudge),
    Remove,
}

impl EqGraph {
    pub(super) fn connect_input(&self) {
        self.connect_drag();
        self.connect_clicks();
        self.connect_pointer();
    }

    pub(super) fn new_handle(&self, index: usize) -> EqHandle {
        let handle = EqHandle::default();
        handle.set_widget_name("eq-handle");
        let keys = gtk::EventControllerKey::new();
        keys.connect_key_pressed(glib::clone!(
            #[weak(rename_to = graph)]
            self,
            #[upgrade_or]
            glib::Propagation::Proceed,
            move |_, key, _, _| graph.key_pressed(index, key)
        ));
        handle.add_controller(keys);
        handle
    }

    pub fn hover_at(&self, x: f32, y: f32) {
        self.imp().pointer.set(Some((x, y)));
        let over = self.imp().editable.get() && self.hit(x, y).is_some();
        self.set_cursor_from_name(over.then_some(GRAB_CURSOR));
    }

    #[must_use]
    pub fn scroll_by(&self, steps: f64) -> bool {
        let imp = self.imp();
        let Some((x, y)) = imp.pointer.get() else {
            return false;
        };
        let target = self.hit(x, y).filter(|_| imp.editable.get());
        let Some(handle) = target.and_then(|index| self.handle_model(index)) else {
            return false;
        };
        let edit = scrolled_q(handle.filter, steps);
        self.emit(GraphAction::Commit(handle.index, edit));
        true
    }

    pub fn context_menu_at(&self, x: f32, y: f32) {
        let Some(index) = self.hit(x, y) else {
            return;
        };
        self.pick(index);
        if !self.imp().editable.get() {
            return;
        }
        let menu = self.menu();
        let half = pixels(HANDLE_SIZE) / 2.0;
        menu.set_pointing_to(Some(&gdk::Rectangle::new(
            whole(x - half),
            whole(y - half),
            HANDLE_SIZE,
            HANDLE_SIZE,
        )));
        menu.popup();
    }

    fn key_pressed(&self, index: usize, key: gdk::Key) -> glib::Propagation {
        let meaning = key_meaning(key);
        let Some(meaning) = meaning.filter(|_| self.imp().editable.get()) else {
            return glib::Propagation::Proceed;
        };
        let action = match meaning {
            KeyMeaning::Remove => Some(GraphAction::Remove(index)),
            KeyMeaning::Nudge(nudge) => self
                .handle_model(index)
                .and_then(|handle| nudged(handle.filter, nudge))
                .map(|edit| GraphAction::Commit(index, edit)),
        };
        self.select(Some(index));
        if let Some(action) = action {
            self.emit(action);
        }
        glib::Propagation::Stop
    }

    fn menu(&self) -> gtk::PopoverMenu {
        self.imp()
            .menu
            .get_or_init(|| {
                let model = gio::Menu::new();
                model.append(
                    Some(&translated(msgid("_Remove Band"))),
                    Some("graph.remove-band"),
                );
                let menu = gtk::PopoverMenu::from_model(Some(&model));
                menu.set_has_arrow(false);
                menu.set_parent(self);
                menu
            })
            .clone()
    }

    fn connect_drag(&self) {
        let drag = gtk::GestureDrag::new();
        drag.connect_drag_begin(glib::clone!(
            #[weak(rename_to = graph)]
            self,
            move |_, x, y| graph.begin_drag(x, y)
        ));
        drag.connect_drag_update(glib::clone!(
            #[weak(rename_to = graph)]
            self,
            move |_, dx, dy| graph.drag_by(narrow(dx), narrow(dy))
        ));
        drag.connect_drag_end(glib::clone!(
            #[weak(rename_to = graph)]
            self,
            move |_, dx, dy| graph.end_drag(narrow(dx), narrow(dy))
        ));
        self.add_controller(drag);
    }

    fn begin_drag(&self, x: f64, y: f64) {
        self.press_at(narrow(x), narrow(y));
        if self.imp().drag.get().is_some() {
            self.set_cursor_from_name(Some(GRABBING_CURSOR));
        }
    }

    fn end_drag(&self, dx: f32, dy: f32) {
        self.release_by(dx, dy);
        self.set_cursor_from_name(None);
    }

    fn connect_clicks(&self) {
        let primary = gtk::GestureClick::new();
        primary.connect_pressed(glib::clone!(
            #[weak(rename_to = graph)]
            self,
            move |_, presses, x, y| {
                if presses == DOUBLE_CLICK {
                    graph.double_click_at(narrow(x), narrow(y));
                }
            }
        ));
        self.add_controller(primary);
        let secondary = gtk::GestureClick::new();
        secondary.set_button(gdk::BUTTON_SECONDARY);
        secondary.connect_pressed(glib::clone!(
            #[weak(rename_to = graph)]
            self,
            move |_, _, x, y| graph.context_menu_at(narrow(x), narrow(y))
        ));
        self.add_controller(secondary);
    }

    fn connect_pointer(&self) {
        let motion = gtk::EventControllerMotion::new();
        motion.connect_motion(glib::clone!(
            #[weak(rename_to = graph)]
            self,
            move |_, x, y| graph.hover_at(narrow(x), narrow(y))
        ));
        motion.connect_leave(glib::clone!(
            #[weak(rename_to = graph)]
            self,
            move |_| graph.imp().pointer.set(None)
        ));
        self.add_controller(motion);
        let flags =
            gtk::EventControllerScrollFlags::VERTICAL | gtk::EventControllerScrollFlags::DISCRETE;
        let scroll = gtk::EventControllerScroll::new(flags);
        scroll.connect_scroll(glib::clone!(
            #[weak(rename_to = graph)]
            self,
            #[upgrade_or]
            glib::Propagation::Proceed,
            move |_, _, dy| {
                if graph.scroll_by(-dy) {
                    glib::Propagation::Stop
                } else {
                    glib::Propagation::Proceed
                }
            }
        ));
        self.add_controller(scroll);
    }
}

fn key_meaning(key: gdk::Key) -> Option<KeyMeaning> {
    let nudge = match key {
        gdk::Key::Left | gdk::Key::KP_Left => Nudge::Lower,
        gdk::Key::Right | gdk::Key::KP_Right => Nudge::Higher,
        gdk::Key::Up | gdk::Key::KP_Up => Nudge::Up,
        gdk::Key::Down | gdk::Key::KP_Down => Nudge::Down,
        gdk::Key::Page_Up | gdk::Key::plus | gdk::Key::KP_Add | gdk::Key::equal => Nudge::Narrower,
        gdk::Key::Page_Down | gdk::Key::minus | gdk::Key::KP_Subtract => Nudge::Wider,
        gdk::Key::Delete | gdk::Key::KP_Delete | gdk::Key::BackSpace => {
            return Some(KeyMeaning::Remove);
        }
        _ => return None,
    };
    Some(KeyMeaning::Nudge(nudge))
}

#[expect(clippy::cast_possible_truncation)]
fn whole(value: f32) -> i32 {
    value.round() as i32
}

#[expect(clippy::cast_possible_truncation)]
fn narrow(value: f64) -> f32 {
    value.clamp(f64::from(f32::MIN), f64::from(f32::MAX)) as f32
}
