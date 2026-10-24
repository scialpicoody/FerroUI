use crate::{MaterialColorRole, MaterialColorScheme, MaterialMotionRole, MaterialMotionScheme, MaterialTheme};
use ferroui_base::animation::easings::Easing;
use ferroui_base::animation::TimeSpan;
use ferroui_base::controls::ResourceKey;
use ferroui_base::media::{Color, IBrush};
use ferroui_base::metadata::from_markup_value;
use ferroui_base::styling::{ControlTheme, ThemeVariant};
use ferroui_base::Ref;
use ferroui_controls::testing::{TestServices, UnitTestApplication};
use ferroui_controls::{Button, CheckBox, ComboBox, GroupBox, ListBoxItem, ProgressBar, RadioButton, Slider, TabControl, TextBox, ToggleSwitch};
use std::rc::Rc;

#[test]
fn pinned_color_roles_match_androidx_and_are_complete() {
    assert_eq!(MaterialColorRole::ALL.len(), 48);
    assert_eq!(MaterialColorScheme::light().color(MaterialColorRole::Primary), Color::from_uint32(0xff6750a4));
    assert_eq!(MaterialColorScheme::dark().color(MaterialColorRole::Surface), Color::from_uint32(0xff141218));
    for scheme in [MaterialColorScheme::light(), MaterialColorScheme::dark()] {
        for role in MaterialColorRole::ALL {
            assert_eq!(scheme.color(role).a, 255, "{} must be an opaque role", role.name());
        }
    }
}

#[test]
fn compiled_theme_resolves_every_role_in_both_variants() {
    crate::register_types();
    let _app = UnitTestApplication::start(TestServices::styled_window());
    let theme = MaterialTheme::new();
    for (variant, expected) in [(ThemeVariant::light(), MaterialColorScheme::light()), (ThemeVariant::dark(), MaterialColorScheme::dark())] {
        for role in MaterialColorRole::ALL {
            let key = ResourceKey::from(format!("Material{}Color", role.name()));
            let color = theme.try_get_resource(&key, Some(&variant)).flatten();
            assert_eq!(from_markup_value::<Color>(&color), Some(expected.color(role)), "{}", role.name());
            let brush = theme.try_get_resource(&ResourceKey::from(format!("Material{}Brush", role.name())), Some(&variant)).flatten();
            assert!(from_markup_value::<Rc<dyn IBrush>>(&brush).is_some(), "{} brush", role.name());
        }
    }
}

#[test]
fn replacing_palette_updates_role_and_compatibility_brushes_together() {
    crate::register_types();
    let _app = UnitTestApplication::start(TestServices::styled_window());
    let theme = MaterialTheme::new();
    let custom = MaterialColorScheme::light().with_color(MaterialColorRole::Primary, Color::from_rgb(14, 78, 44));
    theme.set_color_scheme(ThemeVariant::light(), &custom);
    let color = theme.try_get_resource(&ResourceKey::from("MaterialPrimaryColor"), Some(&ThemeVariant::light())).flatten();
    assert_eq!(from_markup_value::<Color>(&color), Some(Color::from_rgb(14, 78, 44)));
    let dark = theme.try_get_resource(&ResourceKey::from("MaterialPrimaryColor"), Some(&ThemeVariant::dark())).flatten();
    assert_eq!(from_markup_value::<Color>(&dark), Some(MaterialColorScheme::dark().color(MaterialColorRole::Primary)));
}

#[test]
fn reduced_motion_replaces_the_resources_consumed_by_transitions() {
    crate::register_types();
    let _app = UnitTestApplication::start(TestServices::styled_window());
    let theme = MaterialTheme::new();
    theme.set_reduced_motion(true);
    for role in MaterialMotionRole::ALL {
        let duration = theme.try_get_resource(&ResourceKey::from(format!("MaterialMotion{}Duration", role.resource_name())), None).flatten();
        assert_eq!(from_markup_value::<TimeSpan>(&duration), Some(TimeSpan::ZERO));
        let easing = theme.try_get_resource(&ResourceKey::from(format!("MaterialMotion{}Easing", role.resource_name())), None).flatten();
        let easing = from_markup_value::<Easing>(&easing).expect("actual transition easing");
        assert_eq!(easing.ease(0.25), 1.0);
    }
    theme.set_reduced_motion(false);
    theme.set_motion_scheme(MaterialMotionScheme::Standard);
    let duration = theme.try_get_resource(&ResourceKey::from("MaterialMotionFastSpatialDuration"), None).flatten();
    assert_eq!(from_markup_value::<TimeSpan>(&duration), Some(MaterialMotionScheme::Standard.duration(MaterialMotionRole::FastSpatial, false)));
}

#[test]
fn default_controls_and_material_variants_resolve_compiled_templates() {
    crate::register_types();
    let _app = UnitTestApplication::start(TestServices::styled_window());
    let theme = MaterialTheme::new();
    for type_ in [Button::TYPE, CheckBox::TYPE, RadioButton::TYPE, ToggleSwitch::TYPE, Slider::TYPE, TextBox::TYPE, ComboBox::TYPE, ListBoxItem::TYPE, TabControl::TYPE, GroupBox::TYPE, ProgressBar::TYPE] {
        let value = theme.try_get_resource(&ResourceKey::Type(type_), None).flatten();
        assert!(from_markup_value::<Ref<ControlTheme>>(&value).is_some(), "{}", type_.name());
    }
    for key in ["MaterialFilledButton", "MaterialTonalButton", "MaterialOutlinedButton", "MaterialTextButton", "MaterialElevatedButton", "MaterialFilledCard", "MaterialElevatedCard", "MaterialOutlinedCard", "MaterialDialog", "MaterialAssistChip", "MaterialFilterChip", "MaterialSearchBar", "MaterialNavigationBar", "MaterialNavigationRail"] {
        let value = theme.try_get_resource(&ResourceKey::from(key), None).flatten();
        assert!(from_markup_value::<Ref<ControlTheme>>(&value).is_some(), "{key}");
    }
}
