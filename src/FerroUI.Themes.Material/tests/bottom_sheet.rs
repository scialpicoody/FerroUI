use super::support::start_themed_application;
use crate::{MaterialBottomSheetHost, MaterialSheetState};
use ferroui_base::input::{
    IPointer, InputElement, KeyModifiers, MouseButton, Pointer, PointerEventArgs,
    PointerPointProperties, PointerPressedEventArgs, PointerReleasedEventArgs, PointerType,
    PointerUpdateKind, RawInputModifiers,
};
use ferroui_base::{Point, Ref};
use ferroui_controls::{Border, Button, Control, Grid, ScrollViewer, Window};
use std::cell::Cell;
use std::rc::Rc;

fn host(
    sheet_height: f64,
) -> (
    Ref<Window>,
    Ref<MaterialBottomSheetHost>,
    Ref<Border>,
    Ref<Button>,
) {
    let window = Window::new();
    window.set_width(480.0);
    window.set_height(800.0);
    let grid = Grid::new();
    let trigger = Button::new();
    trigger.set_content(Some(Rc::new("Open".to_string())));
    grid.children().add(trigger.clone().upcast::<Control>());
    let sheet = MaterialBottomSheetHost::new();
    sheet.set_reduced_motion(true);
    let body = Border::new();
    body.set_height(sheet_height - 48.0);
    sheet.set_content(Some(Control::boxed(&body)));
    grid.children().add(sheet.clone().upcast::<Control>());
    window.set_content(Some(Control::boxed(&grid)));
    window.show();
    window.update_layout();
    (window, sheet, body, trigger)
}

#[test]
fn sheet_measures_real_content_and_back_visits_partial_before_dismiss() {
    let _app = start_themed_application();
    let (window, sheet, _, _) = host(600.0);
    assert!(!sheet.is_visible());
    assert!(sheet.show());
    window.update_layout();
    assert_eq!(MaterialSheetState::PartiallyExpanded, sheet.current_state());
    assert_eq!(400.0, sheet.anchors().partially_expanded);
    assert_eq!(200.0, sheet.anchors().expanded);
    assert_eq!(400.0, sheet.offset());
    assert!(sheet.expand());
    assert_eq!(200.0, sheet.offset());
    assert!(sheet.handle_back());
    assert_eq!(MaterialSheetState::PartiallyExpanded, sheet.current_state());
    assert!(sheet.handle_back());
    assert_eq!(MaterialSheetState::Hidden, sheet.current_state());
    assert!(!sheet.is_visible());
}

#[test]
fn short_sheet_remains_partial_then_expands_when_content_grows() {
    let _app = start_themed_application();
    let (window, sheet, body, _) = host(300.0);
    sheet.show();
    window.update_layout();
    assert_eq!(MaterialSheetState::PartiallyExpanded, sheet.state());
    assert_eq!(500.0, sheet.offset());
    body.set_height(552.0);
    window.update_layout();
    assert_eq!(MaterialSheetState::Expanded, sheet.state());
    assert_eq!(200.0, sheet.offset());
}

#[test]
fn dismissal_veto_and_completion_are_identity_safe() {
    let _app = start_themed_application();
    let (window, sheet, _, _) = host(600.0);
    let dismissed = Rc::new(Cell::new(0));
    let count = dismissed.clone();
    sheet.dismissed(move |_, _| count.set(count.get() + 1));
    sheet.show();
    window.update_layout();
    sheet.set_confirm_state_change(Some(Rc::new(|state| state != MaterialSheetState::Hidden)));
    assert!(!sheet.hide());
    assert_eq!(MaterialSheetState::PartiallyExpanded, sheet.state());
    assert_eq!(0, dismissed.get());
    sheet.set_confirm_state_change(None);
    assert!(sheet.hide());
    assert_eq!(1, dismissed.get());
    assert!(sheet.hide());
    assert_eq!(1, dismissed.get());
}

#[test]
fn sheet_restores_previous_focus_when_dismissed() {
    let _app = start_themed_application();
    let (window, sheet, _, trigger) = host(300.0);
    assert!(trigger.focus());
    sheet.show();
    window.update_layout();
    assert!(sheet.is_keyboard_focus_within());
    sheet.hide();
    assert!(trigger.is_focused());
}

#[test]
fn disabling_partial_state_retargets_expanded() {
    let _app = start_themed_application();
    let (window, sheet, _, _) = host(600.0);
    sheet.show();
    window.update_layout();
    sheet.set_is_partial_enabled(false);
    assert_eq!(MaterialSheetState::Expanded, sheet.state());
    assert_eq!(200.0, sheet.offset());
}

#[test]
fn host_is_not_retained_after_detachment_and_pending_layout() {
    let _app = start_themed_application();
    let (window, sheet, _, _) = host(600.0);
    sheet.show();
    window.update_layout();
    let weak = sheet.downgrade();
    window.set_content(None);
    window.update_layout();
    drop(sheet);
    window.close();
    ferroui_base::threading::Dispatcher::ui_thread().run_jobs(None);
    assert!(weak.upgrade().is_none());
}

fn touch_press(source: &Control, window: &Window, point: Point) -> Rc<Pointer> {
    let pointer = Pointer::new(Pointer::get_next_free_id(), PointerType::Touch, true);
    source.raise_event(&PointerPressedEventArgs::new(
        source.to_ref(),
        pointer.clone(),
        window,
        point,
        100,
        PointerPointProperties::new(
            RawInputModifiers::LEFT_MOUSE_BUTTON,
            PointerUpdateKind::LeftButtonPressed,
        ),
        KeyModifiers::NONE,
        1,
    ));
    pointer
}

fn touch_move(
    source: &Control,
    window: &Window,
    pointer: &Rc<Pointer>,
    point: Point,
    timestamp: u64,
) -> bool {
    let event = PointerEventArgs::new(
        Some(InputElement::pointer_moved_event()),
        source.to_ref(),
        pointer.clone(),
        Some(window),
        point,
        timestamp,
        PointerPointProperties::new(
            RawInputModifiers::LEFT_MOUSE_BUTTON,
            PointerUpdateKind::Other,
        ),
        KeyModifiers::NONE,
    );
    source.raise_event(&event);
    event.handled()
}

fn touch_release(source: &Control, window: &Window, pointer: &Rc<Pointer>, point: Point) {
    source.raise_event(&PointerReleasedEventArgs::new(
        source.to_ref(),
        pointer.clone(),
        window,
        point,
        500,
        PointerPointProperties::new(
            RawInputModifiers::NONE,
            PointerUpdateKind::LeftButtonReleased,
        ),
        KeyModifiers::NONE,
        MouseButton::Left,
    ));
}

#[test]
fn touch_scroll_expands_before_child_and_collapses_only_after_child_reaches_start() {
    let _app = start_themed_application();
    let (window, sheet, body, _) = host(1048.0);
    sheet.show();
    window.update_layout();
    let scroll = sheet
        .get_visual_descendants()
        .find_map(|visual| {
            (visual.name().as_deref() == Some("PART_ContentScroll"))
                .then(|| visual.cast::<ScrollViewer>())
                .flatten()
        })
        .unwrap();
    assert_eq!(400.0, sheet.offset());
    let pointer = touch_press(&body, &window, Point::new(100.0, 600.0));
    assert!(!touch_move(
        &body,
        &window,
        &pointer,
        Point::new(100.0, 598.0),
        110
    ));
    assert_eq!(400.0, sheet.offset());
    assert!(touch_move(
        &body,
        &window,
        &pointer,
        Point::new(100.0, 100.0),
        130
    ));
    assert_eq!(sheet.anchors().expanded, sheet.offset());
    assert!(
        scroll.offset().y > 50.0,
        "unconsumed upward motion must reach the child"
    );
    let child_offset = scroll.offset().y;
    assert!(touch_move(
        &sheet,
        &window,
        &pointer,
        Point::new(100.0, 100.0 + child_offset / 2.0),
        160
    ));
    assert_eq!(
        0.0,
        sheet.offset(),
        "the sheet must stay expanded while the child can scroll back"
    );
    assert!((scroll.offset().y - child_offset / 2.0).abs() < 0.001);
    assert!(touch_move(
        &sheet,
        &window,
        &pointer,
        Point::new(100.0, 300.0),
        190
    ));
    assert_eq!(0.0, scroll.offset().y);
    assert!(
        sheet.offset() > 80.0,
        "remaining downward motion must collapse the sheet"
    );
    touch_release(&sheet, &window, &pointer, Point::new(100.0, 300.0));
    assert!(pointer.captured().is_none());
    assert!(!sheet.classes().contains(":dragging"));
}

#[test]
fn horizontal_touch_is_left_to_the_child_and_detach_releases_a_sheet_capture() {
    let _app = start_themed_application();
    let (window, sheet, body, _) = host(600.0);
    sheet.show();
    window.update_layout();
    let pointer = touch_press(&body, &window, Point::new(100.0, 600.0));
    assert!(!touch_move(
        &body,
        &window,
        &pointer,
        Point::new(180.0, 602.0),
        130
    ));
    assert_eq!(400.0, sheet.offset());
    touch_release(&body, &window, &pointer, Point::new(180.0, 602.0));
    let pointer = touch_press(&body, &window, Point::new(100.0, 600.0));
    assert!(touch_move(
        &body,
        &window,
        &pointer,
        Point::new(100.0, 500.0),
        130
    ));
    assert!(pointer.captured().is_some());
    window.set_content(None);
    assert!(pointer.captured().is_none());
}

#[test]
fn persistent_expanded_sheet_cannot_drag_into_a_disabled_partial_anchor() {
    let _app = start_themed_application();
    let (window, sheet, body, _) = host(600.0);
    sheet.set_is_partial_enabled(false);
    sheet.set_is_dismiss_enabled(false);
    sheet.show();
    window.update_layout();
    let expanded = sheet.anchors().expanded;
    let pointer = touch_press(&body, &window, Point::new(100.0, 400.0));
    assert!(touch_move(
        &body,
        &window,
        &pointer,
        Point::new(100.0, 600.0),
        130
    ));
    assert_eq!(expanded, sheet.offset());
    touch_release(&sheet, &window, &pointer, Point::new(100.0, 600.0));
    assert_eq!(MaterialSheetState::Expanded, sheet.current_state());
}
