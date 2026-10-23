//! Part C of the crate: the folders of the reference listed below, one module per folder.
//!
//! - `Animations/` ([`animations`](self::animations)): key-frame animations;
//! - `SplitView/` ([`split_view`](self::split_view)): the split view;
//! - `Notifications/` ([`notifications`](self::notifications)): the notifications of a top level;
//! - `RenderTarget/` ([`render_target`](self::render_target)): drawing into a render target bitmap, with its contracts;
//! - `WriteableBitmap/` ([`writeable_bitmap`](self::writeable_bitmap)): rendering into a writeable bitmap, with its contract;
//! - `Window/` ([`window`](self::window)): the window of the associated control;
//! - `Button/` ([`button`](self::button)): buttons.
//!
//! The part owns its classes ([`TYPES`]), their public Rust paths and its markup types
//! ([`register`]); `lib.rs` re-exports everything public of it.

use ferroui_base::animation::Animatable;
use ferroui_base::data::core::ValueTypes;
use ferroui_base::metadata::{MarkupType, MarkupTyped};
use ferroui_base::{BoxedValue, ObjectType, Ref, StaticType, TypeInfo, Visual};
use ferroui_controls::Control;
use ferroui_xaml_interactivity::helpers::member_lookup::object_of;
use std::rc::Rc;

#[path = "animations/mod.rs"]
mod animations;
#[path = "button/mod.rs"]
mod button;
#[path = "notifications/mod.rs"]
mod notifications;
#[path = "render_target/mod.rs"]
mod render_target;
#[path = "split_view/mod.rs"]
mod split_view;
#[path = "window/mod.rs"]
mod window;
#[path = "writeable_bitmap/mod.rs"]
mod writeable_bitmap;

pub use self::animations::*;
pub use self::button::*;
pub use self::notifications::*;
pub use self::render_target::*;
pub use self::split_view::*;
pub use self::window::*;
pub use self::writeable_bitmap::*;

macro_rules! types {
    ($($type_:ty),* $(,)?) => {
        &[$(<$type_ as StaticType>::TYPE),*]
    };
}

/// The classes of this part, registered by `register_types`.
pub(crate) const TYPES: &[&TypeInfo] = types![
    // Animations/
    crate::AnimateOnAttachedBehavior,
    crate::AnimationCompletedTrigger,
    crate::BeginAnimationAction,
    crate::FadeInBehavior,
    crate::PlayAnimationBehavior,
    crate::RunAnimationTrigger,
    crate::StartAnimationAction,
    crate::StartBuiltAnimationAction,
    // SplitView/
    crate::SplitViewPaneClosedTrigger,
    crate::SplitViewPaneClosingTrigger,
    crate::SplitViewPaneOpenedTrigger,
    crate::SplitViewPaneOpeningTrigger,
    crate::SplitViewStateBehavior,
    crate::SplitViewStateSetter,
    crate::SplitViewTogglePaneAction,
    // Notifications/
    crate::CloseNotificationAction,
    crate::NotificationManagerBehavior,
    crate::ShowErrorNotificationAction,
    crate::ShowInformationNotificationAction,
    crate::ShowNotificationAction,
    crate::ShowSuccessNotificationAction,
    crate::ShowWarningNotificationAction,
    // RenderTarget/
    crate::RenderRenderTargetBitmapAction,
    crate::RenderTargetBitmapBehavior,
    crate::RenderTargetBitmapTrigger,
    crate::StaticRenderTargetBitmapBehavior,
    // WriteableBitmap/
    crate::WriteableBitmapBehavior,
    crate::WriteableBitmapRenderAction,
    crate::WriteableBitmapRenderBehavior,
    crate::WriteableBitmapTimerTrigger,
    crate::WriteableBitmapTrigger,
    // Window/
    crate::CenterWindowBehavior,
    crate::CloseWindowAction,
    crate::ShowWindowAction,
    crate::WindowAction,
    crate::WindowDragMoveBehavior,
    crate::WindowStateTrigger,
    // Button/
    crate::ButtonClickEventTriggerBehavior,
    crate::ButtonExecuteCommandOnKeyDownBehavior,
    crate::ButtonHideFlyoutBehavior,
    crate::ButtonHideFlyoutOnClickBehavior,
    crate::ButtonHidePopupOnClickBehavior,
    crate::InvokeButtonClickAction,
];

/// The types of this part that are not classes (lists, enumerations and contracts), declared for
/// markup.
const MARKUP_TYPES: &[&MarkupType] = &[
    // SplitView/
    <crate::SplitViewStateSetters as MarkupTyped>::MARKUP,
    // RenderTarget/
    <dyn crate::IRenderTargetBitmapRenderHost as MarkupTyped>::MARKUP,
    <dyn crate::IRenderTargetBitmapRenderer as MarkupTyped>::MARKUP,
    <dyn crate::IRenderTargetBitmapSimpleRenderer as MarkupTyped>::MARKUP,
    // WriteableBitmap/
    <dyn crate::IWriteableBitmapRenderer as MarkupTyped>::MARKUP,
    // Window/
    <crate::WindowActionType as MarkupTyped>::MARKUP,
];

/// The public Rust paths of the classes of this part, by the shortest path another crate names
/// them by; the emitter of Rust source of the markup compiler reads them.
mod rust_paths {
    ferroui_base::ferro_rust_paths! {
        classes: [
            // Animations/
            crate::AnimateOnAttachedBehavior,
            crate::AnimationCompletedTrigger,
            crate::BeginAnimationAction,
            crate::FadeInBehavior,
            crate::PlayAnimationBehavior,
            crate::RunAnimationTrigger,
            crate::StartAnimationAction,
            crate::StartBuiltAnimationAction,
            // SplitView/
            crate::SplitViewPaneClosedTrigger,
            crate::SplitViewPaneClosingTrigger,
            crate::SplitViewPaneOpenedTrigger,
            crate::SplitViewPaneOpeningTrigger,
            crate::SplitViewStateBehavior,
            crate::SplitViewStateSetter,
            crate::SplitViewTogglePaneAction,
            // Notifications/
            crate::CloseNotificationAction,
            crate::NotificationManagerBehavior,
            crate::ShowErrorNotificationAction,
            crate::ShowInformationNotificationAction,
            crate::ShowNotificationAction,
            crate::ShowSuccessNotificationAction,
            crate::ShowWarningNotificationAction,
            // RenderTarget/
            crate::RenderRenderTargetBitmapAction,
            crate::RenderTargetBitmapBehavior,
            crate::RenderTargetBitmapTrigger,
            crate::StaticRenderTargetBitmapBehavior,
            // WriteableBitmap/
            crate::WriteableBitmapBehavior,
            crate::WriteableBitmapRenderAction,
            crate::WriteableBitmapRenderBehavior,
            crate::WriteableBitmapTimerTrigger,
            crate::WriteableBitmapTrigger,
            // Window/
            crate::CenterWindowBehavior,
            crate::CloseWindowAction,
            crate::ShowWindowAction,
            crate::WindowAction,
            crate::WindowDragMoveBehavior,
            crate::WindowStateTrigger,
            // Button/
            crate::ButtonClickEventTriggerBehavior,
            crate::ButtonExecuteCommandOnKeyDownBehavior,
            crate::ButtonHideFlyoutBehavior,
            crate::ButtonHideFlyoutOnClickBehavior,
            crate::ButtonHidePopupOnClickBehavior,
            crate::InvokeButtonClickAction,
        ],
        types: [
            // SplitView/
            crate::SplitViewStateSetters,
            // Window/
            crate::WindowActionType,
        ],
        contracts: [
            // RenderTarget/
            crate::IRenderTargetBitmapRenderHost,
            crate::IRenderTargetBitmapRenderer,
            crate::IRenderTargetBitmapSimpleRenderer,
            // WriteableBitmap/
            crate::IWriteableBitmapRenderer,
        ],
        generics: [
        ],
        generic_contracts: [
        ],
    }
}

/// Registers the Rust paths (`ferroui_base::ferro_rust_paths!`) and the markup types
/// (`MarkupType::register_all`) of this part. Called once by `register_types`.
pub(crate) fn register() {
    rust_paths::register_rust_paths();
    MarkupType::register_all(MARKUP_TYPES);
    ValueTypes::register_global(register_value_types);
}

/// What the untyped value conversions of markup and bindings must know about the types of this
/// part: the nullable forms of the handles, contracts, lists and enumerations its properties hold,
/// and the element references of the properties that name an element.
fn register_value_types() {
    // Animations/
    ValueTypes::register_nullable::<crate::AnimationBuilderRef>();
    ValueTypes::register_element_ref::<Control>();
    // SplitView/
    ValueTypes::register_nullable::<crate::SplitViewStateSetters>();
    ValueTypes::register_element_ref::<ferroui_controls::SplitView>();
    // Notifications/
    ValueTypes::register_nullable::<Rc<dyn ferroui_controls::INotificationManager>>();
    ValueTypes::register_nullable::<Rc<dyn ferroui_controls::INotification>>();
    ValueTypes::register_element_ref::<ferroui_controls::NotificationCard>();
    // RenderTarget/
    ValueTypes::register_nullable::<Rc<dyn crate::IRenderTargetBitmapRenderHost>>();
    ValueTypes::register_nullable::<Rc<dyn crate::IRenderTargetBitmapRenderer>>();
    ValueTypes::register_nullable::<Rc<dyn crate::IRenderTargetBitmapSimpleRenderer>>();
    // WriteableBitmap/
    ValueTypes::register_nullable::<crate::WriteableBitmapRef>();
    ValueTypes::register_nullable::<Rc<dyn crate::IWriteableBitmapRenderer>>();
    // Window/
    ValueTypes::register_nullable::<crate::WindowActionType>();
    ValueTypes::register_element_ref::<ferroui_controls::Window>();
    // Button/
    ValueTypes::register_element_ref::<ferroui_controls::Button>();
}

/// The sender of an action as an object of class `T` (`sender as T` of the reference).
pub(crate) fn sender_as<T: ObjectType>(sender: Option<&BoxedValue>) -> Option<Ref<T>> {
    sender.and_then(object_of).and_then(|object| object.cast::<T>())
}

/// A control as the animatable target of an animation.
pub(crate) fn as_animatable(control: &Control) -> &Animatable {
    control
}

/// A control as a visual.
pub(crate) fn control_as_visual(control: &Control) -> &Visual {
    control
}

/// A visual as the animatable target of an animation.
pub(crate) fn visual_as_animatable(visual: &Visual) -> &Animatable {
    visual
}
