//! Behavioral checks for Material navigation templates: the selected page remains wired to the
//! real TabControl, responsive rail placement retains its content host, and dropdown fields keep
//! their selection/editing contract after replacing the Windows visual structure.

use crate::{register_types, MaterialTheme};
use ferroui_base::animation::{IClock, IGlobalClock, PlayState, TimeSpan};
use ferroui_base::controls::ResourceKey;
use ferroui_base::metadata::from_markup_value;
use ferroui_base::reactive::{IDisposable, IObservable, IObserver, LightweightSubject};
use ferroui_base::styling::ControlTheme;
use ferroui_base::{ObjectType, Ref, Visual};
use ferroui_controls::page::{ContentPage, TabPlacement, TabbedPage};
use ferroui_controls::testing::{MockWindowingPlatform, TestServices, UnitTestApplication, UnitTestApplicationScope};
use ferroui_controls::{Application, Border, ComboBox, ComboBoxItem, Control, Dock, TabControl, TabItem, Window};
use std::cell::Cell;
use std::rc::Rc;

struct FrozenClock {
    subject: LightweightSubject<TimeSpan>,
    state: Cell<PlayState>,
}

impl IObservable<TimeSpan> for FrozenClock {
    fn subscribe(&self, observer: Rc<dyn IObserver<TimeSpan>>) -> Rc<dyn IDisposable> {
        self.subject.subscribe(observer)
    }
}
impl IClock for FrozenClock {
    fn play_state(&self) -> PlayState { self.state.get() }
    fn set_play_state(&self, state: PlayState) { self.state.set(state); }
}
impl IGlobalClock for FrozenClock {}

fn app() -> UnitTestApplicationScope {
    register_types();
    UnitTestApplication::start(TestServices::styled_window()
        .with_global_clock(Rc::new(FrozenClock { subject: LightweightSubject::new(), state: Cell::new(PlayState::Run) }))
        .with_theme(|| MaterialTheme::new().as_style()))
}

fn named<T: ObjectType>(root: &Visual, name: &str) -> Ref<T> {
    root.get_visual_descendants()
        .filter(|node| node.name().as_deref() == Some(name))
        .find_map(|node| node.cast::<T>())
        .unwrap_or_else(|| panic!("required Material template part {name} is missing"))
}

fn window(content: impl Into<ferroui_base::BoxedValue>) -> Ref<Window> {
    let window = Window::with_impl(MockWindowingPlatform::create_window_mock_with_size(360.0, 640.0));
    window.set_content(Some(content.into()));
    window.show();
    window.update_layout();
    window
}

#[test]
fn bottom_navigation_shares_width_and_updates_selected_page() {
    let _app = app();
    let pages = TabbedPage::new();
    for header in ["Home", "Library", "Settings"] {
        let page = ContentPage::new();
        page.set_header(Some(Rc::new(header.to_owned())));
        pages.pages().expect("pages collection").add(page.upcast());
    }
    pages.set_selected_index(0);
    let window = window(Control::boxed(&pages));
    assert_eq!(pages.tab_placement(), TabPlacement::Bottom);
    let tabs: Ref<TabControl> = named(&pages, "PART_TabControl");
    assert_eq!(tabs.tab_strip_placement(), Dock::Bottom);
    let realized: Vec<Ref<TabItem>> = tabs.get_visual_descendants().filter_map(|node| node.cast()).collect();
    assert_eq!(realized.len(), 3);
    for item in &realized {
        assert!((item.bounds().width - 120.0).abs() <= 1.0, "navigation items share the 360dp viewport");
        assert!((item.bounds().height - 64.0).abs() <= 1.0, "Expressive navigation bar has a 64dp target");
    }
    tabs.set_selected_index(2);
    window.update_layout();
    assert_eq!(pages.selected_index(), 2, "template selection must reach the logical page controller");
    pages.set_selected_index(1);
    assert_eq!(tabs.selected_index(), 1, "programmatic page navigation must reach the visual bar");
    window.close();
}

#[test]
fn rail_theme_keeps_navigation_and_96dp_container_after_placement_changes() {
    let _app = app();
    let tabs = TabControl::new();
    let resource = Application::current().unwrap().try_get_resource(&ResourceKey::from("MaterialNavigationRail"), None).flatten();
    let theme = from_markup_value::<Ref<ControlTheme>>(&resource).expect("Material rail theme");
    tabs.set_theme(theme);
    for label in ["First", "Second"] {
        let tab = TabItem::new();
        tab.set_header(Some(Rc::new(label.to_string())));
        tabs.items().add(Some(Control::boxed(tab)));
    }
    tabs.set_selected_index(0);
    let window = window(Control::boxed(&tabs));
    let strip: Ref<Border> = named(&tabs, "TabContainerGrid");
    assert_eq!(tabs.tab_strip_placement(), Dock::Left);
    assert!((strip.bounds().width - 96.0).abs() <= 1.0);
    tabs.set_selected_index(1);
    window.update_layout();
    assert_eq!(tabs.selected_index(), 1);
    tabs.set_tab_strip_placement(Dock::Right);
    window.update_layout();
    assert!(strip.bounds().x > 0.0, "rail follows right placement without a mirrored content transform");
    assert_eq!(tabs.selected_index(), 1);
    window.close();
}

#[test]
fn exposed_dropdown_retains_selection_and_editable_text_box_parts() {
    let _app = app();
    let field = ComboBox::new();
    field.set_width(280.0);
    for label in ["First", "Second"] {
        let item = ComboBoxItem::new();
        item.set_content(Some(Rc::new(label.to_owned())));
        field.items().add(Some(Control::boxed(item)));
    }
    field.set_selected_index(1);
    let window = window(Control::boxed(&field));
    assert_eq!(field.selected_index(), 1);
    assert_eq!(field.min_height(), 56.0);
    field.set_is_editable(true);
    window.update_layout();
    let editor: Ref<ferroui_controls::TextBox> = named(&field, "PART_EditableTextBox");
    assert!(editor.is_visible());
    field.set_text(Some("search".to_owned()));
    window.update_layout();
    assert_eq!(editor.text().as_deref(), Some("search"));
    window.close();
}
