//! What the untyped value conversions of markup and bindings must know about the types of this
//! crate: the handler contracts as nullable values of the `Handler` properties, the element
//! references of the properties that name a control, and the storage item handles.

use crate::values::StorageItemValue;
use crate::{IDragHandler, IDropHandler};
use ferroui_base::data::core::ValueTypes;
use ferroui_controls::{Control, ItemsControl};
use std::rc::Rc;

/// Registers the value types of this crate. Called once by `register_types()`.
pub(crate) fn register() {
    ValueTypes::register_global(register_value_types);
}

fn register_value_types() {
    ValueTypes::register_nullable::<Rc<dyn IDragHandler>>();
    ValueTypes::register_nullable::<Rc<dyn IDropHandler>>();
    ValueTypes::register_nullable::<StorageItemValue>();
    ValueTypes::register_element_ref::<Control>();
    ValueTypes::register_element_ref::<ItemsControl>();
}
