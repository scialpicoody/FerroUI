# The pages of the behaviors gallery

The gallery is the port of the sample application of Xaml.Behaviors
(`samples/BehaviorsTestApplication` of the reference, commit `45387e32`): 234 pages, one per tab
item of its main view, in the same order. This file lists every page with its state, says how the
crate is laid out, and how a page is ported.

Run the gallery with `cargo run -p behaviors-gallery`. The smoke run shows every page in turn and
exits, with a failure when a page cannot be created:

```text
FERROUI_SMOKE_PAGES=all cargo run -p behaviors-gallery
FERROUI_SMOKE_PAGES=ported FERROUI_SMOKE_PAGE_MS=500 cargo run -p behaviors-gallery
FERROUI_SMOKE_PAGES="CallMethodAction,TimerTriggerView" cargo run -p behaviors-gallery
```

`FERROUI_SMOKE_PAGES` is `all`, `ported`, or headers and view names separated by commas;
`FERROUI_SMOKE_PAGE_MS` is the time each page is shown (150 ms by default). The options of the
other samples work too (`FERROUI_SMOKE_EXIT_MS`, `FERROUI_SMOKE_SCREENSHOT`).

## Layout of the crate

| Path | What it is |
|---|---|
| `lib.rs`, `app.rs`, `App.xaml`, `SideBar.xaml` | the application: resources, the Fluent theme, the styles of the side bar |
| `program.rs`, `smoke.rs` | the desktop entry point and the page-cycling smoke run |
| `markup.rs`, `register_types.rs` | the table of the documents and classes, the namespaces and the registration of the types |
| `Views/` | `MainWindow`, `MainView` (the search box and the tab control of the pages), `SampleView`; `Windows/` holds the windows of two pages |
| `ViewModels/` | `MainWindowViewModel` (the data context every page inherits), `ItemViewModel`, `SampleViewModel`, the drag and drop view models, and the helpers `command.rs` and `items.rs` |
| `Models/`, `Controls/`, `Converters/`, `Dialogs/`, `Behaviors/` | the shared models, controls (`SingleSelectionTabControl`, `CustomControl`, `EditableItem`), the converter, the dialog and the shared drop handlers |
| `Animations/`, `Renderers/`, `Triggers/` | the types of single pages, ported with their pages |
| `Assets/` | the pictures of the reference |
| `pages/` | one module (`<page>.rs`) and one document (`<Page>.xaml`) per page, the table of the pages (`pages/mod.rs`) and the placeholder of a page not ported yet (`pages/placeholder.rs`) |
| `tests/` | the generated per-document tests, the comparison of the compiled markup with the run-time loader, the checks of the page table and the behaviors of some pages |

The namespaces are the ones of the reference with the name of the gallery
(`BehaviorsGallery.ViewModels`, `BehaviorsGallery.Views.Pages`, ...). `BehaviorsGallery`,
`BehaviorsGallery.Controls`, `BehaviorsGallery.Converters` and `BehaviorsGallery.Behaviors` are in
the default XML namespace (`https://github.com/ferroui`), as the reference states for its own, so
documents name their types without a prefix; the behavior crates are in it too.

What a single page uses (its view model, a behavior of the sample, a renderer, a window) is ported
with the page: its module line in the folder (`ViewModels/mod.rs`, `Behaviors/mod.rs`, ...) is
written as a comment naming its owner page, and the page registers the type in its own entry of the
page table. What several pages share is ported already and registered by the shared tables.

## How to port a page

1. Convert the page with the helper of the port (outside the repository):

   ```text
   python3 -I /Volumes/1TB-macOS/behaviors-port-notes/convert_gallery_page.py <worktree> <ViewName> --apply
   ```

   It writes `pages/<ViewName>.xaml` (the document of the reference with the names of the port:
   the XML namespace `https://github.com/ferroui`, `BehaviorsGallery` for the namespaces of the
   sample, `FerroUI.Xaml.Interactions.*` for the namespaces of the behaviors, `.xaml`,
   `ferres://`, `FerroProperty`), writes `pages/<view_name>.rs` (the class of the page and its entry
   of the table) when it does not exist, changes the two lines of the page in `pages/mod.rs`, and
   prints the code-behind of the reference when it does more than load the document, and every
   line that still names the upstream framework.
2. The two lines of `pages/mod.rs` (the helper changes them; by hand it is the same):
   - the module line: `// pub mod <view_name>;` becomes `pub mod <view_name>;`;
   - the table line: `PageInfo::placeholder("<Header>", "Views/Pages/<ViewName>"),` becomes
     `<view_name>::PAGE,`.

   No other line of a shared file changes, except the module line of a type the page owns (step 4).
3. The page module (`pages/<view_name>.rs`), as the ported pages show (`call_method_action_view.rs`
   is the plain form):
   - the class: `#[repr(C)] pub struct <ViewName> { base: UserControl }`,
     `user_control_class!(<ViewName>)`, `ferro_class_info!(<ViewName> { new: <ViewName>::new })`,
     `xaml_class!(<ViewName>, "/pages/<ViewName>.xaml")`, and `new()` that calls
     `initialize_component()`;
   - `pub(crate) const TYPES: &[&TypeInfo]`: the class of the page and every class only the page
     uses (a behavior of the sample);
   - `pub(crate) const PAGE: PageInfo`: the header (the header of the tab item of the reference),
     the reference file, `create`, `TYPES`, the classes with a document (`&X::XAML_CLASS`), the
     markup types only the page uses (`<X as MarkupTyped>::MARKUP`) and the function that registers
     their value types (`ValueTypes::register_reference::<X>()`, casts to contracts), or
     `no_value_types`.
4. The code-behind:
   - an override of the reference (`OnPointerPressed`, ...) is the implementation of the trait of
     its level, written instead of the one `user_control_class!` writes: declare the class with
     `ferro_class!` and `ferro_impl_classes!` without that trait, and implement it by hand
     (`event_trigger_behavior_view.rs`);
   - `DataContext = new XViewModel()` is `this.set_data_context(Some(XViewModel::new() as BoxedValue))`
     after `initialize_component()` (`drag_drop_commands_view.rs`);
   - an event handler a document names (`Click="OnClick"`) is a method in the `markup` part of
     `ferro_class_info!` (`fn OnClick(Option<BoxedValue>, Rc<dyn IRoutedEventArgs>) => ..`);
   - `this.FindControl<T>("Name")` is `this.find_control::<T>("Name")`.
5. A view model only the page uses: remove the comment marker of its line in `ViewModels/mod.rs`,
   port the file there, and register it in the `PAGE` of the page (`markup_types` and
   `register_value_types`), as `drag_drop_commands_view.rs` does for its view model.
6. View models follow `ViewModels/`:
   - the struct is declared outside a macro (the scanner of the build must read it: a type a
     document names needs a public path), holds `base: ViewModelBase`, and
     `reactive_object!(X, base)` makes it a reactive object;
   - a `[Reactive]` property of the reference is a `RefCell<T>` field with its accessors written by
     `super::reactive_accessors!(X { field, set_field, "Field": T; })`;
   - a command (`ReactiveCommand.Create(..)`) is a `Command` of `ViewModels/command.rs`, held as
     `Rc<dyn ICommand>`;
   - an `ObservableCollection<T>` of view models is an `ItemList` of `ViewModels/items.rs` (a
     notifying list of untyped values, which the item actions and the drop handlers change), exposed
     to markup as `ItemsSource`;
   - the type is declared to markup with `ferro_markup_type!` (properties, methods for
     `CallMethodAction`, `notify_property_changed`), and implements `PartialEq` by identity.
7. Build and run (only the orchestrator builds; a page agent writes code it believes compiles):
   `cargo build -p behaviors-gallery`, `cargo test -p behaviors-gallery`, and
   `FERROUI_SMOKE_PAGES=<Header> cargo run -p behaviors-gallery`. The build compiles every
   document under `pages/`; a refusal names the document and the line.
8. Update the row of the page in the table below: the state `ported`, and what the notes need
   (a deviation, a part left out). A deviation in code or markup is written at its place as
   `Deviation (DEVIATIONS.md, Behaviors gallery): <why>`.

Rules: no name of the upstream framework anywhere under `samples/` (file names, documents,
strings, comments; the text of a page that names it says FerroUI), `ferro_thread_local!` and never
`thread_local!`, no emojis.

## Groups for parallel work

The placeholders are split into three contiguous ranges of the table with about the same number of
reference lines (document and code-behind). The two lists of `pages/mod.rs` carry a comment at the
start of each group, so that the changes of two groups never touch neighbouring lines. Pages that
share a type of a single group stay in one group (the editable lists, the animation builder pages,
the execute command pages).

| Group | Pages | Placeholders | Reference lines |
|---|---|---|---|
| A | 1 to 86 | 73 | 4255 |
| B | 87 to 144 | 56 | 4451 |
| C | 145 to 234 | 89 | 4490 |

## Known blockers in the behavior crates

- `ferroui-xaml-interactions-reactiveui` is not a dependency of the gallery yet: its type model has
  a cast registration the scanner of the build cannot read (the generic `register_cast` of
  `register_interaction_casts` in `src/Behaviors/FerroUI.Xaml.Interactions.ReactiveUI/interaction_trigger_behavior.rs`),
  and with it the compiler of the documents refuses every question about casts. Pages:
  `InteractionTriggerBehaviorView`, `ReactiveNavigationView` (and `ObservableTriggerBehaviorView`
  once it needs the crate).
- `IDropHandler` and `IDragHandler` have no markup metadata
  (`src/Behaviors/FerroUI.Xaml.Interactions.DragAndDrop/markup_types/mod.rs`, which registers their
  nullable forms only), so a drop handler declared as a resource and assigned with
  `Handler="{StaticResource ..}"` is refused ("has no metadata"). Pages: `TypedDragBehaviorView`,
  `DragAndDropView`, `EditableDraggableListBoxView`, `EditableDragTreeViewView`,
  `FileDropHandlerView`.

## The pages

The groups, the reference lines (the document and its code-behind) and the notes of each page. A
note `own view model` or `owns` names a type only that page uses, whose module line is a comment
until the page is ported.

| # | Header | Reference file | Group | Lines | State | Notes |
|---|---|---|---|---|---|---|
| 1 | SelectAllOnFocusBehavior | `Views/Pages/SelectAllOnFocusBehaviorView` | A | 40 | in progress |  |
| 2 | FocusNextElementAction | `Views/Pages/FocusNextElementActionView` | A | 38 | in progress |  |
| 3 | WindowDragMoveBehavior | `Views/Pages/WindowDragMoveBehaviorView` | A | 46 | in progress | owns: WindowDemo (commented line in its folder); port the code-behind |
| 4 | TiltEffectBehavior | `Views/Pages/TiltEffectBehaviorView` | A | 53 | in progress |  |
| 5 | OrbitEffectBehavior | `Views/Pages/OrbitEffectBehaviorView` | A | 59 | in progress |  |
| 6 | ParallaxBehavior | `Views/Pages/ParallaxBehaviorView` | A | 70 | in progress |  |
| 7 | ConditionalAction | `Views/Pages/ConditionalActionView` | A | 51 | in progress |  |
| 8 | SwitchCaseAction | `Views/Pages/SwitchCaseActionView` | A | 67 | in progress |  |
| 9 | SwitchCaseBehavior | `Views/Pages/SwitchCaseBehaviorView` | A | 64 | in progress |  |
| 10 | ConditionalBehavior | `Views/Pages/ConditionalBehaviorView` | A | 41 | in progress |  |
| 11 | LoseFocusOnEnterBehavior | `Views/Pages/LoseFocusOnEnterBehaviorView` | A | 34 | in progress |  |
| 12 | AutoScrollToBottomBehavior | `Views/Pages/AutoScrollToBottomBehaviorView` | A | 57 | in progress | port the code-behind |
| 13 | CallMethodAction | `Views/Pages/CallMethodActionView` | A | 63 | ported | EventTriggerBehavior (Click) with CallMethodAction and ChangePropertyAction on the view model; tested (`tests/page_behaviors.rs`) |
| 14 | Debounce & Throttle | `Views/Pages/DebounceThrottleActionView` | A | 74 | in progress | port the code-behind |
| 15 | ObservableStreamBehavior | `Views/Pages/ObservableStreamBehaviorView` | A | 61 | in progress |  |
| 16 | AsyncActionGroup | `Views/Pages/AsyncActionGroupView` | A | 56 | in progress |  |
| 17 | FocusTrapBehavior | `Views/Pages/FocusTrapBehaviorView` | A | 48 | in progress |  |
| 18 | ScreenReaderAnnounceAction | `Views/Pages/ScreenReaderAnnounceActionView` | A | 49 | in progress |  |
| 19 | GlobalHotkeyBehavior | `Views/Pages/GlobalHotkeyBehaviorView` | A | 43 | in progress |  |
| 20 | VisualDebugBehavior | `Views/Pages/VisualDebugBehaviorView` | A | 49 | in progress |  |
| 21 | LogAction | `Views/Pages/LogActionView` | A | 62 | in progress |  |
| 22 | NetworkStatusTrigger | `Views/Pages/NetworkStatusTriggerView` | A | 50 | in progress |  |
| 23 | ClipboardMonitorBehavior | `Views/Pages/ClipboardMonitorBehaviorView` | A | 52 | in progress | port the code-behind |
| 24 | ChangePropertyAction | `Views/Pages/ChangePropertyActionView` | A | 55 | ported | ChangePropertyAction with a dynamic resource value; tested |
| 25 | DataTriggerBehavior | `Views/Pages/DataTriggerBehaviorView` | A | 64 | ported | DataTriggerBehavior with comparison conditions on an element binding |
| 26 | DataTriggerBehavior Advanced | `Views/Pages/DataTriggerBehaviorAdvancedView` | A | 51 | in progress |  |
| 27 | MultiDataTriggerBehavior | `Views/Pages/MultiDataTriggerBehaviorView` | A | 87 | in progress |  |
| 28 | BindingTriggerBehavior | `Views/Pages/BindingTriggerBehaviorView` | A | 64 | in progress |  |
| 29 | ValueChangedTriggerBehavior | `Views/Pages/ValueChangedTriggerBehaviorView` | A | 40 | ported | ValueChangedTriggerBehavior on a stream binding (`Values^`) |
| 30 | PropertyChangedTrigger | `Views/Pages/PropertyChangedTriggerView` | A | 39 | in progress |  |
| 31 | EventTriggerBehavior | `Views/Pages/EventTriggerBehaviorView` | A | 56 | ported | code-behind: `OnPointerPressed` override (`InputElementImpl`); shows `Views/SampleView` |
| 32 | TimerTrigger | `Views/Pages/TimerTriggerView` | A | 45 | ported | TimerTrigger with CallMethodAction |
| 33 | TaskCompletedTrigger | `Views/Pages/TaskCompletedTriggerView` | A | 45 | in progress |  |
| 34 | LoadedTrigger | `Views/Pages/LoadedTriggerView` | A | 39 | in progress |  |
| 35 | InvokeCommandAction | `Views/Pages/InvokeCommandActionView` | A | 62 | ported | InvokeCommandAction; deviation: no `Unit.Default` command parameter; tested |
| 36 | AutoFocusBehavior | `Views/Pages/AutoFocusBehaviorView` | A | 34 | in progress |  |
| 37 | AutoSelectBehavior | `Views/Pages/AutoSelectBehaviorView` | A | 34 | in progress |  |
| 38 | TextBoxValidationBehavior | `Views/Pages/TextBoxValidationBehaviorView` | A | 43 | in progress |  |
| 39 | NumericUpDownValidationBehavior | `Views/Pages/NumericUpDownValidationBehaviorView` | A | 46 | in progress |  |
| 40 | NotNullValidationRule | `Views/Pages/NotNullValidationRuleView` | A | 45 | in progress |  |
| 41 | PropertyValidationBehavior | `Views/Pages/PropertyValidationBehaviorView` | A | 56 | in progress |  |
| 42 | SliderValidationBehavior | `Views/Pages/SliderValidationBehaviorView` | A | 60 | in progress |  |
| 43 | DatePickerValidationBehavior | `Views/Pages/DatePickerValidationBehaviorView` | A | 47 | in progress |  |
| 44 | ComboBoxValidationBehavior | `Views/Pages/ComboBoxValidationBehaviorView` | A | 46 | in progress |  |
| 45 | FocusControlBehavior | `Views/Pages/FocusControlBehaviorView` | A | 37 | in progress |  |
| 46 | FocusOnAttachedBehavior | `Views/Pages/FocusOnAttachedBehaviorView` | A | 34 | ported | a behavior of the Custom crate |
| 47 | FocusOnAttachedToVisualTreeBehavior | `Views/Pages/FocusOnAttachedToVisualTreeBehaviorView` | A | 34 | in progress |  |
| 48 | FocusOnPointerMovedBehavior | `Views/Pages/FocusOnPointerMovedBehaviorView` | A | 34 | in progress |  |
| 49 | FocusOnPointerPressedBehavior | `Views/Pages/FocusOnPointerPressedBehaviorView` | A | 34 | in progress |  |
| 50 | FocusOnVisibleBehavior | `Views/Pages/FocusOnVisibleBehaviorView` | A | 38 | in progress |  |
| 51 | FocusSelectedItemBehavior | `Views/Pages/FocusSelectedItemBehaviorView` | A | 34 | in progress |  |
| 52 | RoutedEventTriggerBehavior | `Views/Pages/RoutedEventTriggerBehaviorView` | A | 61 | in progress |  |
| 53 | ChangeFerroPropertyAction | `Views/Pages/ChangeFerroPropertyActionView` | A | 59 | in progress | the reference names the view and its text after the upstream framework (`ChangeFerroPropertyAction` here) |
| 54 | ViewModel Property Actions | `Views/Pages/ViewModelPropertyActionsView` | A | 66 | in progress |  |
| 55 | Custom Action | `Views/Pages/CustomActionView` | A | 67 | ported | ShowPointerPositionBehavior and PopupAction (custom action) |
| 56 | Custom Behavior | `Views/Pages/CustomBehaviorView` | A | 68 | ported | DragControlBehavior (custom behavior), also through a style setter with a BehaviorCollectionTemplate |
| 57 | ButtonClickEventTriggerBehavior | `Views/Pages/ButtonClickEventTriggerBehaviorView` | A | 53 | in progress |  |
| 58 | ClickEventTrigger | `Views/Pages/ClickEventTriggerView` | A | 336 | in progress |  |
| 59 | Advanced Behavior | `Views/Pages/AdvancedView` | A | 66 | in progress |  |
| 60 | Add/RemoveClassAction | `Views/Pages/AddRemoveClassActionView` | A | 64 | ported | AddClassAction and RemoveClassAction from data triggers on the bounds |
| 61 | RemoveElementAction | `Views/Pages/RemoveElementActionView` | A | 45 | in progress |  |
| 62 | RemoveItemInListBoxAction | `Views/Pages/RemoveItemInListBoxActionView` | A | 45 | in progress |  |
| 63 | RemoveItemInItemsControlAction | `Views/Pages/RemoveItemInItemsControlActionView` | A | 45 | in progress |  |
| 64 | AddItemToItemsControlAction | `Views/Pages/AddItemToItemsControlActionView` | A | 64 | in progress |  |
| 65 | InsertItemToItemsControlAction | `Views/Pages/InsertItemToItemsControlActionView` | A | 60 | in progress |  |
| 66 | ObjectTemplate | `Views/Pages/ObjectTemplateView` | A | 82 | in progress |  |
| 67 | ClearItemsControlAction | `Views/Pages/ClearItemsControlActionView` | A | 45 | in progress |  |
| 68 | AddRangeAction | `Views/Pages/AddRangeActionView` | A | 72 | in progress |  |
| 69 | RemoveRangeAction | `Views/Pages/RemoveRangeActionView` | A | 45 | in progress |  |
| 70 | MoveItemInItemsControlAction | `Views/Pages/MoveItemInItemsControlActionView` | A | 56 | in progress |  |
| 71 | MoveElementToPanelAction | `Views/Pages/MoveElementActionsView` | A | 76 | in progress |  |
| 72 | CollectionChangedTrigger | `Views/Pages/CollectionChangedTriggerBehaviorView` | A | 55 | in progress |  |
| 73 | ItemsControl Container Triggers | `Views/Pages/ItemsControlContainerTriggersView` | A | 106 | in progress |  |
| 74 | Remove Items Sample | `Views/Pages/RemoveItemsSampleView` | A | 61 | in progress |  |
| 75 | AdaptiveBehavior | `Views/Pages/AdaptiveBehaviorView` | A | 155 | ported | the Responsive crate; the shared `ClassesToStringConverter` as a markup extension |
| 76 | AspectRatioBehavior | `Views/Pages/AspectRatioBehaviorView` | A | 76 | in progress | uses the shared `ClassesToStringConverter` |
| 77 | SplitView | `Views/Pages/SplitViewStateBehaviorView` | A | 75 | in progress |  |
| 78 | EditableListBox | `Views/Pages/EditableListBoxView` | A | 44 | in progress | uses the shared `EditableItem` control |
| 79 | EditableDraggableListBox | `Views/Pages/EditableDraggableListBoxView` | A | 83 | in progress | BLOCKED: a drop handler as a resource (IDropHandler has no markup metadata); port the code-behind |
| 80 | EditableListBox DoubleTap | `Views/Pages/EditableDoubleTappedListBoxView` | A | 44 | in progress | uses the shared `EditableItem` control |
| 81 | EditableTree | `Views/Pages/EditableTreeViewView` | A | 45 | in progress | uses the shared `EditableItem` control |
| 82 | EditableTree DoubleTap | `Views/Pages/EditableDoubleTappedTreeViewView` | A | 44 | in progress | uses the shared `EditableItem` control |
| 83 | Editable Drag TreeView | `Views/Pages/EditableDragTreeViewView` | A | 67 | in progress | BLOCKED: a drop handler as a resource (IDropHandler has no markup metadata); port the code-behind |
| 84 | Sliding Animation | `Views/Pages/SlidingAnimationView` | A | 77 | in progress |  |
| 85 | Fade Animation | `Views/Pages/FadeAnimationView` | A | 49 | ported | attached properties of the Custom crate (`custom:FadeAnimation`): an animation page |
| 86 | Scale Animation | `Views/Pages/ScaleAnimationView` | A | 91 | in progress |  |
| 87 | Rotate Animation | `Views/Pages/RotateAnimationView` | B | 105 | placeholder |  |
| 88 | Entrance Animations | `Views/Pages/EntranceAnimationsView` | B | 198 | placeholder |  |
| 89 | Attention Animations | `Views/Pages/AttentionAnimationsView` | B | 117 | placeholder |  |
| 90 | Exit Animations | `Views/Pages/ExitAnimationsView` | B | 195 | placeholder |  |
| 91 | Framer Motion Animations | `Views/Pages/FramerMotionAnimationsView` | B | 157 | placeholder |  |
| 92 | Special Animations | `Views/Pages/SpecialAnimationsView` | B | 34 | placeholder |  |
| 93 | Animation Behaviors | `Views/Pages/AnimationBehaviorView` | B | 69 | placeholder |  |
| 94 | Animation Completed Trigger | `Views/Pages/AnimationCompletedTriggerView` | B | 49 | placeholder |  |
| 95 | StartAnimationAction | `Views/Pages/StartAnimationActionView` | B | 50 | placeholder |  |
| 96 | AnimationBuilder | `Views/Pages/AnimationBuilderView` | B | 40 | placeholder | own view model: CustomAnimatorViewModel (commented line in `ViewModels/mod.rs`); the reference names the upstream framework: write FerroUI; port the code-behind |
| 97 | StartBuiltAnimationAction | `Views/Pages/StartBuiltAnimationActionView` | B | 41 | placeholder | own view model: CustomAnimatorViewModel (commented line in `ViewModels/mod.rs`); port the code-behind |
| 98 | RunAnimationTrigger | `Views/Pages/RunAnimationTriggerView` | B | 40 | placeholder | own view model: CustomAnimatorViewModel (commented line in `ViewModels/mod.rs`); the reference names the upstream framework: write FerroUI; port the code-behind |
| 99 | FluidMoveBehavior | `Views/Pages/FluidMoveBehaviorView` | B | 51 | placeholder | own view model: FluidMoveBehaviorViewModel (commented line in `ViewModels/mod.rs`); port the code-behind |
| 100 | SelectingItemsControlBehavior | `Views/Pages/SelectingItemsControlBehaviorView` | B | 40 | placeholder |  |
| 101 | ListBox Selection Behaviors | `Views/Pages/ListBoxSelectionBehaviorsView` | B | 103 | placeholder |  |
| 102 | BehaviorCollectionTemplate | `Views/Pages/BehaviorCollectionTemplateView` | B | 70 | placeholder | the reference names the upstream framework: write FerroUI |
| 103 | ActionCollectionTemplate | `Views/Pages/ActionCollectionTemplateView` | B | 90 | ported | BehaviorCollectionTemplate and ActionCollectionTemplate in styles, ChangeFerroPropertyAction, PopupAction |
| 104 | TemplateBinding | `Views/Pages/TemplateBindingView` | B | 30 | placeholder | uses the shared `Controls/CustomControl` (its styles are in `App.xaml`) |
| 105 | OneTime Binding | `Views/Pages/OneTimeBinding` | B | 55 | placeholder |  |
| 106 | Pointer Triggers | `Views/Pages/PointerTriggersView` | B | 51 | placeholder | own view model: PointerTriggersViewModel (commented line in `ViewModels/mod.rs`); port the code-behind |
| 107 | KeyGestureTrigger | `Views/Pages/KeyGestureTriggerView` | B | 40 | placeholder | own view model: KeyGestureTriggerViewModel (commented line in `ViewModels/mod.rs`); port the code-behind |
| 108 | ExecuteCommand Behaviors | `Views/Pages/ExecuteCommandBehaviorsView` | B | 176 | placeholder | own view model: ExecuteCommandBehaviorsViewModel (commented line in `ViewModels/mod.rs`); port the code-behind |
| 109 | ButtonExecuteCommandOnKeyDownBehavior | `Views/Pages/ButtonExecuteCommandOnKeyDownBehaviorView` | B | 36 | placeholder | own view model: ExecuteCommandBehaviorsViewModel (commented line in `ViewModels/mod.rs`); port the code-behind |
| 110 | Storage Provider | `Views/Pages/StorageProviderView` | B | 203 | placeholder | port the code-behind |
| 111 | Picker CanExecuteCommand | `Views/Pages/PickerCanExecuteCommandView` | B | 188 | placeholder |  |
| 112 | Clipboard | `Views/Pages/ClipboardView` | B | 52 | placeholder | the reference names the upstream framework: write FerroUI |
| 113 | GetClipboardDataAction | `Views/Pages/GetClipboardDataActionView` | B | 38 | placeholder |  |
| 114 | GetClipboardFormatsAction | `Views/Pages/GetClipboardFormatsActionView` | B | 38 | placeholder |  |
| 115 | SetClipboardDataObjectAction | `Views/Pages/SetClipboardDataObjectActionView` | B | 38 | placeholder |  |
| 116 | FocusControlAction | `Views/Pages/FocusControlActionView` | B | 39 | placeholder |  |
| 117 | Hide/ShowControlAction | `Views/Pages/HideShowControlActionView` | B | 46 | placeholder |  |
| 118 | ShowHideFlyoutAction | `Views/Pages/ShowHideFlyoutActionView` | B | 48 | placeholder |  |
| 119 | ShowContextMenuAction | `Views/Pages/ShowContextMenuActionView` | B | 42 | placeholder |  |
| 120 | ContextDialog | `Views/Pages/ContextDialogView` | B | 60 | placeholder |  |
| 121 | ShowHidePopupAction | `Views/Pages/ShowHidePopupActionView` | B | 47 | placeholder |  |
| 122 | ShowOn Event Behaviors | `Views/Pages/ShowOnEventBehaviorsView` | B | 138 | placeholder |  |
| 123 | SetEnabledAction | `Views/Pages/SetEnabledActionView` | B | 44 | placeholder |  |
| 124 | ShowNotificationAction | `Views/Pages/ShowNotificationActionView` | B | 49 | placeholder |  |
| 125 | Dialog Actions | `Views/Pages/DialogActionView` | B | 52 | placeholder | uses the shared `Dialogs/SimpleDialog` |
| 126 | File Drop Handler | `Views/Pages/FileDropHandlerView` | B | 55 | placeholder | owns: FilesDropHandler (commented line in its folder); BLOCKED: a drop handler as a resource (IDropHandler has no markup metadata) |
| 127 | Upload File | `Views/Pages/UploadFileView` | B | 102 | placeholder |  |
| 128 | ContentControlFilesDropBehavior | `Views/Pages/ContentControlFilesDropBehaviorView` | B | 51 | placeholder |  |
| 129 | Drag and Drop | `Views/Pages/DragAndDropView` | B | 426 | placeholder | owns: ItemsDataGridDropHandler, ManagedItemsListBoxDropHandler, NodesListBoxDropHandler (commented line in its folder); uses DataGrid, which the framework has not: leave those parts out (deviation); BLOCKED: a drop handler as a resource (IDropHandler has no markup metadata); port the code-behind |
| 130 | DragDrop Commands | `Views/Pages/DragDropCommandsView` | B | 50 | ported | drag and drop: DragDropCommandsBehavior and ContextDragBehavior; own view model (`ViewModels/drag_drop_commands_view_model.rs`, registered by the page) |
| 131 | Files Preview | `Views/Pages/FilesPreviewView` | B | 51 | placeholder |  |
| 132 | TypedDragBehavior | `Views/Pages/TypedDragBehaviorView` | B | 66 | placeholder | BLOCKED: a drop handler as a resource (IDropHandler has no markup metadata); shared parts ported (`DragAndDropSampleViewModel`, `ItemsListBoxDropHandler`) |
| 133 | Draggable | `Views/Pages/DraggableView` | B | 232 | placeholder | own view model: DraggableViewModel (commented line in `ViewModels/mod.rs`); owns: DraggableCustomStyles (commented line in its folder); the reference names the upstream framework: write FerroUI; port the code-behind |
| 134 | Mouse Drag Behaviors | `Views/Pages/MouseDragBehaviorView` | B | 69 | placeholder | port the code-behind |
| 135 | Canvas Drag Behavior | `Views/Pages/CanvasDragBehaviorView` | B | 72 | placeholder |  |
| 136 | Auto Scroll During Drag | `Views/Pages/AutoScrollDuringDragBehaviorView` | B | 45 | placeholder |  |
| 137 | Drag Between Panels | `Views/Pages/DragBetweenPanelsView` | B | 54 | placeholder |  |
| 138 | Cursor Behavior | `Views/Pages/CursorView` | B | 67 | placeholder | own view model: CursorViewModel (commented line in `ViewModels/mod.rs`); port the code-behind |
| 139 | Drawn Cursor | `Views/Pages/DrawnCursorView` | B | 51 | placeholder | own view model: DrawnCursorViewModel (commented line in `ViewModels/mod.rs`); port the code-behind |
| 140 | IfElseTrigger | `Views/Pages/IfElseTriggerView` | B | 49 | placeholder |  |
| 141 | ExecuteScriptAction | `Views/Pages/ExecuteScriptActionView` | B | 53 | placeholder | ExecuteScriptAction has no port (no scripting): port the rest, state the deviation |
| 142 | Observable Trigger | `Views/ReactiveUI/ObservableTriggerBehaviorView` | B | 46 | placeholder |  |
| 143 | InteractionTriggerBehavior | `Views/ReactiveUI/InteractionTriggerBehaviorView` | B | 45 | placeholder | own view model: InteractionTriggerBehaviorViewModel (commented line in `ViewModels/mod.rs`); BLOCKED: needs ferroui-xaml-interactions-reactiveui (not a dependency yet); port the code-behind |
| 144 | Flyout Binding | `Views/Pages/FlyoutBindingView` | B | 48 | placeholder |  |
| 145 | KeyTrigger | `Views/Pages/KeyTriggerView` | C | 49 | placeholder |  |
| 146 | ToolTip Helpers | `Views/Pages/ToolTipHelpersView` | C | 62 | placeholder |  |
| 147 | WriteableBitmap | `Views/Pages/WriteableBitmapView` | C | 123 | placeholder | owns: SampleWriteableBitmapRenderer (commented line in its folder) |
| 148 | RenderTargetBitmap | `Views/Pages/RenderTargetBitmapView` | C | 67 | placeholder | owns: RotatingSquareRenderer, SolidSquareRenderer (commented line in its folder) |
| 149 | LaunchUriOrFileAction | `Views/Pages/LaunchUriOrFileActionView` | C | 39 | placeholder |  |
| 150 | Reactive Navigation | `Views/ReactiveUI/ReactiveNavigationView` | C | 84 | placeholder | own view model: DetailPageViewModel, DetailsViewModel, HomePageViewModel, ReactiveNavigationViewModel (commented line in `ViewModels/mod.rs`); BLOCKED: needs ferroui-xaml-interactions-reactiveui (not a dependency yet); the reference names the upstream framework: write FerroUI; port the code-behind |
| 151 | Transitions Behavior | `Views/Pages/TransitionsBehaviorView` | C | 60 | placeholder |  |
| 152 | Transitions Actions | `Views/Pages/TransitionsActionsView` | C | 75 | placeholder |  |
| 153 | Transitions Trigger | `Views/Pages/TransitionsChangedTriggerView` | C | 75 | placeholder |  |
| 154 | Notifications | `Views/Pages/NotificationsView` | C | 70 | placeholder |  |
| 155 | Carousel Navigation | `Views/Pages/CarouselNavigationView` | C | 56 | placeholder |  |
| 156 | CarouselSelectionChangedTrigger | `Views/Pages/CarouselSelectionChangedTriggerView` | C | 42 | placeholder |  |
| 157 | AutoCompleteBox | `Views/Pages/AutoCompleteBoxView` | C | 49 | placeholder |  |
| 158 | FocusAutoCompleteBoxTextBoxBehavior | `Views/Pages/FocusAutoCompleteBoxTextBoxBehaviorView` | C | 36 | placeholder |  |
| 159 | TabControl Navigation | `Views/Pages/TabControlNavigationView` | C | 60 | placeholder |  |
| 160 | TabControl Selection Trigger | `Views/Pages/TabControlSelectionChangedTriggerView` | C | 42 | placeholder |  |
| 161 | Theme Variant | `Views/Pages/ThemeVariantView` | C | 58 | placeholder | port the code-behind |
| 162 | Screens | `Views/Pages/ScreenView` | C | 86 | placeholder |  |
| 163 | BindPointerOverBehavior | `Views/Pages/BindPointerOverBehaviorView` | C | 35 | placeholder |  |
| 164 | BindTagToVisualRootDataContextBehavior | `Views/Pages/BindTagToVisualRootDataContextBehaviorView` | C | 45 | placeholder |  |
| 165 | BoundsObserverBehavior | `Views/Pages/BoundsObserverBehaviorView` | C | 39 | placeholder |  |
| 166 | HideAttachedFlyoutBehavior | `Views/Pages/HideAttachedFlyoutBehaviorView` | C | 48 | placeholder |  |
| 167 | ButtonHideFlyoutBehavior | `Views/Pages/ButtonHideFlyoutBehaviorView` | C | 48 | placeholder |  |
| 168 | ButtonHideFlyoutOnClickBehavior | `Views/Pages/ButtonHideFlyoutOnClickBehaviorView` | C | 45 | placeholder |  |
| 169 | HideOnKeyPressedBehavior | `Views/Pages/HideOnKeyPressedBehaviorView` | C | 40 | placeholder |  |
| 170 | HideOnLostFocusBehavior | `Views/Pages/HideOnLostFocusBehaviorView` | C | 40 | placeholder |  |
| 171 | SizeChangedTrigger | `Views/Pages/SizeChangedTriggerView` | C | 40 | placeholder |  |
| 172 | FadeInBehavior | `Views/Pages/FadeInBehaviorView` | C | 34 | placeholder |  |
| 173 | DelayedLoadBehavior | `Views/Pages/DelayedLoadBehaviorView` | C | 34 | placeholder |  |
| 174 | DelayedLoadTrigger | `Views/Pages/DelayedLoadTriggerView` | C | 38 | placeholder |  |
| 175 | DelayedShowControlAction | `Views/Pages/DelayedShowControlActionView` | C | 38 | placeholder |  |
| 176 | AnimateOnAttachedBehavior | `Views/Pages/AnimateOnAttachedBehaviorView` | C | 45 | placeholder |  |
| 177 | Async Loading | `Views/Pages/AsyncLoadingView` | C | 44 | placeholder |  |
| 178 | ThemeVariantTrigger | `Views/Pages/ThemeVariantTriggerView` | C | 57 | placeholder | the reference names the upstream framework: write FerroUI |
| 179 | ActualThemeVariantChangedTrigger | `Views/Pages/ActualThemeVariantChangedTriggerView` | C | 47 | placeholder |  |
| 180 | ToggleIsExpandedOnDoubleTappedBehavior | `Views/Pages/ToggleIsExpandedOnDoubleTappedBehaviorView` | C | 50 | placeholder |  |
| 181 | TreeView Filter | `Views/Pages/TreeViewFilterView` | C | 57 | placeholder |  |
| 182 | ViewportBehavior | `Views/Pages/ViewportBehaviorView` | C | 47 | placeholder |  |
| 183 | ScrollChangedTrigger | `Views/Pages/ScrollChangedTriggerView` | C | 44 | placeholder |  |
| 184 | ScrollViewerOffsetBehavior | `Views/Pages/ScrollViewerOffsetBehaviorView` | C | 43 | placeholder |  |
| 185 | ScrollToOffsetAction | `Views/Pages/ScrollToOffsetActionView` | C | 44 | placeholder |  |
| 186 | Scroll Behaviors | `Views/Pages/ScrollBehaviorsView` | C | 116 | placeholder |  |
| 187 | Window Interactions | `Views/Pages/WindowInteractionsView` | C | 48 | placeholder | owns: SampleWindow (commented line in its folder) |
| 188 | Events Behaviors | `Views/Pages/EventsBehaviorsView` | C | 34 | placeholder |  |
| 189 | Focus Event Triggers | `Views/Pages/FocusEventTriggersView` | C | 37 | placeholder |  |
| 190 | Key Event Triggers | `Views/Pages/KeyEventTriggersView` | C | 43 | placeholder |  |
| 191 | Pointer Event Triggers | `Views/Pages/PointerEventTriggersView` | C | 52 | ported | the triggers of the Events crate |
| 192 | PointerEventsTrigger | `Views/Pages/PointerEventsTriggerView` | C | 69 | placeholder | port the code-behind |
| 193 | DragLeaveEventTrigger | `Views/Pages/DragLeaveEventTriggerView` | C | 77 | placeholder | port the code-behind |
| 194 | Gesture Event Triggers | `Views/Pages/GestureEventTriggersView` | C | 46 | placeholder |  |
| 195 | Gesture Advanced Triggers | `Views/Pages/GestureAdvancedTriggersView` | C | 82 | placeholder | port the code-behind |
| 196 | Pull Gesture Triggers | `Views/Pages/PullGestureTriggersView` | C | 82 | placeholder | port the code-behind |
| 197 | Scroll Gesture Triggers | `Views/Pages/ScrollGestureTriggersView` | C | 78 | placeholder | port the code-behind |
| 198 | TouchPad Gesture Triggers | `Views/Pages/TouchPadGestureTriggersView` | C | 78 | placeholder | port the code-behind |
| 199 | Popup Event Triggers | `Views/Pages/PopupEventTriggersView` | C | 50 | placeholder |  |
| 200 | Focus Triggers | `Views/Pages/FocusTriggersView` | C | 37 | placeholder |  |
| 201 | Key Input Triggers | `Views/Pages/KeyInputTriggersView` | C | 43 | placeholder |  |
| 202 | Pointer Extra Triggers | `Views/Pages/PointerExtraTriggersView` | C | 43 | placeholder |  |
| 203 | Gesture Triggers | `Views/Pages/GestureTriggersView` | C | 57 | placeholder |  |
| 204 | Icon Sample | `Views/Pages/IconView` | C | 49 | placeholder | own view model: IconViewModel (commented line in `ViewModels/mod.rs`); the reference names the upstream framework: write FerroUI; port the code-behind |
| 205 | Automation | `Views/Pages/AutomationView` | C | 57 | placeholder | the reference names the upstream framework: write FerroUI |
| 206 | ActualThemeVariantChangedBehavior | `Views/Pages/ActualThemeVariantChangedBehaviorView` | C | 45 | placeholder | owns: ActualThemeVariantChangedMessageBehavior (commented line in its folder) |
| 207 | AttachedToLogicalTreeBehavior | `Views/Pages/AttachedToLogicalTreeBehaviorView` | C | 38 | placeholder | owns: AttachedToLogicalTreeMessageBehavior (commented line in its folder) |
| 208 | AttachedToLogicalTreeTrigger | `Views/Pages/AttachedToLogicalTreeTriggerView` | C | 35 | placeholder |  |
| 209 | AttachedToVisualTreeTrigger | `Views/Pages/AttachedToVisualTreeTriggerView` | C | 35 | placeholder |  |
| 210 | BindingBehavior | `Views/Pages/BindingBehaviorView` | C | 38 | placeholder |  |
| 211 | DataContextChangedBehavior | `Views/Pages/DataContextChangedBehaviorView` | C | 38 | placeholder | owns: DataContextChangedMessageBehavior (commented line in its folder) |
| 212 | DataContextChangedTrigger | `Views/Pages/DataContextChangedTriggerView` | C | 35 | placeholder |  |
| 213 | DetachedFromLogicalTreeTrigger | `Views/Pages/DetachedFromLogicalTreeTriggerView` | C | 49 | placeholder |  |
| 214 | DetachedFromVisualTreeTrigger | `Views/Pages/DetachedFromVisualTreeTriggerView` | C | 49 | placeholder |  |
| 215 | DisposingBehavior | `Views/Pages/DisposingBehaviorView` | C | 39 | placeholder | owns: DisposingMessageBehavior (commented line in its folder) |
| 216 | DisposingTrigger | `Views/Pages/DisposingTriggerView` | C | 41 | placeholder | owns: PointerPressedDisposingTrigger (commented line in its folder) |
| 217 | DisposableAction | `Views/Pages/DisposableActionView` | C | 39 | placeholder | owns: DisposableActionBehavior (commented line in its folder) |
| 218 | InitializedBehavior | `Views/Pages/InitializedBehaviorView` | C | 39 | placeholder | owns: InitializedMessageBehavior (commented line in its folder) |
| 219 | InitializedTrigger | `Views/Pages/InitializedTriggerView` | C | 40 | placeholder |  |
| 220 | InvokeCommandActionBase | `Views/Pages/InvokeCommandActionBaseView` | C | 36 | placeholder |  |
| 221 | LoadedBehavior | `Views/Pages/LoadedBehaviorView` | C | 37 | placeholder | owns: LoadedMessageBehavior (commented line in its folder) |
| 222 | ResourcesChangedBehavior | `Views/Pages/ResourcesChangedBehaviorView` | C | 46 | placeholder | owns: ResourcesChangedMessageBehavior (commented line in its folder); port the code-behind |
| 223 | ResourcesChangedTrigger | `Views/Pages/ResourcesChangedTriggerView` | C | 48 | placeholder | port the code-behind |
| 224 | RoutedEventTriggerBase | `Views/Pages/RoutedEventTriggerBaseView` | C | 40 | placeholder |  |
| 225 | UnloadedTrigger | `Views/Pages/UnloadedTriggerView` | C | 49 | placeholder |  |
| 226 | FileSystem | `Views/Pages/FileSystemView` | C | 93 | placeholder | own view model: FileSystemViewModel (commented line in `ViewModels/mod.rs`) |
| 227 | Network | `Views/Pages/NetworkView` | C | 70 | placeholder | own view model: NetworkViewModel (commented line in `ViewModels/mod.rs`) |
| 228 | ToggleClassAction | `Views/Pages/ToggleClassActionView` | C | 40 | placeholder |  |
| 229 | LaunchUriAction | `Views/Pages/LaunchUriActionView` | C | 34 | placeholder | the reference names the upstream framework: write FerroUI |
| 230 | ScreenshotAction | `Views/Pages/ScreenshotActionView` | C | 34 | placeholder |  |
| 231 | ScrollToControlAction | `Views/Pages/ScrollToControlActionView` | C | 40 | placeholder |  |
| 232 | ConsoleBeepAction | `Views/Pages/ConsoleBeepActionView` | C | 34 | placeholder |  |
| 233 | BreakAction | `Views/Pages/BreakActionView` | C | 34 | placeholder |  |
| 234 | InactivityTrigger | `Views/Pages/InactivityTriggerView` | C | 34 | placeholder |  |
