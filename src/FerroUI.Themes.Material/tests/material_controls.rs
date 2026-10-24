//! Behavioral/layout regressions for the Android component templates. These tests
//! deliberately check the arranged controls and template bindings, not XAML text.

use crate::{register_types, MaterialTheme};
use ferroui_base::animation::{IClock, IGlobalClock, PlayState, TimeSpan};
use ferroui_base::layout::{HorizontalAlignment, Orientation, VerticalAlignment};
use ferroui_base::reactive::{IDisposable, IObservable, IObserver, LightweightSubject};
use ferroui_base::{ObjectType, Ref, Visual};
use ferroui_controls::presenters::TextPresenter;
use ferroui_controls::primitives::{TemplatedControl, Track};
use ferroui_controls::testing::{TestServices, UnitTestApplication, UnitTestApplicationScope};
use ferroui_controls::{Border, Button, Canvas, CheckBox, Control, Panel, Slider, TextBox, ToggleSwitch, Window};
use std::cell::Cell;
use std::rc::Rc;

struct TestClock {
    subject: LightweightSubject<TimeSpan>,
    state: Cell<PlayState>,
}
impl IObservable<TimeSpan> for TestClock {
    fn subscribe(&self, observer: Rc<dyn IObserver<TimeSpan>>) -> Rc<dyn IDisposable> {
        self.subject.subscribe(observer)
    }
}
impl IClock for TestClock {
    fn play_state(&self) -> PlayState { self.state.get() }
    fn set_play_state(&self, value: PlayState) { self.state.set(value); }
}
impl IGlobalClock for TestClock {}

fn application() -> UnitTestApplicationScope {
    register_types();
    UnitTestApplication::start(
        TestServices::styled_window()
            .with_global_clock(Rc::new(TestClock { subject: LightweightSubject::new(), state: Cell::new(PlayState::Run) }))
            .with_theme(|| MaterialTheme::new().as_style()),
    )
}

fn host(control: &Control) -> Ref<Window> {
    let frame = Border::new();
    frame.set_horizontal_alignment(HorizontalAlignment::Left);
    frame.set_vertical_alignment(VerticalAlignment::Top);
    frame.set_child(control);
    let window = Window::new();
    window.set_width(480.0);
    window.set_height(320.0);
    window.set_content(Some(Control::boxed(&frame)));
    window.show();
    window.update_layout();
    window
}

fn named<T: ObjectType>(visual: &Visual, name: &str) -> Ref<T> {
    visual.get_visual_descendants()
        .filter(|node| node.name().as_deref() == Some(name))
        .find_map(|node| node.cast::<T>())
        .unwrap_or_else(|| panic!("Material template does not contain {name}"))
}

#[test]
fn material_button_preserves_48dp_target_around_40dp_surface() {
    let _app = application();
    let button = Button::new();
    button.set_content(Some(Rc::new("Save".to_string())));
    let _window = host(&button);
    let surface: Ref<Border> = named(&button, "MaterialContainer");
    assert!(button.bounds().height >= 48.0);
    assert_eq!(40.0, surface.bounds().height);
    assert_eq!(20.0, surface.corner_radius().top_left);
}

#[test]
fn checkbox_container_and_feedback_are_distinct_from_touch_target() {
    let _app = application();
    let check = CheckBox::new();
    let _window = host(&check);
    let outline: Ref<Border> = named(&check, "SelectionOutline");
    let layer: Ref<Border> = named(&check, "MaterialStateLayer");
    assert_eq!(18.0, outline.bounds().width);
    assert_eq!(18.0, outline.bounds().height);
    assert_eq!(40.0, layer.bounds().width);
    assert!(check.bounds().height >= 48.0);
}

#[test]
fn switch_retains_core_knob_travel_with_android_geometry() {
    let _app = application();
    let switch = ToggleSwitch::new();
    let window = host(&switch);
    let track: Ref<Border> = named(&switch, "MaterialSwitchTrack");
    let travel: Ref<Panel> = named(&switch, "PART_SwitchKnob");
    let moving: Ref<Panel> = named(&switch, "PART_MovingKnobs");
    let thumb: Ref<Border> = named(&switch, "MaterialSwitchThumb");
    assert_eq!(52.0, track.bounds().width);
    assert_eq!(32.0, track.bounds().height);
    assert_eq!(20.0, travel.bounds().width);
    assert_eq!(16.0, thumb.bounds().width);
    switch.set_knob_transitions(None);
    moving.set_transitions(None);
    thumb.set_transitions(None);
    switch.set_is_checked(Some(true));
    window.update_layout();
    assert_eq!(20.0, Canvas::get_left(&moving));
    assert_eq!(24.0, thumb.bounds().width);
}

#[test]
fn slider_keeps_value_binding_when_orientation_changes() {
    let _app = application();
    let slider = Slider::new();
    slider.set_width(240.0);
    slider.set_value(30.0);
    let window = host(&slider);
    let track: Ref<Track> = named(&slider, "PART_Track");
    let handle: Ref<Border> = named(&slider, "MaterialHandle");
    assert_eq!(30.0, track.value());
    assert_eq!(4.0, handle.bounds().width);
    assert_eq!(44.0, handle.bounds().height);
    slider.set_orientation(Orientation::Vertical);
    slider.set_width(48.0);
    slider.set_height(240.0);
    window.update_layout();
    let track: Ref<Track> = named(&slider, "PART_Track");
    let handle: Ref<Border> = named(&slider, "MaterialHandle");
    assert_eq!(30.0, track.value());
    assert_eq!(44.0, handle.bounds().width);
    assert_eq!(4.0, handle.bounds().height);
    slider.set_value(70.0);
    assert_eq!(70.0, track.value());
}

#[test]
fn material_text_field_keeps_editing_and_selection_template_contracts() {
    let _app = application();
    let field = TextBox::new();
    field.set_text(Some("Material"));
    let _window = host(&field);
    let surface: Ref<Border> = named(&field, "BorderElement");
    let presenter: Ref<TextPresenter> = named(&field, "PART_TextPresenter");
    assert_eq!(56.0, surface.bounds().height);
    assert_eq!(Some("Material".to_string()), presenter.text());
    field.set_text(Some("Edited"));
    assert_eq!(Some("Edited".to_string()), presenter.text());
    let templated: &TemplatedControl = &field;
    assert!(templated.template().is_some());
}
