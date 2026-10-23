//! Port of `Views/Pages/PropertyValidationBehaviorView` of the reference sample: the class of
//! `pages/PropertyValidationBehaviorView.xaml`, and the validation behavior its document declares
//! (`PropertyValidationBehavior<TextBox, string>`).

use crate::markup::{user_control_class, xaml_class};
use crate::pages::{no_value_types, PageInfo};
use ferroui_base::{
    ferro_class, ferro_class_info, instantiate, FerroObjectImpl, Ref, StaticType,
    StyledElementImpl, TypeInfo,
};
use ferroui_controls::{Control, TextBox, UserControl};
use ferroui_xaml_interactions_custom::PropertyValidationBehavior;
use ferroui_xaml_interactivity::{DisposingBehaviorImpl, StyledElementBehaviorImpl};
use std::any::TypeId;

#[repr(C)]
pub struct PropertyValidationBehaviorView {
    base: UserControl,
}

user_control_class!(PropertyValidationBehaviorView);
ferro_class_info!(PropertyValidationBehaviorView {
    new: PropertyValidationBehaviorView::new
});
xaml_class!(
    PropertyValidationBehaviorView,
    "/pages/PropertyValidationBehaviorView.xaml"
);

impl PropertyValidationBehaviorView {
    pub fn construct() -> Self {
        Self {
            base: UserControl::construct(),
        }
    }

    /// `PropertyValidationBehaviorView()`.
    pub fn new() -> Ref<Self> {
        let this = instantiate(Self::construct());
        this.initialize_component();
        this
    }
}

/// `PropertyValidationBehavior<TextBox, string>` of the document of the page: the validation
/// behavior of a property of a text box whose values are texts.
///
/// Deviation (DEVIATIONS.md, Behaviors gallery): the document of the reference closes the generic
/// behavior with `x:TypeArguments="TextBox, sys:String"`; the behavior of the port is not generic
/// (its control and value types are given at creation), so the closed type is this class of the
/// page, which creates the behavior with those types and is otherwise the behavior itself (the
/// validated property is still set by the document).
#[repr(C)]
pub struct TextBoxPropertyValidationBehavior {
    base: PropertyValidationBehavior,
}

ferro_class!(TextBoxPropertyValidationBehavior: PropertyValidationBehavior);
ferro_class_info!(TextBoxPropertyValidationBehavior {
    new: TextBoxPropertyValidationBehavior::new
});
ferroui_base::ferro_impl_classes!(
    TextBoxPropertyValidationBehavior: FerroObjectImpl,
    StyledElementImpl,
    StyledElementBehaviorImpl,
    DisposingBehaviorImpl
);

impl TextBoxPropertyValidationBehavior {
    pub fn construct() -> Self {
        Self {
            base: PropertyValidationBehavior::construct(
                <TextBox as StaticType>::TYPE,
                TypeId::of::<Option<String>>(),
            ),
        }
    }

    /// `new PropertyValidationBehavior<TextBox, string>()`.
    pub fn new() -> Ref<Self> {
        instantiate(Self::construct())
    }
}

/// Creates the page.
fn create() -> Ref<Control> {
    PropertyValidationBehaviorView::new().upcast()
}

/// The classes of the page (`X::TYPE`), with the validation behavior of its document.
pub(crate) const TYPES: &[&TypeInfo] = &[
    PropertyValidationBehaviorView::TYPE,
    TextBoxPropertyValidationBehavior::TYPE,
];

/// The entry of the page in the page table.
pub(crate) const PAGE: PageInfo = PageInfo {
    header: "PropertyValidationBehavior",
    reference: "Views/Pages/PropertyValidationBehaviorView",
    create: Some(create),
    types: TYPES,
    classes: &[&PropertyValidationBehaviorView::XAML_CLASS],
    markup_types: &[],
    register_value_types: no_value_types,
};
