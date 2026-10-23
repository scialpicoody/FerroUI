//! Ported from `tests/Xaml.Behaviors.Interactions.UnitTests/Custom/AttachedToVisualTreeLifecycleTests.cs`
//! of Xaml.Behaviors.
//!
//! Adapted: the window is shown with the services of the test application (see `support.rs`); a
//! panel removes and adds its child without a rendered frame.

use super::support::start_app;
use crate::{
    AttachedToVisualTreeBehavior, AttachedToVisualTreeBehaviorImpl, AttachedToVisualTreeTriggerBase,
    AttachedToVisualTreeTriggerBaseImpl,
};
use ferroui_base::reactive::IDisposable;
use ferroui_base::{ferro_class, instantiate, FerroObjectImpl, Ref, StaticType, StyledElementImpl};
use ferroui_controls::{Border, Control, Panel, Window};
use ferroui_xaml_interactivity::{
    DisposableAction, DisposingBehaviorImpl, DisposingTriggerImpl, Interaction, StyledElementBehaviorImpl,
};
use std::cell::Cell;
use std::rc::Rc;

/// The counts of the subscriptions of a tracking trigger or behavior.
#[derive(Default)]
pub(crate) struct Subscriptions {
    active: Cell<i32>,
    created: Cell<i32>,
    disposed: Cell<i32>,
}

impl Subscriptions {
    /// `OnAttachedToVisualTreeOverride` of the tracking classes: counts a new subscription and
    /// returns its disposable, which counts its disposal.
    fn create(subscriptions: &Rc<Subscriptions>) -> Rc<dyn IDisposable> {
        subscriptions.active.set(subscriptions.active.get() + 1);
        subscriptions.created.set(subscriptions.created.get() + 1);

        let subscriptions = subscriptions.clone();
        DisposableAction::create(move || {
            subscriptions.active.set(subscriptions.active.get() - 1);
            subscriptions.disposed.set(subscriptions.disposed.get() + 1);
        })
    }
}

/// `TrackingTrigger`: an `AttachedToVisualTreeTriggerBase<Border>` that counts its subscriptions.
#[repr(C)]
pub(crate) struct TrackingTrigger {
    base: AttachedToVisualTreeTriggerBase,
    subscriptions: Rc<Subscriptions>,
}

ferro_class!(TrackingTrigger: AttachedToVisualTreeTriggerBase);
ferroui_base::ferro_impl_classes!(
    TrackingTrigger: FerroObjectImpl,
    StyledElementImpl,
    StyledElementBehaviorImpl,
    DisposingTriggerImpl,
);

ferroui_base::ferro_overrides! {
    impl AttachedToVisualTreeTriggerBaseImpl for TrackingTrigger {
        fn on_attached_to_visual_tree_override(this: &Self) -> Rc<dyn IDisposable> {
            Subscriptions::create(&this.subscriptions)
        }
    }
}

impl TrackingTrigger {
    fn new() -> Ref<Self> {
        instantiate(Self {
            base: AttachedToVisualTreeTriggerBase::construct(<Border as StaticType>::TYPE),
            subscriptions: Rc::new(Subscriptions::default()),
        })
    }
}

/// `TrackingBehavior`: an `AttachedToVisualTreeBehavior<Border>` that counts its subscriptions.
#[repr(C)]
pub(crate) struct TrackingBehavior {
    base: AttachedToVisualTreeBehavior,
    subscriptions: Rc<Subscriptions>,
}

ferro_class!(TrackingBehavior: AttachedToVisualTreeBehavior);
ferroui_base::ferro_impl_classes!(
    TrackingBehavior: FerroObjectImpl,
    StyledElementImpl,
    StyledElementBehaviorImpl,
    DisposingBehaviorImpl,
);

ferroui_base::ferro_overrides! {
    impl AttachedToVisualTreeBehaviorImpl for TrackingBehavior {
        fn on_attached_to_visual_tree_override(this: &Self) -> Rc<dyn IDisposable> {
            Subscriptions::create(&this.subscriptions)
        }
    }
}

impl TrackingBehavior {
    fn new() -> Ref<Self> {
        instantiate(Self {
            base: AttachedToVisualTreeBehavior::construct(<Border as StaticType>::TYPE),
            subscriptions: Rc::new(Subscriptions::default()),
        })
    }
}

/// `new Window { Content = panel }`, shown.
fn show_window_with(panel: &Ref<Panel>) -> Ref<Window> {
    let window = Window::new();
    window.set_content(Some(Control::boxed(panel)));
    window.show();
    window
}

// Not ported: PointerEnteredTrigger_ExecutesOnceAfterVisualTreeReattach - it uses
// `PointerEnteredTrigger` of `InputElement/Triggers/`, a folder of another part of the crate;
// later stage (once the parts are merged).

#[test]
fn attached_to_visual_tree_trigger_base_disposes_for_each_visual_tree_lifetime() {
    let _app = start_app();
    let target = Border::new();
    let panel = Panel::new();
    panel.children().add(&target);
    let trigger = TrackingTrigger::new();
    Interaction::get_behaviors(&target).add(trigger.clone().upcast());
    let _window = show_window_with(&panel);
    let subscriptions = trigger.subscriptions.clone();

    assert_eq!(1, subscriptions.active.get());

    panel.children().remove(&target);

    assert_eq!(0, subscriptions.active.get());
    assert_eq!(1, subscriptions.disposed.get());

    panel.children().add(&target);

    assert_eq!(1, subscriptions.active.get());
    assert_eq!(2, subscriptions.created.get());

    panel.children().remove(&target);

    assert_eq!(0, subscriptions.active.get());
    assert_eq!(2, subscriptions.disposed.get());
}

#[test]
fn attached_to_visual_tree_behavior_disposes_for_each_visual_tree_lifetime() {
    let _app = start_app();
    let target = Border::new();
    let panel = Panel::new();
    panel.children().add(&target);
    let behavior = TrackingBehavior::new();
    Interaction::get_behaviors(&target).add(behavior.clone().upcast());
    let _window = show_window_with(&panel);
    let subscriptions = behavior.subscriptions.clone();

    assert_eq!(1, subscriptions.active.get());

    panel.children().remove(&target);

    assert_eq!(0, subscriptions.active.get());
    assert_eq!(1, subscriptions.disposed.get());

    panel.children().add(&target);

    assert_eq!(1, subscriptions.active.get());
    assert_eq!(2, subscriptions.created.get());

    panel.children().remove(&target);

    assert_eq!(0, subscriptions.active.get());
    assert_eq!(2, subscriptions.disposed.get());
}
