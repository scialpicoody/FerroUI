//! What markup and the untyped value conversions must know about the types the classes of this
//! crate hold. The classes state their properties through their registered properties, as every
//! class of the framework; the crate declares no other type for markup.

use crate::interaction_trigger_behavior::register_interaction_casts;
use crate::AnyInteraction;
use ferroui_base::data::core::ValueTypes;
use ferroui_reactiveui::{RoutableViewModel, RoutingState};

/// Registers the value types of this crate with the untyped value conversions of every thread.
/// Called once by `register_types()`.
pub(crate) fn register() {
    ValueTypes::register_global(register_value_types);
}

/// A router a view model exposes binds to the `Router` of the actions (a router is a reference
/// type); a routable view model binds to their `ViewModel`; an interaction of `Unit` input and
/// output (the type arguments the reference uses in markup) binds to the `Interaction` of the
/// trigger.
fn register_value_types() {
    ValueTypes::register_reference::<RoutingState>();
    ValueTypes::register_nullable::<RoutableViewModel>();
    ValueTypes::register_nullable::<AnyInteraction>();
    register_interaction_casts::<(), ()>();
}
