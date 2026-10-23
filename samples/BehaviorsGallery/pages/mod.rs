//! The pages of the gallery (namespace `BehaviorsGallery.Views.Pages`): one module and one
//! document per page (`<page>.rs` and `<Page>.xaml`), and the table of the pages ([`PAGES`]), in
//! the order of the tab items of the main view of the reference.
//!
//! A page module declares its class and its entry of the table (`PAGE`, a [`PageInfo`]): its
//! header, the reference file it is ported from, the function that creates it, and the types it
//! registers (its class, and the view models, behaviors and other types only it uses). A page not
//! ported yet is an entry made by [`PageInfo::placeholder`]: the gallery shows its name and that
//! it is not ported yet.
//!
//! Porting a page (`PAGES.md`, "How to port a page") changes two lines of this file, the module
//! line of the page (the comment marker is removed) and its line of the table (the placeholder
//! becomes `<page>::PAGE`); the two lists are in the same order and grouped the same way.

use crate::markup::XamlClass;
use ferroui_base::metadata::MarkupType;
use ferroui_base::{Ref, TypeInfo};
use ferroui_controls::Control;

pub mod placeholder;

// The modules of the pages, in the order of the table; a page not ported yet is a comment.
// Group A (pages 1 to 86)
pub mod select_all_on_focus_behavior_view;
pub mod focus_next_element_action_view;
pub mod window_drag_move_behavior_view;
pub mod tilt_effect_behavior_view;
pub mod orbit_effect_behavior_view;
pub mod parallax_behavior_view;
pub mod conditional_action_view;
pub mod switch_case_action_view;
pub mod switch_case_behavior_view;
pub mod conditional_behavior_view;
pub mod lose_focus_on_enter_behavior_view;
pub mod auto_scroll_to_bottom_behavior_view;
pub mod call_method_action_view;
pub mod debounce_throttle_action_view;
pub mod observable_stream_behavior_view;
pub mod async_action_group_view;
pub mod focus_trap_behavior_view;
pub mod screen_reader_announce_action_view;
pub mod global_hotkey_behavior_view;
pub mod visual_debug_behavior_view;
pub mod log_action_view;
pub mod network_status_trigger_view;
pub mod clipboard_monitor_behavior_view;
pub mod change_property_action_view;
pub mod data_trigger_behavior_view;
pub mod data_trigger_behavior_advanced_view;
pub mod multi_data_trigger_behavior_view;
pub mod binding_trigger_behavior_view;
pub mod value_changed_trigger_behavior_view;
pub mod property_changed_trigger_view;
pub mod event_trigger_behavior_view;
pub mod timer_trigger_view;
pub mod task_completed_trigger_view;
pub mod loaded_trigger_view;
pub mod invoke_command_action_view;
pub mod auto_focus_behavior_view;
pub mod auto_select_behavior_view;
pub mod text_box_validation_behavior_view;
pub mod numeric_up_down_validation_behavior_view;
pub mod not_null_validation_rule_view;
pub mod property_validation_behavior_view;
pub mod slider_validation_behavior_view;
pub mod date_picker_validation_behavior_view;
pub mod combo_box_validation_behavior_view;
pub mod focus_control_behavior_view;
pub mod focus_on_attached_behavior_view;
pub mod focus_on_attached_to_visual_tree_behavior_view;
pub mod focus_on_pointer_moved_behavior_view;
pub mod focus_on_pointer_pressed_behavior_view;
pub mod focus_on_visible_behavior_view;
pub mod focus_selected_item_behavior_view;
pub mod routed_event_trigger_behavior_view;
pub mod change_ferro_property_action_view;
pub mod view_model_property_actions_view;
pub mod custom_action_view;
pub mod custom_behavior_view;
pub mod button_click_event_trigger_behavior_view;
pub mod click_event_trigger_view;
pub mod advanced_view;
pub mod add_remove_class_action_view;
pub mod remove_element_action_view;
pub mod remove_item_in_list_box_action_view;
pub mod remove_item_in_items_control_action_view;
pub mod add_item_to_items_control_action_view;
pub mod insert_item_to_items_control_action_view;
pub mod object_template_view;
pub mod clear_items_control_action_view;
pub mod add_range_action_view;
pub mod remove_range_action_view;
pub mod move_item_in_items_control_action_view;
pub mod move_element_actions_view;
pub mod collection_changed_trigger_behavior_view;
pub mod items_control_container_triggers_view;
pub mod remove_items_sample_view;
pub mod adaptive_behavior_view;
pub mod aspect_ratio_behavior_view;
pub mod split_view_state_behavior_view;
pub mod editable_list_box_view;
pub mod editable_draggable_list_box_view;
pub mod editable_double_tapped_list_box_view;
pub mod editable_tree_view_view;
pub mod editable_double_tapped_tree_view_view;
pub mod editable_drag_tree_view_view;
pub mod sliding_animation_view;
pub mod fade_animation_view;
pub mod scale_animation_view;
// Group B (pages 87 to 144)
// pub mod rotate_animation_view;
// pub mod entrance_animations_view;
// pub mod attention_animations_view;
// pub mod exit_animations_view;
// pub mod framer_motion_animations_view;
// pub mod special_animations_view;
// pub mod animation_behavior_view;
// pub mod animation_completed_trigger_view;
// pub mod start_animation_action_view;
// pub mod animation_builder_view;
// pub mod start_built_animation_action_view;
// pub mod run_animation_trigger_view;
// pub mod fluid_move_behavior_view;
// pub mod selecting_items_control_behavior_view;
// pub mod list_box_selection_behaviors_view;
// pub mod behavior_collection_template_view;
pub mod action_collection_template_view;
// pub mod template_binding_view;
// pub mod one_time_binding;
// pub mod pointer_triggers_view;
// pub mod key_gesture_trigger_view;
// pub mod execute_command_behaviors_view;
// pub mod button_execute_command_on_key_down_behavior_view;
// pub mod storage_provider_view;
// pub mod picker_can_execute_command_view;
// pub mod clipboard_view;
// pub mod get_clipboard_data_action_view;
// pub mod get_clipboard_formats_action_view;
// pub mod set_clipboard_data_object_action_view;
// pub mod focus_control_action_view;
// pub mod hide_show_control_action_view;
// pub mod show_hide_flyout_action_view;
// pub mod show_context_menu_action_view;
// pub mod context_dialog_view;
// pub mod show_hide_popup_action_view;
// pub mod show_on_event_behaviors_view;
// pub mod set_enabled_action_view;
// pub mod show_notification_action_view;
// pub mod dialog_action_view;
// pub mod file_drop_handler_view;
// pub mod upload_file_view;
// pub mod content_control_files_drop_behavior_view;
// pub mod drag_and_drop_view;
pub mod drag_drop_commands_view;
// pub mod files_preview_view;
// pub mod typed_drag_behavior_view;
// pub mod draggable_view;
// pub mod mouse_drag_behavior_view;
// pub mod canvas_drag_behavior_view;
// pub mod auto_scroll_during_drag_behavior_view;
// pub mod drag_between_panels_view;
// pub mod cursor_view;
// pub mod drawn_cursor_view;
// pub mod if_else_trigger_view;
// pub mod execute_script_action_view;
// pub mod observable_trigger_behavior_view;
// pub mod interaction_trigger_behavior_view;
// pub mod flyout_binding_view;
// Group C (pages 145 to 234)
// pub mod key_trigger_view;
// pub mod tool_tip_helpers_view;
// pub mod writeable_bitmap_view;
// pub mod render_target_bitmap_view;
// pub mod launch_uri_or_file_action_view;
// pub mod reactive_navigation_view;
// pub mod transitions_behavior_view;
// pub mod transitions_actions_view;
// pub mod transitions_changed_trigger_view;
// pub mod notifications_view;
// pub mod carousel_navigation_view;
// pub mod carousel_selection_changed_trigger_view;
// pub mod auto_complete_box_view;
// pub mod focus_auto_complete_box_text_box_behavior_view;
// pub mod tab_control_navigation_view;
// pub mod tab_control_selection_changed_trigger_view;
// pub mod theme_variant_view;
// pub mod screen_view;
// pub mod bind_pointer_over_behavior_view;
// pub mod bind_tag_to_visual_root_data_context_behavior_view;
// pub mod bounds_observer_behavior_view;
// pub mod hide_attached_flyout_behavior_view;
// pub mod button_hide_flyout_behavior_view;
// pub mod button_hide_flyout_on_click_behavior_view;
// pub mod hide_on_key_pressed_behavior_view;
// pub mod hide_on_lost_focus_behavior_view;
// pub mod size_changed_trigger_view;
// pub mod fade_in_behavior_view;
// pub mod delayed_load_behavior_view;
// pub mod delayed_load_trigger_view;
// pub mod delayed_show_control_action_view;
// pub mod animate_on_attached_behavior_view;
// pub mod async_loading_view;
// pub mod theme_variant_trigger_view;
// pub mod actual_theme_variant_changed_trigger_view;
// pub mod toggle_is_expanded_on_double_tapped_behavior_view;
// pub mod tree_view_filter_view;
// pub mod viewport_behavior_view;
// pub mod scroll_changed_trigger_view;
// pub mod scroll_viewer_offset_behavior_view;
// pub mod scroll_to_offset_action_view;
// pub mod scroll_behaviors_view;
// pub mod window_interactions_view;
// pub mod events_behaviors_view;
// pub mod focus_event_triggers_view;
// pub mod key_event_triggers_view;
pub mod pointer_event_triggers_view;
// pub mod pointer_events_trigger_view;
// pub mod drag_leave_event_trigger_view;
// pub mod gesture_event_triggers_view;
// pub mod gesture_advanced_triggers_view;
// pub mod pull_gesture_triggers_view;
// pub mod scroll_gesture_triggers_view;
// pub mod touch_pad_gesture_triggers_view;
// pub mod popup_event_triggers_view;
// pub mod focus_triggers_view;
// pub mod key_input_triggers_view;
// pub mod pointer_extra_triggers_view;
// pub mod gesture_triggers_view;
// pub mod icon_view;
// pub mod automation_view;
// pub mod actual_theme_variant_changed_behavior_view;
// pub mod attached_to_logical_tree_behavior_view;
// pub mod attached_to_logical_tree_trigger_view;
// pub mod attached_to_visual_tree_trigger_view;
// pub mod binding_behavior_view;
// pub mod data_context_changed_behavior_view;
// pub mod data_context_changed_trigger_view;
// pub mod detached_from_logical_tree_trigger_view;
// pub mod detached_from_visual_tree_trigger_view;
// pub mod disposing_behavior_view;
// pub mod disposing_trigger_view;
// pub mod disposable_action_view;
// pub mod initialized_behavior_view;
// pub mod initialized_trigger_view;
// pub mod invoke_command_action_base_view;
// pub mod loaded_behavior_view;
// pub mod resources_changed_behavior_view;
// pub mod resources_changed_trigger_view;
// pub mod routed_event_trigger_base_view;
// pub mod unloaded_trigger_view;
// pub mod file_system_view;
// pub mod network_view;
// pub mod toggle_class_action_view;
// pub mod launch_uri_action_view;
// pub mod screenshot_action_view;
// pub mod scroll_to_control_action_view;
// pub mod console_beep_action_view;
// pub mod break_action_view;
// pub mod inactivity_trigger_view;

/// A page of the gallery: an entry of the page table.
pub struct PageInfo {
    /// The header of the tab of the page (the header of its tab item in the reference).
    pub header: &'static str,
    /// The reference file of the page, below the directory of the reference sample, without its
    /// extension (the document and its code-behind).
    pub reference: &'static str,
    /// Creates the page (its constructor loads its document, and panics if it does not load);
    /// `None` for a page not ported yet.
    pub create: Option<fn() -> Ref<Control>>,
    /// The classes the page registers: its class, and the classes only it uses (`X::TYPE`).
    pub types: &'static [&'static TypeInfo],
    /// The classes with a document the page declares (`&X::XAML_CLASS`).
    pub classes: &'static [&'static XamlClass],
    /// The types declared with `ferro_markup_type!` only the page uses: its view models, drop
    /// handlers and converters (`<X as MarkupTyped>::MARKUP`).
    pub markup_types: &'static [&'static MarkupType],
    /// Registers what the untyped value conversions must know about the types only the page uses
    /// (`ValueTypes::register_reference::<X>()`, casts to contracts).
    pub register_value_types: fn(),
}

impl PageInfo {
    /// The entry of a page not ported yet: its header and its reference file.
    pub const fn placeholder(header: &'static str, reference: &'static str) -> Self {
        Self {
            header,
            reference,
            create: None,
            types: &[],
            classes: &[],
            markup_types: &[],
            register_value_types: no_value_types,
        }
    }

    /// The name of the view of the page (the class of the reference).
    pub fn view_name(&self) -> &'static str {
        self.reference.rsplit('/').next().unwrap_or(self.reference)
    }

    /// Whether the page is ported (its entry creates its view).
    pub fn is_ported(&self) -> bool {
        self.create.is_some()
    }

    /// Creates the page, or the placeholder of a page not ported yet.
    pub fn create_view(&'static self) -> Ref<Control> {
        match self.create {
            Some(create) => create(),
            None => placeholder::create(self),
        }
    }

    /// The page with the header or the view name `name` (without regard to case).
    pub fn find(name: &str) -> Option<&'static PageInfo> {
        let name = name.trim();
        PAGES.iter().find(|page| {
            page.header.eq_ignore_ascii_case(name) || page.view_name().eq_ignore_ascii_case(name)
        })
    }
}

/// `register_value_types` of a page whose types need no registration.
pub fn no_value_types() {}

/// The pages, in the order of the reference (a constant, so that the table of the classes is built
/// from it at compile time).
#[rustfmt::skip]
pub(crate) const PAGE_TABLE: &[PageInfo] = &[
    // Group A (pages 1 to 86)
    select_all_on_focus_behavior_view::PAGE,
    focus_next_element_action_view::PAGE,
    window_drag_move_behavior_view::PAGE,
    tilt_effect_behavior_view::PAGE,
    orbit_effect_behavior_view::PAGE,
    parallax_behavior_view::PAGE,
    conditional_action_view::PAGE,
    switch_case_action_view::PAGE,
    switch_case_behavior_view::PAGE,
    conditional_behavior_view::PAGE,
    lose_focus_on_enter_behavior_view::PAGE,
    auto_scroll_to_bottom_behavior_view::PAGE,
    call_method_action_view::PAGE,
    debounce_throttle_action_view::PAGE,
    observable_stream_behavior_view::PAGE,
    async_action_group_view::PAGE,
    focus_trap_behavior_view::PAGE,
    screen_reader_announce_action_view::PAGE,
    global_hotkey_behavior_view::PAGE,
    visual_debug_behavior_view::PAGE,
    log_action_view::PAGE,
    network_status_trigger_view::PAGE,
    clipboard_monitor_behavior_view::PAGE,
    change_property_action_view::PAGE,
    data_trigger_behavior_view::PAGE,
    data_trigger_behavior_advanced_view::PAGE,
    multi_data_trigger_behavior_view::PAGE,
    binding_trigger_behavior_view::PAGE,
    value_changed_trigger_behavior_view::PAGE,
    property_changed_trigger_view::PAGE,
    event_trigger_behavior_view::PAGE,
    timer_trigger_view::PAGE,
    task_completed_trigger_view::PAGE,
    loaded_trigger_view::PAGE,
    invoke_command_action_view::PAGE,
    auto_focus_behavior_view::PAGE,
    auto_select_behavior_view::PAGE,
    text_box_validation_behavior_view::PAGE,
    numeric_up_down_validation_behavior_view::PAGE,
    not_null_validation_rule_view::PAGE,
    property_validation_behavior_view::PAGE,
    slider_validation_behavior_view::PAGE,
    date_picker_validation_behavior_view::PAGE,
    combo_box_validation_behavior_view::PAGE,
    focus_control_behavior_view::PAGE,
    focus_on_attached_behavior_view::PAGE,
    focus_on_attached_to_visual_tree_behavior_view::PAGE,
    focus_on_pointer_moved_behavior_view::PAGE,
    focus_on_pointer_pressed_behavior_view::PAGE,
    focus_on_visible_behavior_view::PAGE,
    focus_selected_item_behavior_view::PAGE,
    routed_event_trigger_behavior_view::PAGE,
    change_ferro_property_action_view::PAGE,
    view_model_property_actions_view::PAGE,
    custom_action_view::PAGE,
    custom_behavior_view::PAGE,
    button_click_event_trigger_behavior_view::PAGE,
    click_event_trigger_view::PAGE,
    advanced_view::PAGE,
    add_remove_class_action_view::PAGE,
    remove_element_action_view::PAGE,
    remove_item_in_list_box_action_view::PAGE,
    remove_item_in_items_control_action_view::PAGE,
    add_item_to_items_control_action_view::PAGE,
    insert_item_to_items_control_action_view::PAGE,
    object_template_view::PAGE,
    clear_items_control_action_view::PAGE,
    add_range_action_view::PAGE,
    remove_range_action_view::PAGE,
    move_item_in_items_control_action_view::PAGE,
    move_element_actions_view::PAGE,
    collection_changed_trigger_behavior_view::PAGE,
    items_control_container_triggers_view::PAGE,
    remove_items_sample_view::PAGE,
    adaptive_behavior_view::PAGE,
    aspect_ratio_behavior_view::PAGE,
    split_view_state_behavior_view::PAGE,
    editable_list_box_view::PAGE,
    editable_draggable_list_box_view::PAGE,
    editable_double_tapped_list_box_view::PAGE,
    editable_tree_view_view::PAGE,
    editable_double_tapped_tree_view_view::PAGE,
    editable_drag_tree_view_view::PAGE,
    sliding_animation_view::PAGE,
    fade_animation_view::PAGE,
    scale_animation_view::PAGE,
    // Group B (pages 87 to 144)
    PageInfo::placeholder("Rotate Animation", "Views/Pages/RotateAnimationView"),
    PageInfo::placeholder("Entrance Animations", "Views/Pages/EntranceAnimationsView"),
    PageInfo::placeholder("Attention Animations", "Views/Pages/AttentionAnimationsView"),
    PageInfo::placeholder("Exit Animations", "Views/Pages/ExitAnimationsView"),
    PageInfo::placeholder("Framer Motion Animations", "Views/Pages/FramerMotionAnimationsView"),
    PageInfo::placeholder("Special Animations", "Views/Pages/SpecialAnimationsView"),
    PageInfo::placeholder("Animation Behaviors", "Views/Pages/AnimationBehaviorView"),
    PageInfo::placeholder("Animation Completed Trigger", "Views/Pages/AnimationCompletedTriggerView"),
    PageInfo::placeholder("StartAnimationAction", "Views/Pages/StartAnimationActionView"),
    PageInfo::placeholder("AnimationBuilder", "Views/Pages/AnimationBuilderView"),
    PageInfo::placeholder("StartBuiltAnimationAction", "Views/Pages/StartBuiltAnimationActionView"),
    PageInfo::placeholder("RunAnimationTrigger", "Views/Pages/RunAnimationTriggerView"),
    PageInfo::placeholder("FluidMoveBehavior", "Views/Pages/FluidMoveBehaviorView"),
    PageInfo::placeholder("SelectingItemsControlBehavior", "Views/Pages/SelectingItemsControlBehaviorView"),
    PageInfo::placeholder("ListBox Selection Behaviors", "Views/Pages/ListBoxSelectionBehaviorsView"),
    PageInfo::placeholder("BehaviorCollectionTemplate", "Views/Pages/BehaviorCollectionTemplateView"),
    action_collection_template_view::PAGE,
    PageInfo::placeholder("TemplateBinding", "Views/Pages/TemplateBindingView"),
    PageInfo::placeholder("OneTime Binding", "Views/Pages/OneTimeBinding"),
    PageInfo::placeholder("Pointer Triggers", "Views/Pages/PointerTriggersView"),
    PageInfo::placeholder("KeyGestureTrigger", "Views/Pages/KeyGestureTriggerView"),
    PageInfo::placeholder("ExecuteCommand Behaviors", "Views/Pages/ExecuteCommandBehaviorsView"),
    PageInfo::placeholder("ButtonExecuteCommandOnKeyDownBehavior", "Views/Pages/ButtonExecuteCommandOnKeyDownBehaviorView"),
    PageInfo::placeholder("Storage Provider", "Views/Pages/StorageProviderView"),
    PageInfo::placeholder("Picker CanExecuteCommand", "Views/Pages/PickerCanExecuteCommandView"),
    PageInfo::placeholder("Clipboard", "Views/Pages/ClipboardView"),
    PageInfo::placeholder("GetClipboardDataAction", "Views/Pages/GetClipboardDataActionView"),
    PageInfo::placeholder("GetClipboardFormatsAction", "Views/Pages/GetClipboardFormatsActionView"),
    PageInfo::placeholder("SetClipboardDataObjectAction", "Views/Pages/SetClipboardDataObjectActionView"),
    PageInfo::placeholder("FocusControlAction", "Views/Pages/FocusControlActionView"),
    PageInfo::placeholder("Hide/ShowControlAction", "Views/Pages/HideShowControlActionView"),
    PageInfo::placeholder("ShowHideFlyoutAction", "Views/Pages/ShowHideFlyoutActionView"),
    PageInfo::placeholder("ShowContextMenuAction", "Views/Pages/ShowContextMenuActionView"),
    PageInfo::placeholder("ContextDialog", "Views/Pages/ContextDialogView"),
    PageInfo::placeholder("ShowHidePopupAction", "Views/Pages/ShowHidePopupActionView"),
    PageInfo::placeholder("ShowOn Event Behaviors", "Views/Pages/ShowOnEventBehaviorsView"),
    PageInfo::placeholder("SetEnabledAction", "Views/Pages/SetEnabledActionView"),
    PageInfo::placeholder("ShowNotificationAction", "Views/Pages/ShowNotificationActionView"),
    PageInfo::placeholder("Dialog Actions", "Views/Pages/DialogActionView"),
    PageInfo::placeholder("File Drop Handler", "Views/Pages/FileDropHandlerView"),
    PageInfo::placeholder("Upload File", "Views/Pages/UploadFileView"),
    PageInfo::placeholder("ContentControlFilesDropBehavior", "Views/Pages/ContentControlFilesDropBehaviorView"),
    PageInfo::placeholder("Drag and Drop", "Views/Pages/DragAndDropView"),
    drag_drop_commands_view::PAGE,
    PageInfo::placeholder("Files Preview", "Views/Pages/FilesPreviewView"),
    PageInfo::placeholder("TypedDragBehavior", "Views/Pages/TypedDragBehaviorView"),
    PageInfo::placeholder("Draggable", "Views/Pages/DraggableView"),
    PageInfo::placeholder("Mouse Drag Behaviors", "Views/Pages/MouseDragBehaviorView"),
    PageInfo::placeholder("Canvas Drag Behavior", "Views/Pages/CanvasDragBehaviorView"),
    PageInfo::placeholder("Auto Scroll During Drag", "Views/Pages/AutoScrollDuringDragBehaviorView"),
    PageInfo::placeholder("Drag Between Panels", "Views/Pages/DragBetweenPanelsView"),
    PageInfo::placeholder("Cursor Behavior", "Views/Pages/CursorView"),
    PageInfo::placeholder("Drawn Cursor", "Views/Pages/DrawnCursorView"),
    PageInfo::placeholder("IfElseTrigger", "Views/Pages/IfElseTriggerView"),
    PageInfo::placeholder("ExecuteScriptAction", "Views/Pages/ExecuteScriptActionView"),
    PageInfo::placeholder("Observable Trigger", "Views/ReactiveUI/ObservableTriggerBehaviorView"),
    PageInfo::placeholder("InteractionTriggerBehavior", "Views/ReactiveUI/InteractionTriggerBehaviorView"),
    PageInfo::placeholder("Flyout Binding", "Views/Pages/FlyoutBindingView"),
    // Group C (pages 145 to 234)
    PageInfo::placeholder("KeyTrigger", "Views/Pages/KeyTriggerView"),
    PageInfo::placeholder("ToolTip Helpers", "Views/Pages/ToolTipHelpersView"),
    PageInfo::placeholder("WriteableBitmap", "Views/Pages/WriteableBitmapView"),
    PageInfo::placeholder("RenderTargetBitmap", "Views/Pages/RenderTargetBitmapView"),
    PageInfo::placeholder("LaunchUriOrFileAction", "Views/Pages/LaunchUriOrFileActionView"),
    PageInfo::placeholder("Reactive Navigation", "Views/ReactiveUI/ReactiveNavigationView"),
    PageInfo::placeholder("Transitions Behavior", "Views/Pages/TransitionsBehaviorView"),
    PageInfo::placeholder("Transitions Actions", "Views/Pages/TransitionsActionsView"),
    PageInfo::placeholder("Transitions Trigger", "Views/Pages/TransitionsChangedTriggerView"),
    PageInfo::placeholder("Notifications", "Views/Pages/NotificationsView"),
    PageInfo::placeholder("Carousel Navigation", "Views/Pages/CarouselNavigationView"),
    PageInfo::placeholder("CarouselSelectionChangedTrigger", "Views/Pages/CarouselSelectionChangedTriggerView"),
    PageInfo::placeholder("AutoCompleteBox", "Views/Pages/AutoCompleteBoxView"),
    PageInfo::placeholder("FocusAutoCompleteBoxTextBoxBehavior", "Views/Pages/FocusAutoCompleteBoxTextBoxBehaviorView"),
    PageInfo::placeholder("TabControl Navigation", "Views/Pages/TabControlNavigationView"),
    PageInfo::placeholder("TabControl Selection Trigger", "Views/Pages/TabControlSelectionChangedTriggerView"),
    PageInfo::placeholder("Theme Variant", "Views/Pages/ThemeVariantView"),
    PageInfo::placeholder("Screens", "Views/Pages/ScreenView"),
    PageInfo::placeholder("BindPointerOverBehavior", "Views/Pages/BindPointerOverBehaviorView"),
    PageInfo::placeholder("BindTagToVisualRootDataContextBehavior", "Views/Pages/BindTagToVisualRootDataContextBehaviorView"),
    PageInfo::placeholder("BoundsObserverBehavior", "Views/Pages/BoundsObserverBehaviorView"),
    PageInfo::placeholder("HideAttachedFlyoutBehavior", "Views/Pages/HideAttachedFlyoutBehaviorView"),
    PageInfo::placeholder("ButtonHideFlyoutBehavior", "Views/Pages/ButtonHideFlyoutBehaviorView"),
    PageInfo::placeholder("ButtonHideFlyoutOnClickBehavior", "Views/Pages/ButtonHideFlyoutOnClickBehaviorView"),
    PageInfo::placeholder("HideOnKeyPressedBehavior", "Views/Pages/HideOnKeyPressedBehaviorView"),
    PageInfo::placeholder("HideOnLostFocusBehavior", "Views/Pages/HideOnLostFocusBehaviorView"),
    PageInfo::placeholder("SizeChangedTrigger", "Views/Pages/SizeChangedTriggerView"),
    PageInfo::placeholder("FadeInBehavior", "Views/Pages/FadeInBehaviorView"),
    PageInfo::placeholder("DelayedLoadBehavior", "Views/Pages/DelayedLoadBehaviorView"),
    PageInfo::placeholder("DelayedLoadTrigger", "Views/Pages/DelayedLoadTriggerView"),
    PageInfo::placeholder("DelayedShowControlAction", "Views/Pages/DelayedShowControlActionView"),
    PageInfo::placeholder("AnimateOnAttachedBehavior", "Views/Pages/AnimateOnAttachedBehaviorView"),
    PageInfo::placeholder("Async Loading", "Views/Pages/AsyncLoadingView"),
    PageInfo::placeholder("ThemeVariantTrigger", "Views/Pages/ThemeVariantTriggerView"),
    PageInfo::placeholder("ActualThemeVariantChangedTrigger", "Views/Pages/ActualThemeVariantChangedTriggerView"),
    PageInfo::placeholder("ToggleIsExpandedOnDoubleTappedBehavior", "Views/Pages/ToggleIsExpandedOnDoubleTappedBehaviorView"),
    PageInfo::placeholder("TreeView Filter", "Views/Pages/TreeViewFilterView"),
    PageInfo::placeholder("ViewportBehavior", "Views/Pages/ViewportBehaviorView"),
    PageInfo::placeholder("ScrollChangedTrigger", "Views/Pages/ScrollChangedTriggerView"),
    PageInfo::placeholder("ScrollViewerOffsetBehavior", "Views/Pages/ScrollViewerOffsetBehaviorView"),
    PageInfo::placeholder("ScrollToOffsetAction", "Views/Pages/ScrollToOffsetActionView"),
    PageInfo::placeholder("Scroll Behaviors", "Views/Pages/ScrollBehaviorsView"),
    PageInfo::placeholder("Window Interactions", "Views/Pages/WindowInteractionsView"),
    PageInfo::placeholder("Events Behaviors", "Views/Pages/EventsBehaviorsView"),
    PageInfo::placeholder("Focus Event Triggers", "Views/Pages/FocusEventTriggersView"),
    PageInfo::placeholder("Key Event Triggers", "Views/Pages/KeyEventTriggersView"),
    pointer_event_triggers_view::PAGE,
    PageInfo::placeholder("PointerEventsTrigger", "Views/Pages/PointerEventsTriggerView"),
    PageInfo::placeholder("DragLeaveEventTrigger", "Views/Pages/DragLeaveEventTriggerView"),
    PageInfo::placeholder("Gesture Event Triggers", "Views/Pages/GestureEventTriggersView"),
    PageInfo::placeholder("Gesture Advanced Triggers", "Views/Pages/GestureAdvancedTriggersView"),
    PageInfo::placeholder("Pull Gesture Triggers", "Views/Pages/PullGestureTriggersView"),
    PageInfo::placeholder("Scroll Gesture Triggers", "Views/Pages/ScrollGestureTriggersView"),
    PageInfo::placeholder("TouchPad Gesture Triggers", "Views/Pages/TouchPadGestureTriggersView"),
    PageInfo::placeholder("Popup Event Triggers", "Views/Pages/PopupEventTriggersView"),
    PageInfo::placeholder("Focus Triggers", "Views/Pages/FocusTriggersView"),
    PageInfo::placeholder("Key Input Triggers", "Views/Pages/KeyInputTriggersView"),
    PageInfo::placeholder("Pointer Extra Triggers", "Views/Pages/PointerExtraTriggersView"),
    PageInfo::placeholder("Gesture Triggers", "Views/Pages/GestureTriggersView"),
    PageInfo::placeholder("Icon Sample", "Views/Pages/IconView"),
    PageInfo::placeholder("Automation", "Views/Pages/AutomationView"),
    PageInfo::placeholder("ActualThemeVariantChangedBehavior", "Views/Pages/ActualThemeVariantChangedBehaviorView"),
    PageInfo::placeholder("AttachedToLogicalTreeBehavior", "Views/Pages/AttachedToLogicalTreeBehaviorView"),
    PageInfo::placeholder("AttachedToLogicalTreeTrigger", "Views/Pages/AttachedToLogicalTreeTriggerView"),
    PageInfo::placeholder("AttachedToVisualTreeTrigger", "Views/Pages/AttachedToVisualTreeTriggerView"),
    PageInfo::placeholder("BindingBehavior", "Views/Pages/BindingBehaviorView"),
    PageInfo::placeholder("DataContextChangedBehavior", "Views/Pages/DataContextChangedBehaviorView"),
    PageInfo::placeholder("DataContextChangedTrigger", "Views/Pages/DataContextChangedTriggerView"),
    PageInfo::placeholder("DetachedFromLogicalTreeTrigger", "Views/Pages/DetachedFromLogicalTreeTriggerView"),
    PageInfo::placeholder("DetachedFromVisualTreeTrigger", "Views/Pages/DetachedFromVisualTreeTriggerView"),
    PageInfo::placeholder("DisposingBehavior", "Views/Pages/DisposingBehaviorView"),
    PageInfo::placeholder("DisposingTrigger", "Views/Pages/DisposingTriggerView"),
    PageInfo::placeholder("DisposableAction", "Views/Pages/DisposableActionView"),
    PageInfo::placeholder("InitializedBehavior", "Views/Pages/InitializedBehaviorView"),
    PageInfo::placeholder("InitializedTrigger", "Views/Pages/InitializedTriggerView"),
    PageInfo::placeholder("InvokeCommandActionBase", "Views/Pages/InvokeCommandActionBaseView"),
    PageInfo::placeholder("LoadedBehavior", "Views/Pages/LoadedBehaviorView"),
    PageInfo::placeholder("ResourcesChangedBehavior", "Views/Pages/ResourcesChangedBehaviorView"),
    PageInfo::placeholder("ResourcesChangedTrigger", "Views/Pages/ResourcesChangedTriggerView"),
    PageInfo::placeholder("RoutedEventTriggerBase", "Views/Pages/RoutedEventTriggerBaseView"),
    PageInfo::placeholder("UnloadedTrigger", "Views/Pages/UnloadedTriggerView"),
    PageInfo::placeholder("FileSystem", "Views/Pages/FileSystemView"),
    PageInfo::placeholder("Network", "Views/Pages/NetworkView"),
    PageInfo::placeholder("ToggleClassAction", "Views/Pages/ToggleClassActionView"),
    PageInfo::placeholder("LaunchUriAction", "Views/Pages/LaunchUriActionView"),
    PageInfo::placeholder("ScreenshotAction", "Views/Pages/ScreenshotActionView"),
    PageInfo::placeholder("ScrollToControlAction", "Views/Pages/ScrollToControlActionView"),
    PageInfo::placeholder("ConsoleBeepAction", "Views/Pages/ConsoleBeepActionView"),
    PageInfo::placeholder("BreakAction", "Views/Pages/BreakActionView"),
    PageInfo::placeholder("InactivityTrigger", "Views/Pages/InactivityTriggerView"),
];

/// The pages, in the order of the reference.
pub static PAGES: &[PageInfo] = PAGE_TABLE;
