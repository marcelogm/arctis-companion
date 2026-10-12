mod draw;
mod input;

use std::rc::Rc;

use adw::prelude::*;
use adw::subclass::prelude::*;
use arctis_core::view::eq_graph::{GraphModel, Handle, LevelRange};
use arctis_core::view::eq_plot::{Plot, drag_edit, hit_handle};
use arctis_core::view::preset_edits::BandEdit;
use gtk::accessible::Property;
use gtk::{glib, graphene, gsk};

use crate::eq_handle::{EqHandle, HANDLE_SIZE, pixels};
use crate::text::resolve;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GraphAction {
    Select(usize),
    Preview(usize, BandEdit),
    Commit(usize, BandEdit),
    Add { hz: f64, db: f64 },
    Remove(usize),
}

type ActionHandler = Rc<dyn Fn(GraphAction)>;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Drag {
    index: usize,
    start: (f32, f32),
}

mod imp {
    use std::cell::{Cell, OnceCell, RefCell};

    use adw::subclass::prelude::*;
    use arctis_core::view::eq_graph::GraphModel;
    use gtk::glib;
    use gtk::prelude::*;

    use crate::eq_handle::EqHandle;

    #[derive(Default)]
    pub struct EqGraph {
        pub model: RefCell<Option<GraphModel>>,
        pub editable: Cell<bool>,
        pub selected: Cell<Option<usize>>,
        pub handles: RefCell<Vec<EqHandle>>,
        pub drag: Cell<Option<super::Drag>>,
        pub pointer: Cell<Option<(f32, f32)>>,
        pub on_action: RefCell<Option<super::ActionHandler>>,
        pub menu: OnceCell<gtk::PopoverMenu>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for EqGraph {
        const NAME: &'static str = "ArctisEqGraph";
        type Type = super::EqGraph;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.set_css_name("eq-graph");
            klass.set_accessible_role(gtk::AccessibleRole::Group);
            klass.install_action("graph.remove-band", None, |graph, _, _| {
                graph.remove_selected();
            });
        }
    }

    impl ObjectImpl for EqGraph {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().connect_input();
        }

        fn dispose(&self) {
            self.obj().release_children();
        }
    }

    impl WidgetImpl for EqGraph {
        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            let graph = self.obj();
            graph.draw(snapshot, graph.width(), graph.height());
            graph.snapshot_handles(snapshot);
        }

        fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
            self.parent_size_allocate(width, height, baseline);
            self.obj().place_handles(width, height);
        }
    }
}

glib::wrapper! {
    pub struct EqGraph(ObjectSubclass<imp::EqGraph>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl Default for EqGraph {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl EqGraph {
    pub fn render(&self, model: &GraphModel, editable: bool) {
        let imp = self.imp();
        imp.editable.set(editable);
        self.update_property(&[Property::Description(&resolve(&model.description))]);
        self.sync_handles(&model.handles);
        imp.model.replace(Some(model.clone()));
        self.queue_allocate();
        self.queue_draw();
    }

    pub fn connect_action(&self, handler: impl Fn(GraphAction) + 'static) {
        self.imp().on_action.replace(Some(Rc::new(handler)));
    }

    pub fn select(&self, index: Option<usize>) {
        self.imp().selected.set(index);
        for (at, handle) in self.handles().iter().enumerate() {
            handle.set_selected(Some(at) == index);
        }
    }

    #[must_use]
    pub fn selected(&self) -> Option<usize> {
        self.imp().selected.get()
    }

    #[must_use]
    pub fn handles(&self) -> Vec<EqHandle> {
        self.imp().handles.borrow().clone()
    }

    #[must_use]
    pub fn plot(&self) -> Plot {
        Plot::within(pixels(self.width()), pixels(self.height()), self.levels())
    }

    pub fn press_at(&self, x: f32, y: f32) {
        let Some(index) = self.hit(x, y) else {
            return;
        };
        self.pick(index);
        let imp = self.imp();
        if imp.editable.get() {
            let start = self.handle_centre(index).unwrap_or((x, y));
            imp.drag.set(Some(Drag { index, start }));
        }
    }

    pub fn drag_by(&self, dx: f32, dy: f32) {
        if let Some(drag) = self.imp().drag.get() {
            let edit = drag_edit(&self.plot(), drag.start, (dx, dy));
            self.emit(GraphAction::Preview(drag.index, edit));
        }
    }

    pub fn release_by(&self, dx: f32, dy: f32) {
        let Some(drag) = self.imp().drag.take() else {
            return;
        };
        if dx != 0.0 || dy != 0.0 {
            let edit = drag_edit(&self.plot(), drag.start, (dx, dy));
            self.emit(GraphAction::Commit(drag.index, edit));
        }
    }

    pub fn double_click_at(&self, x: f32, y: f32) {
        if !self.imp().editable.get() || self.hit(x, y).is_some() {
            return;
        }
        let plot = self.plot();
        self.emit(GraphAction::Add {
            hz: f64::from(plot.hz_at(x)),
            db: f64::from(plot.db_at(y)),
        });
    }

    fn snapshot_handles(&self, snapshot: &gtk::Snapshot) {
        for handle in self.imp().handles.borrow().iter() {
            self.snapshot_child(handle, snapshot);
        }
    }

    fn release_children(&self) {
        for handle in self.imp().handles.take() {
            handle.unparent();
        }
        if let Some(menu) = self.imp().menu.get() {
            menu.unparent();
        }
    }

    fn levels(&self) -> LevelRange {
        let model = self.imp().model.borrow();
        model
            .as_ref()
            .map_or(LevelRange::RELATIVE, |model| model.levels)
    }

    fn hit(&self, x: f32, y: f32) -> Option<usize> {
        let model = self.imp().model.borrow();
        let handles = model.as_ref().map(|model| model.handles.as_slice())?;
        hit_handle(&self.plot(), handles, x, y)
    }

    fn handle_model(&self, index: usize) -> Option<Handle> {
        let model = self.imp().model.borrow();
        model.as_ref()?.handles.get(index).cloned()
    }

    fn handle_centre(&self, index: usize) -> Option<(f32, f32)> {
        let handle = self.handle_model(index)?;
        let plot = self.plot();
        Some((plot.x_of(handle.hz), plot.y_of(handle.db)))
    }

    fn pick(&self, index: usize) {
        self.select(Some(index));
        if let Some(handle) = self.handles().get(index) {
            handle.grab_focus();
        }
        self.emit(GraphAction::Select(index));
    }

    fn remove_selected(&self) {
        if let (true, Some(index)) = (self.imp().editable.get(), self.selected()) {
            self.emit(GraphAction::Remove(index));
        }
    }

    fn emit(&self, action: GraphAction) {
        let handler = self.imp().on_action.borrow().clone();
        if let Some(handler) = handler {
            handler(action);
        }
    }

    fn sync_handles(&self, models: &[Handle]) {
        let imp = self.imp();
        let mut handles = imp.handles.borrow_mut();
        let keep = models.len().min(handles.len());
        for handle in handles.drain(keep..) {
            handle.unparent();
        }
        while handles.len() < models.len() {
            let handle = self.new_handle(handles.len());
            handle.set_parent(self);
            handles.push(handle);
        }
        let selected = imp.selected.get().filter(|index| *index < models.len());
        imp.selected.set(selected);
        for (handle, model) in handles.iter().zip(models) {
            handle.render(model, selected == Some(model.index));
        }
    }

    fn place_handles(&self, width: i32, height: i32) {
        let plot = Plot::within(pixels(width), pixels(height), self.levels());
        let model = self.imp().model.borrow();
        let Some(model) = model.as_ref() else {
            return;
        };
        for (handle, at) in self.handles().iter().zip(&model.handles) {
            handle.measure(gtk::Orientation::Horizontal, -1);
            let half = pixels(HANDLE_SIZE) / 2.0;
            let corner = graphene::Point::new(plot.x_of(at.hz) - half, plot.y_of(at.db) - half);
            let place = gsk::Transform::new().translate(&corner);
            handle.allocate(HANDLE_SIZE, HANDLE_SIZE, -1, Some(place));
        }
    }
}
