//! Part A of the crate: the folders of the reference listed below, one module per folder.
//!
//! - `Core/` ([`core`](self::core)): the base classes of the behaviors and triggers of the
//!   crate ([`AttachedToVisualTreeBehavior`], [`AttachedToVisualTreeTriggerBase`],
//!   [`RoutedEventTriggerBase`], [`RoutedEventTriggerBaseOfT`], ..) and the triggers of the
//!   lifecycle of their host;
//! - `ExecuteCommand/` ([`execute_command`](self::execute_command)): the behaviors that execute a
//!   command on an event, with their base classes (`ExecuteCommand/Core/`).
//!
//! The part owns its classes ([`TYPES`]), their public Rust paths and its markup types
//! ([`register`]); `lib.rs` re-exports everything public of it.

use ferroui_base::{StaticType, TypeInfo};

#[path = "core/mod.rs"]
mod core;
#[path = "execute_command/mod.rs"]
mod execute_command;

pub use self::core::*;
pub use self::execute_command::*;

macro_rules! types {
    ($($type_:ty),* $(,)?) => {
        &[$(<$type_ as StaticType>::TYPE),*]
    };
}

/// The classes of this part, registered by `register_types`.
pub(crate) const TYPES: &[&TypeInfo] = types![
    // Core/
    crate::ActualThemeVariantChangedBehavior,
    crate::ActualThemeVariantChangedTrigger,
    crate::AsyncLoadBehavior,
    crate::AttachedToLogicalTreeBehavior,
    crate::AttachedToLogicalTreeTrigger,
    crate::AttachedToLogicalTreeTriggerBase,
    crate::AttachedToVisualTreeBehavior,
    crate::AttachedToVisualTreeTrigger,
    crate::AttachedToVisualTreeTriggerBase,
    crate::BindingBehavior,
    crate::BindingTriggerBehavior,
    crate::DataContextChangedBehavior,
    crate::DataContextChangedTrigger,
    crate::DelayedLoadTrigger,
    crate::DetachedFromLogicalTreeTrigger,
    crate::DetachedFromVisualTreeTrigger,
    crate::IfElseTrigger,
    crate::InitializedBehavior,
    crate::InitializedTrigger,
    crate::LaunchUriOrFileAction,
    crate::LoadedBehavior,
    crate::LoadedTrigger,
    crate::ObservableTriggerBehavior,
    crate::PropertyChangedTrigger,
    crate::ResourcesChangedBehavior,
    crate::ResourcesChangedTrigger,
    crate::RoutedEventTrigger,
    crate::RoutedEventTriggerBase,
    crate::RoutedEventTriggerBaseOfT,
    crate::RoutedEventTriggerBehavior,
    crate::SizeChangedTrigger,
    crate::UnloadedTrigger,
    crate::ValueChangedTriggerBehavior,
    // ExecuteCommand/Core/
    crate::ExecuteCommandBehaviorBase,
    crate::ExecuteCommandOnKeyBehaviorBase,
    crate::ExecuteCommandRoutedEventBehaviorBase,
];

/// The public Rust paths of the classes of this part, by the shortest path another crate names
/// them by; the emitter of Rust source of the markup compiler reads them.
mod rust_paths {
    ferroui_base::ferro_rust_paths! {
        classes: [
            // Core/
            crate::ActualThemeVariantChangedBehavior,
            crate::ActualThemeVariantChangedTrigger,
            crate::AsyncLoadBehavior,
            crate::AttachedToLogicalTreeBehavior,
            crate::AttachedToLogicalTreeTrigger,
            crate::AttachedToLogicalTreeTriggerBase,
            crate::AttachedToVisualTreeBehavior,
            crate::AttachedToVisualTreeTrigger,
            crate::AttachedToVisualTreeTriggerBase,
            crate::BindingBehavior,
            crate::BindingTriggerBehavior,
            crate::DataContextChangedBehavior,
            crate::DataContextChangedTrigger,
            crate::DelayedLoadTrigger,
            crate::DetachedFromLogicalTreeTrigger,
            crate::DetachedFromVisualTreeTrigger,
            crate::IfElseTrigger,
            crate::InitializedBehavior,
            crate::InitializedTrigger,
            crate::LaunchUriOrFileAction,
            crate::LoadedBehavior,
            crate::LoadedTrigger,
            crate::ObservableTriggerBehavior,
            crate::PropertyChangedTrigger,
            crate::ResourcesChangedBehavior,
            crate::ResourcesChangedTrigger,
            crate::RoutedEventTrigger,
            crate::RoutedEventTriggerBase,
            crate::RoutedEventTriggerBaseOfT,
            crate::RoutedEventTriggerBehavior,
            crate::SizeChangedTrigger,
            crate::UnloadedTrigger,
            crate::ValueChangedTriggerBehavior,
            // ExecuteCommand/Core/
            crate::ExecuteCommandBehaviorBase,
            crate::ExecuteCommandOnKeyBehaviorBase,
            crate::ExecuteCommandRoutedEventBehaviorBase,
        ],
        types: [
        ],
        contracts: [
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
}
