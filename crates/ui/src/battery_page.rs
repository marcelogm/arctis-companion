use std::rc::Rc;

use adw::prelude::*;
use adw::subclass::prelude::*;
use arctis_core::battery::sample::UnixSeconds;
use arctis_core::battery::series::Window as ChartWindow;
use arctis_core::companion::Snapshot;
use arctis_core::view::battery_chart::{BatteryContent, BatteryPageModel, battery_page};
use gtk::glib;

use crate::text::resolve;
use crate::widget_values::show_if;

const WEEK: &str = "week";
const EMPTY_PAGE: &str = "empty";
const CHART_PAGE: &str = "chart";

#[derive(Clone)]
pub struct Displayed {
    snapshot: Rc<Snapshot>,
    now: UnixSeconds,
}

mod imp {
    use adw::subclass::prelude::*;
    use gtk::glib;
    use gtk::prelude::StaticTypeExt;
    use std::cell::RefCell;

    use crate::battery_chart::BatteryChart;
    use crate::status_pages::StatusView;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/marcelogm/ArctisCompanion/ui/battery_page.ui")]
    pub struct BatteryPage {
        #[template_child]
        pub content_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub empty_view: TemplateChild<StatusView>,
        #[template_child]
        pub range_toggle: TemplateChild<adw::ToggleGroup>,
        #[template_child]
        pub chart: TemplateChild<BatteryChart>,
        #[template_child]
        pub last_full_charge: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub current_level: TemplateChild<adw::ActionRow>,
        pub displayed: RefCell<Option<super::Displayed>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for BatteryPage {
        const NAME: &'static str = "ArctisBatteryPage";
        type Type = super::BatteryPage;
        type ParentType = adw::NavigationPage;

        fn class_init(klass: &mut Self::Class) {
            StatusView::ensure_type();
            BatteryChart::ensure_type();
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for BatteryPage {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().redraw_on_range_change();
        }
    }
    impl WidgetImpl for BatteryPage {}
    impl NavigationPageImpl for BatteryPage {}
}

glib::wrapper! {
    pub struct BatteryPage(ObjectSubclass<imp::BatteryPage>)
        @extends adw::NavigationPage, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl Default for BatteryPage {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl BatteryPage {
    pub fn display_history(&self, snapshot: Rc<Snapshot>, now: UnixSeconds) {
        self.imp()
            .displayed
            .replace(Some(Displayed { snapshot, now }));
        self.redraw();
    }

    fn redraw_on_range_change(&self) {
        let page = self.downgrade();
        self.imp()
            .range_toggle
            .connect_active_name_notify(move |_| {
                page.upgrade().inspect(Self::redraw);
            });
    }

    fn range(&self) -> ChartWindow {
        match self.imp().range_toggle.active_name().as_deref() {
            Some(WEEK) => ChartWindow::Week,
            _ => ChartWindow::Day,
        }
    }

    fn redraw(&self) {
        let displayed = self.imp().displayed.borrow().clone();
        if let Some(Displayed { snapshot, now }) = displayed {
            self.render(&battery_page(&snapshot, self.range(), now));
        }
    }

    fn render(&self, model: &BatteryPageModel) {
        let imp = self.imp();
        match &model.content {
            BatteryContent::Empty(empty) => {
                imp.empty_view.render(empty, None);
                imp.content_stack.set_visible_child_name(EMPTY_PAGE);
            }
            BatteryContent::Chart(chart) => {
                imp.chart.render(chart);
                imp.content_stack.set_visible_child_name(CHART_PAGE);
            }
        }
        imp.last_full_charge
            .set_subtitle(&resolve(&model.last_full_charge));
        show_if(
            &*imp.current_level,
            model.current_level.as_ref(),
            |row, level| {
                row.set_subtitle(&resolve(level));
            },
        );
    }
}
