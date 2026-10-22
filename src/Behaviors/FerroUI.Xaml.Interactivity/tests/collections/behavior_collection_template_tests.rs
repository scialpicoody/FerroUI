//! Ported from
//! `tests/Xaml.Behaviors.Interactivity.UnitTests/Collections/BehaviorCollectionTemplateTests.cs` of
//! Xaml.Behaviors (with its window `BehaviorCollectionTemplate001`, `BehaviorCollectionTemplate001.xaml`
//! here, loaded with the run-time loader).

// Needs EventTriggerBehavior and ChangePropertyAction from ferroui-xaml-interactions (stage 3), and
// ferroui-markup-xaml-loader as a dev-dependency (the run-time loader).
#[cfg(any())]
mod enabled_with_interactions {
    use crate::tests::support::{object, start_app};
    use crate::{Interaction, ASSEMBLY};
    use ferroui_base::controls::{NameScopeRef, ResourceKey};
    use ferroui_base::interactivity::RoutedEventArgs;
    use ferroui_base::media::{Brushes, IBrush};
    use ferroui_base::metadata::from_markup_value;
    use ferroui_base::Ref;
    use ferroui_controls::{Button, Window};
    use ferroui_markup_xaml::{RuntimeXamlLoaderConfiguration, RuntimeXamlLoaderDocument};
    use ferroui_markup_xaml_loader::FerroRuntimeXamlLoader;
    use std::rc::Rc;

    fn load_window() -> Ref<Window> {
        FerroRuntimeXamlLoader::register();
        let mut configuration = RuntimeXamlLoaderConfiguration::new();
        configuration.local_assembly = Some(&ASSEMBLY);
        let loaded = FerroRuntimeXamlLoader::load_document(
            RuntimeXamlLoaderDocument::new(include_str!("BehaviorCollectionTemplate001.xaml")),
            Some(configuration),
        )
        .unwrap_or_else(|error| panic!("BehaviorCollectionTemplate001.xaml failed to load: {}", error.message()));
        from_markup_value::<Ref<Window>>(&Some(loaded)).expect("a window")
    }

    fn find_button(window: &Window, name: &str) -> Ref<Button> {
        NameScopeRef::get_name_scope(window)
            .and_then(|scope| scope.find(name))
            .and_then(|object| object.cast::<Button>())
            .expect("the named button")
    }

    #[test]
    fn behavior_collection_template_001() {
        let _app = start_app();
        let window = load_window();

        window.show();

        let buttons = [
            find_button(&window, "TargetButton1"),
            find_button(&window, "TargetButton2"),
            find_button(&window, "TargetButton3"),
        ];

        for button in &buttons {
            let behaviors = button.get_value(Interaction::behaviors_property());
            assert!(behaviors.is_some());
            let behaviors = behaviors.unwrap();
            assert_eq!(1, behaviors.count());
            assert_eq!(Some(object(button)), behaviors.associated_object());
        }

        for button in &buttons {
            assert_eq!(Some(Brushes::transparent()), button.background());
        }

        let red_brush = window.resources().get(&ResourceKey::from("RedBrush"));
        for button in &buttons {
            button.raise_event(&RoutedEventArgs::with_event(Button::click_event()));

            assert_eq!(red_brush, button.background().map(|brush| Rc::new(brush) as _));
        }
    }
}
