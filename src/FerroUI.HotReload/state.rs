//! Conservative state transfer for named, unambiguous controls in one reload boundary.
use ferroui_base::{BoxedValue, Ref, StyledElement, Vector};
use ferroui_controls::{primitives::SelectingItemsControl, SelectionMode};
use ferroui_controls::{ScrollViewer, TextBox};
use ferroui_markup_xaml::FromXamlObject;
use std::collections::BTreeMap;
#[derive(Default)]
pub(crate) struct ViewState(BTreeMap<Vec<String>, Option<Entry>>);
struct Entry {
    kind: &'static ferroui_base::TypeInfo,
    focused: bool,
    text: Option<(Option<String>, i32, i32)>,
    scroll: Option<Vector>,
    selection: Option<SelectionState>,
}
struct SelectionState {
    items: Vec<Option<BoxedValue>>,
    anchor: Option<Option<BoxedValue>>,
    mode: SelectionMode,
}
fn unique_index(items: &[Option<BoxedValue>], value: &Option<BoxedValue>) -> Option<i32> {
    let mut matches = items.iter().enumerate().filter(|(_, item)| *item == value);
    let index = matches.next()?.0;
    if matches.next().is_some() {
        None
    } else {
        i32::try_from(index).ok()
    }
}
impl SelectionState {
    fn capture(control: &SelectingItemsControl) -> Option<Self> {
        let model = control.selection();
        // A caller-owned model is durable application state, not UI state to copy.
        model.as_internal_selection_model()?;
        let items: Vec<_> = control.items_view().iter().collect();
        let selected = model
            .selected_items()
            .iter()
            .filter(|item| unique_index(&items, item).is_some())
            .collect();
        let anchor = usize::try_from(model.anchor_index())
            .ok()
            .and_then(|index| items.get(index))
            .filter(|item| unique_index(&items, item).is_some())
            .cloned();
        Some(Self {
            items: selected,
            anchor,
            mode: control.selection_mode(),
        })
    }
    fn restore(&self, control: &SelectingItemsControl, before: &str, after: &str, name: &str) {
        let properties = [
            SelectingItemsControl::selected_index_property().as_property(),
            SelectingItemsControl::selected_item_property().as_property(),
            SelectingItemsControl::selected_items_property().as_property(),
            SelectingItemsControl::selection_property().as_property(),
            SelectingItemsControl::selected_value_property().as_property(),
            SelectingItemsControl::selected_value_binding_property().as_property(),
        ];
        for property in properties {
            if control.has_local_value_binding(property)
                || control.is_set(property)
                || authored(after, name, property.name()).is_some()
                || !unchanged_unbound(before, after, name, property.name())
            {
                return;
            }
        }
        let model = control.selection();
        if model.as_internal_selection_model().is_none()
            || model.count() != 0
            || self.mode != control.selection_mode()
        {
            return;
        }
        let items: Vec<_> = control.items_view().iter().collect();
        let indexes: Vec<_> = self
            .items
            .iter()
            .filter_map(|item| unique_index(&items, item))
            .collect();
        if indexes.is_empty() {
            return;
        }
        struct EndBatch(std::rc::Rc<dyn ferroui_controls::selection::ISelectionModel>);
        impl Drop for EndBatch {
            fn drop(&mut self) {
                self.0.end_batch_update();
            }
        }
        model.begin_batch_update();
        let _batch = EndBatch(model.clone());
        for index in indexes {
            model.select(index);
        }
        if let Some(index) = self
            .anchor
            .as_ref()
            .and_then(|anchor| unique_index(&items, anchor))
        {
            model.set_anchor_index(index);
        }
    }
}
fn controls(root: &BoxedValue, source: &str) -> Vec<(Vec<String>, Ref<StyledElement>)> {
    let Some(root) = Ref::<StyledElement>::from_xaml_object(root) else {
        return Vec::new();
    };
    let mut result = Vec::new();
    let mut pending = vec![(root, Vec::<String>::new(), None, true)];
    while let Some((element, mut scope_path, parent_scope, is_root)) = pending.pop() {
        let own_scope = ferroui_base::controls::NameScope::get_name_scope(&element);
        let identity = element
            .name()
            .and_then(|name| crate::identity::identity(source, &name));
        if !is_root && own_scope.is_some() && own_scope != parent_scope {
            // An unnamed nested scope has no stable identity across reconstructions.
            let Some(owner) = &identity else { continue };
            scope_path.push(owner.clone());
        }
        let scope = own_scope.or(parent_scope);
        pending.extend(
            element
                .logical_children()
                .to_vec()
                .into_iter()
                .map(|child| (child, scope_path.clone(), scope.clone(), false)),
        );
        if let Some(identity) = identity {
            scope_path.push(identity);
            result.push((scope_path, element));
        }
    }
    result
}
use crate::identity::authored;
fn unchanged_unbound(before: &str, after: &str, name: &str, property: &str) -> bool {
    let old = authored(before, name, property);
    let new = authored(after, name, property);
    old == new && !new.as_deref().is_some_and(|value| value.starts_with('{'))
}
impl ViewState {
    pub(crate) fn capture(root: Option<&BoxedValue>, source: &str) -> Self {
        let mut state = Self::default();
        for (identity, element) in root.map(|root| controls(root, source)).unwrap_or_default() {
            let entry = Entry {
                kind: element.get_type(),
                focused: element
                    .cast::<ferroui_base::input::InputElement>()
                    .is_some_and(|input| input.is_focused()),
                text: element
                    .cast::<TextBox>()
                    .map(|text| (text.text(), text.selection_start(), text.selection_end())),
                scroll: element.cast::<ScrollViewer>().map(|scroll| scroll.offset()),
                selection: element
                    .cast::<SelectingItemsControl>()
                    .and_then(|control| SelectionState::capture(&control)),
            };
            state
                .0
                .entry(identity)
                .and_modify(|entry| *entry = None)
                .or_insert(Some(entry));
        }
        state
    }
    pub(crate) fn restore(&self, root: &BoxedValue, before: &str, after: &str) {
        let elements = controls(root, after);
        let has_new_focus = elements.iter().any(|(_, element)| {
            element
                .cast::<ferroui_base::input::InputElement>()
                .is_some_and(|input| input.is_focused())
        });
        let mut counts = BTreeMap::new();
        for (identity, _) in &elements {
            *counts.entry(identity.clone()).or_insert(0) += 1;
        }
        for (identity, element) in elements {
            let Some(name) = element.name() else { continue };
            if counts.get(&identity) != Some(&1) {
                continue;
            }
            let Some(Some(entry)) = self.0.get(&identity) else {
                continue;
            };
            if !std::ptr::eq(entry.kind, element.get_type()) {
                continue;
            }
            if let (Some(text), Some((value, start, end))) =
                (element.cast::<TextBox>(), &entry.text)
            {
                if unchanged_unbound(before, after, &name, "Text")
                    && !text.is_set(TextBox::text_property())
                {
                    text.set_text(value.as_deref());
                }
                // Restore selection only against identical text; never writes through a Text binding.
                if !text.is_set(TextBox::selection_start_property())
                    && !text.is_set(TextBox::selection_end_property())
                    && text.text() == *value
                    && unchanged_unbound(before, after, &name, "SelectionStart")
                    && unchanged_unbound(before, after, &name, "SelectionEnd")
                {
                    text.set_selection_start(*start);
                    text.set_selection_end(*end);
                }
            }
            if let (Some(scroll), Some(offset)) = (element.cast::<ScrollViewer>(), entry.scroll) {
                if !scroll.is_set(ScrollViewer::offset_property())
                    && unchanged_unbound(before, after, &name, "Offset")
                {
                    scroll.set_offset(offset);
                }
            }
            if entry.focused && !has_new_focus {
                if let Some(input) = element.cast::<ferroui_base::input::InputElement>() {
                    input.focus();
                }
            }
            if let (Some(control), Some(selection)) =
                (element.cast::<SelectingItemsControl>(), &entry.selection)
            {
                selection.restore(&control, before, after, &name);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferroui_base::threading::Dispatcher;
    use ferroui_markup_xaml::RuntimeXamlLoaderDocument;
    const LIST_SOURCE: &str = r#"<ListBox xmlns="https://github.com/ferroui" Name="choices"/>"#;
    fn list(items: &[&str], multiple: bool) -> (BoxedValue, Ref<ferroui_controls::ListBox>) {
        ferroui_controls::register_types();
        let control = ferroui_controls::ListBox::new();
        control.set_name(Some("choices".into()));
        if multiple {
            control.set_selection_mode(SelectionMode::MULTIPLE);
        }
        for item in items {
            control
                .items()
                .add(Some(std::rc::Rc::new(item.to_string())));
        }
        (std::rc::Rc::new(control.clone()), control)
    }
    #[test]
    fn selection_follows_unique_items_across_reordering() {
        let _scope = Dispatcher::unit_test_scope();
        let (before, old) = list(&["a", "b", "c"], true);
        old.selection().select(0);
        old.selection().select(2);
        old.selection().set_anchor_index(2);
        let state = ViewState::capture(Some(&before), LIST_SOURCE);
        let (after, new) = list(&["c", "b", "a"], true);
        state.restore(&after, LIST_SOURCE, LIST_SOURCE);
        assert_eq!(new.selection().selected_indexes().to_vec(), vec![0, 2]);
        assert_eq!(new.selection().anchor_index(), 0);
    }
    #[test]
    fn ambiguous_or_removed_selected_items_are_not_restored_by_index() {
        let _scope = Dispatcher::unit_test_scope();
        let (before, old) = list(&["a", "b"], false);
        old.set_selected_index(1);
        let state = ViewState::capture(Some(&before), LIST_SOURCE);
        for items in [&["b", "b"][..], &["a", "c"][..]] {
            let (after, new) = list(items, false);
            state.restore(&after, LIST_SOURCE, LIST_SOURCE);
            assert_eq!(new.selected_index(), -1);
        }
    }
    #[test]
    fn authored_selection_and_removal_override_restored_selection() {
        let _scope = Dispatcher::unit_test_scope();
        let authored = LIST_SOURCE.replace("/>", " SelectedIndex=\"-1\"/>");
        for (before_source, after_source) in [
            (LIST_SOURCE, authored.as_str()),
            (authored.as_str(), LIST_SOURCE),
        ] {
            let (before, old) = list(&["a", "b"], false);
            old.set_selected_index(1);
            let state = ViewState::capture(Some(&before), before_source);
            let (after, new) = list(&["a", "b"], false);
            state.restore(&after, before_source, after_source);
            assert_eq!(new.selected_index(), -1);
        }
    }
    #[test]
    fn direct_two_way_selection_binding_is_never_written_by_restoration() {
        use ferroui_base::data::{BindingMode, IndexerBinding};
        let _scope = Dispatcher::unit_test_scope();
        let (before, old) = list(&["a", "b"], false);
        old.set_selected_index(1);
        let state = ViewState::capture(Some(&before), LIST_SOURCE);
        let (after, new) = list(&["a", "b"], false);
        let (_, model) = list(&["a", "b"], false);
        let property = SelectingItemsControl::selected_index_property().as_property();
        let binding = IndexerBinding::new(model.clone().upcast(), property, BindingMode::TwoWay);
        let expression = new.bind_binding(property, &binding);
        assert!(new.has_local_value_binding(property));
        state.restore(&after, LIST_SOURCE, LIST_SOURCE);
        assert_eq!(new.selected_index(), -1);
        assert_eq!(model.selected_index(), -1);
        new.set_selected_index(1);
        assert_eq!(model.selected_index(), 1);
        expression.dispose();
        assert!(!new.has_local_value_binding(property));
    }
    fn source(attributes: &str) -> String {
        format!(
            r#"<TextBox xmlns="https://github.com/ferroui"
          xmlns:hr="https://github.com/ferroui/hot-reload"
          xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006"
          mc:Ignorable="hr" hr:Key="draft" {attributes}/ >"#
        )
        .replace("/ >", "/>")
    }
    fn load(source: &str) -> (BoxedValue, Ref<TextBox>) {
        ferroui_controls::register_types();
        ferroui_markup_xaml::register_types();
        let prepared = crate::identity::runtime_source(source).unwrap();
        let root = ferroui_markup_xaml_loader::FerroRuntimeXamlLoader::load_document(
            RuntimeXamlLoaderDocument::with_base_uri(None, &prepared),
            None,
        )
        .unwrap();
        let control = Ref::<TextBox>::from_xaml_object(&root).unwrap();
        (root, control)
    }
    #[test]
    fn unnamed_development_key_restores_text_and_selection() {
        let _scope = Dispatcher::unit_test_scope();
        let source = source("");
        let (before, old) = load(&source);
        old.set_text(Some("draft text"));
        old.set_selection_start(2);
        old.set_selection_end(5);
        let state = ViewState::capture(Some(&before), &source);
        let (after, new) = load(&source);
        state.restore(&after, &source, &source);
        assert_eq!(new.text().as_deref(), Some("draft text"));
        assert_eq!((new.selection_start(), new.selection_end()), (2, 5));
    }
    #[test]
    fn authored_changes_and_removals_take_precedence() {
        let _scope = Dispatcher::unit_test_scope();
        let old_source = source(r#"Text="initial""#);
        let (before, old) = load(&old_source);
        old.set_text(Some("user edit"));
        let state = ViewState::capture(Some(&before), &old_source);
        for attributes in [r#"Text="new markup""#, ""] {
            let new_source = source(attributes);
            let (after, new) = load(&new_source);
            let authored = new.text();
            state.restore(&after, &old_source, &new_source);
            assert_eq!(new.text(), authored);
        }
    }
    #[test]
    fn restoration_does_not_write_through_two_way_binding() {
        let _scope = Dispatcher::unit_test_scope();
        let old_source = source("");
        let (before, old) = load(&old_source);
        old.set_text(Some("stale UI"));
        let state = ViewState::capture(Some(&before), &old_source);
        let new_source = source(r#"Text="{Binding Text, Mode=TwoWay}""#);
        let (after, new) = load(&new_source);
        let model = TextBox::new();
        model.set_text(Some("model"));
        new.set_data_context(Some(std::rc::Rc::new(model.clone())));
        state.restore(&after, &old_source, &new_source);
        assert_eq!(model.text().as_deref(), Some("model"));
        assert_eq!(new.text().as_deref(), Some("model"));
        new.set_text(Some("fresh edit"));
        assert_eq!(model.text().as_deref(), Some("fresh edit"));
    }
    #[test]
    fn identical_names_in_distinct_named_scopes_do_not_cross() {
        use ferroui_base::controls::{NameScope, NameScopeRef};
        use ferroui_controls::StackPanel;
        let _scope = Dispatcher::unit_test_scope();
        fn tree(reverse: bool) -> (BoxedValue, Ref<TextBox>, Ref<TextBox>) {
            let root = StackPanel::new();
            let mut texts = Vec::new();
            let mut panels = Vec::new();
            for name in ["left", "right"] {
                let panel = StackPanel::new();
                panel.set_name(Some(name.into()));
                NameScope::set_name_scope(
                    &panel,
                    Some(NameScopeRef(std::rc::Rc::new(NameScope::new()))),
                );
                let text = TextBox::new();
                text.set_name(Some("edit".into()));
                panel
                    .children()
                    .add(text.clone().upcast::<ferroui_controls::Control>());
                texts.push(text);
                panels.push(panel);
            }
            if reverse {
                panels.reverse();
            }
            for panel in panels {
                root.children()
                    .add(panel.upcast::<ferroui_controls::Control>());
            }
            (std::rc::Rc::new(root), texts[0].clone(), texts[1].clone())
        }
        let source = r#"<StackPanel><StackPanel Name="left"><TextBox Name="edit"/></StackPanel><StackPanel Name="right"><TextBox Name="edit"/></StackPanel></StackPanel>"#;
        let (before, left, right) = tree(false);
        left.set_text(Some("left draft"));
        right.set_text(Some("right draft"));
        let state = ViewState::capture(Some(&before), source);
        let (after, left, right) = tree(true);
        state.restore(&after, source, source);
        assert_eq!(left.text().as_deref(), Some("left draft"));
        assert_eq!(right.text().as_deref(), Some("right draft"));
    }
}
