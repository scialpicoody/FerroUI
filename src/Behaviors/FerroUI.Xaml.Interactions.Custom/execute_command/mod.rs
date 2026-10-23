//! The behaviors that execute a command when an event of their control fires (`ExecuteCommand/`).

mod core;

pub use self::core::{
    ExecuteCommandBehaviorBase, ExecuteCommandBehaviorBaseImpl, ExecuteCommandBehaviorBaseImplExt,
    ExecuteCommandBehaviorBaseVTable, ExecuteCommandOnKeyBehaviorBase, ExecuteCommandRoutedEventBehaviorBase,
};
