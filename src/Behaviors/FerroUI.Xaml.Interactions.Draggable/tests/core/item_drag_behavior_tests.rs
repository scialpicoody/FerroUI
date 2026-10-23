//! Ported from `tests/Xaml.Behaviors.Interactions.UnitTests/Core/ItemDragBehaviorTests.cs` of
//! Xaml.Behaviors (with its window `ListReorderDragBehaviorWindow`,
//! `ListReorderDragBehaviorWindow.xaml` here, loaded with the run-time loader).
//!
//! Adapted: the mouse raises the pointer events on the container (`support.rs`); the
//! `ObservableCollection<int>` of the reference is the observable collection of the framework.

// Not ported: ItemDragBehavior_Reorders_Vertical - skipped in the reference ("Drag not supported in
// headless environment"); the reorder is tested without a drag session in `draggable_tests.rs`.
// Not ported: ItemDragBehavior_Reorders_Horizontal - skipped in the reference ("Drag not supported
// in headless environment"); the reorder is tested without a drag session in `draggable_tests.rs`.

use crate::tests::support::{capture_rendered_frame, find, load_window, object, run_jobs, start_app, Mouse};
use crate::ListReorderDragBehavior;
use ferroui_base::{Point, Ref};
use ferroui_controls::{ListBox, ListBoxItem, Window};
use ferroui_controls::ItemsSource;
use ferroui_system::collections::object_model::ObservableCollection;
use ferroui_xaml_interactivity::Interaction;
use std::rc::Rc;

fn list_reorder_drag_behavior_window() -> Ref<Window> {
    load_window("ListReorderDragBehaviorWindow.xaml", include_str!("ListReorderDragBehaviorWindow.xaml"))
}

fn assert_drag_can_start(window: &Window, list_box: &ListBox, index: i32) {
    let container = list_box
        .container_from_index(index)
        .and_then(|container| container.cast::<ListBoxItem>())
        .expect("a list box item");
    let behaviors = container.get_value(Interaction::behaviors_property()).expect("a behavior collection");
    assert_eq!(1, behaviors.count());
    let behavior = behaviors.get(0).cast::<ListReorderDragBehavior>().expect("a list reorder drag behavior");

    assert_eq!(Some(object(&container)), behaviors.associated_object());
    assert_eq!(Some(object(&container)), behavior.associated_object());

    list_box.set_selected_index(index);

    let mouse = Mouse::new();
    mouse.down(&container, Point::new(5.0, 5.0));

    assert!(container.classes().contains(":dragging"));

    mouse.up(&container, Point::new(5.0, 5.0));

    assert!(!container.classes().contains(":dragging"));
    let _ = window;
}

fn show(window: &Window) {
    window.show();
    run_jobs();
    capture_rendered_frame(window);
}

#[test]
fn list_reorder_drag_behavior_can_start_again_after_virtualized_item_moves() {
    let _app = start_app();
    let items = ObservableCollection::from_items((0..100).collect::<Vec<i32>>());
    let window = list_reorder_drag_behavior_window();
    let list_box = find::<ListBox>(&window, "TargetListBox");
    list_box.set_items_source(Some(ItemsSource::new(Rc::new(items.clone()))));

    show(&window);

    assert_drag_can_start(&window, &list_box, 0);

    let moved_item = items.get(0);
    items.move_item(0, 5);
    run_jobs();
    capture_rendered_frame(&window);

    assert_eq!(moved_item, items.get(5));
    assert_drag_can_start(&window, &list_box, 5);
}

#[test]
fn list_reorder_drag_behavior_can_start_after_list_reattaches() {
    let _app = start_app();
    let items = ObservableCollection::from_items((0..100).collect::<Vec<i32>>());
    let window = list_reorder_drag_behavior_window();
    let list_box = find::<ListBox>(&window, "TargetListBox");
    list_box.set_items_source(Some(ItemsSource::new(Rc::new(items))));

    show(&window);

    assert_drag_can_start(&window, &list_box, 0);

    window.set_content(None);
    run_jobs();
    window.set_content(Some(ferroui_controls::Control::boxed(list_box.clone())));
    run_jobs();
    capture_rendered_frame(&window);

    assert_drag_can_start(&window, &list_box, 0);
}
