//! The tests of part A of the crate: the test files of `tests/Xaml.Behaviors.Interactions.UnitTests/Custom`
//! of the reference that test the types of the folders of part A (`Core/`, `ExecuteCommand/`,
//! `Actions/`, `Logic/`), one module per test file, and their support files.
//!
//! Not ported here (the test files of the folder test the types of other parts or crates):
//! - Not ported: `AnimationAdapterTests` - the animations crate.
//! - Not ported: `ClickEventTriggerTests` - `ClickEventTrigger` is in `InputElement/` (another part).
//! - Not ported: `FocusTrapBehaviorTests` - `Focus/` (another part).
//! - Not ported: `InlineEditBehaviorTests` - `Control/` (another part).
//! - Not ported: `InvokeButtonClickActionTests` - `Button/` (another part).
//! - Not ported: `ToolTipTriggerTests` - `ToolTips/` (another part).
//! - Not ported: `ValidationRuleTests` - `Validation/` (another part).
//! - Not ported: `XmlnsDefinitionRuntimeXamlTests` - it loads `EventTriggerBehavior` of the
//!   interactions crate, not a type of this crate.

pub(crate) mod command;
#[allow(dead_code)]
pub(crate) mod observable_command;
pub(crate) mod support;

mod attached_to_visual_tree_lifecycle_tests;
mod detached_from_visual_tree_trigger_tests;
mod execute_command_behavior_base_tests;
mod focus_next_element_action_tests;
mod routed_event_trigger_behavior_tests;
