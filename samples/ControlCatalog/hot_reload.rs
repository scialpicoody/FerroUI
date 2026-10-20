//! Manifest-backed catalog sessions. Cached-reference/handler pages require explicit refresh adapters.
use ferroui_base::threading::{DispatcherPriority, DispatcherTimer};
use ferroui_base::{Ref, TypeInfo, WeakRef};
use ferroui_controls::ContentPage;
use ferroui_hot_reload::{
    ComponentReloadTarget, FileWatcher, HotReloadSession, ReloadManifest, ReloadTarget,
};
use ferroui_markup_xaml::RuntimeXamlLoaderConfiguration;
use std::{cell::RefCell, collections::BTreeSet, rc::Rc, time::Duration};

struct CatalogSession {
    session: HotReloadSession,
    watcher: FileWatcher,
    targets: Vec<(Rc<ComponentReloadTarget>, WeakRef<ContentPage>)>,
    timer: Option<Rc<DispatcherTimer>>,
    bound: BTreeSet<String>,
}
ferroui_base::ferro_thread_local! {
    static SESSION: RefCell<Option<Rc<RefCell<CatalogSession>>>> = const { RefCell::new(None) };
}
fn configuration() -> RuntimeXamlLoaderConfiguration {
    let mut configuration = RuntimeXamlLoaderConfiguration::default();
    configuration.local_assembly = Some(&crate::register_types::ASSEMBLY);
    configuration.use_compiled_bindings_by_default = true;
    configuration
}
pub(crate) fn track(root: Ref<ContentPage>, class: &'static TypeInfo, path: &str) {
    if ![
        "/Pages/BorderPage.xaml",
        "/Pages/SliderPage.xaml",
        "/Pages/ProgressBarPage.xaml",
        "/Pages/CheckBoxPage.xaml",
        "/Pages/CanvasPage.xaml",
        "/Pages/RadioButtonPage.xaml",
        "/Pages/TextBlockPage.xaml",
        "/Pages/ViewboxPage.xaml",
        "/Pages/ExpanderPage.xaml",
    ]
    .contains(&path)
    {
        return;
    }
    SESSION.with(|slot| {
        let mut slot = slot.borrow_mut();
        let uri = crate::markup::document_uri(path);
        let context = slot
            .get_or_insert_with(|| {
                let mut manifest =
                    ReloadManifest::parse(include_str!(env!("FERROUI_HOT_RELOAD_MANIFEST")))
                        .expect("catalog development manifest");
                let overlay_root = std::env::var_os("FERROUI_HOT_RELOAD_ROOT");
                if let Some(directory) = &overlay_root {
                    let original_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                        .canonicalize()
                        .expect("catalog source root");
                    for source in manifest.files.values_mut() {
                        let relative = source
                            .strip_prefix(&original_root)
                            .expect("catalog source path");
                        let overlay = std::path::PathBuf::from(&directory).join(relative);
                        if overlay.exists() {
                            *source = overlay;
                        }
                    }
                }
                let watcher = if overlay_root.is_some() {
                    // An explicit partial source overlay must not collide with
                    // discovery of the original file under the same document URI.
                    FileWatcher::with_baseline(
                        manifest.files.clone(),
                        Duration::from_millis(60),
                        manifest.sources.clone(),
                    )
                } else {
                    manifest.watcher(Duration::from_millis(60))
                };
                for source in manifest.sources.values_mut() {
                    *source = normalize(source.clone()).expect("valid embedded geometry");
                }
                let session = manifest
                    .session()
                    .expect("register catalog reload documents");
                let context = Rc::new(RefCell::new(CatalogSession {
                    session,
                    watcher,
                    targets: Vec::new(),
                    timer: None,
                    bound: BTreeSet::new(),
                }));
                let weak = Rc::downgrade(&context);
                let timer = DispatcherTimer::with_callback(
                    Duration::from_millis(60),
                    DispatcherPriority::BACKGROUND,
                    move |timer| {
                        let Some(context) = weak.upgrade() else {
                            timer.stop();
                            return;
                        };
                        let mut context = context.borrow_mut();
                        context.targets.retain(|(target, _)| target.is_alive());
                        let latest = context.watcher.changes.try_iter().last();
                        if let Some(mut batch) = latest {
                            for file in &mut batch.files {
                                file.contents =
                                    std::mem::replace(&mut file.contents, Err(String::new()))
                                        .and_then(normalize);
                            }
                            context.session.submit_files(batch);
                        }
                        if let Some(outcome) = context.session.apply_pending() {
                            if let Some(top_level) = context.targets.iter().find_map(|(_, root)| {
                                root.upgrade().and_then(|root| {
                                    ferroui_controls::TopLevel::get_top_level(Some(&root))
                                })
                            }) {
                                ferroui_hot_reload::record_frame_timing(&top_level, &outcome);
                            }
                            eprintln!("Catalog reload: {outcome:?}");
                        }
                    },
                );
                context.borrow_mut().timer = Some(timer);
                context
            })
            .clone();
        let mut context = context.borrow_mut();
        let canonical = ferroui_base::utilities::Uri::try_create(
            &uri,
            ferroui_base::utilities::UriKind::Absolute,
        )
        .unwrap();
        if context.bound.insert(canonical.absolute_uri().to_owned()) {
            context
                .session
                .bind_class(class, uri.clone(), configuration())
                .expect("bind catalog class");
        }
        let baseline = context.session.sources()[canonical.absolute_uri()].clone();
        let mut target = ComponentReloadTarget::content_page(&root, uri, baseline, configuration())
            .with_owned_root_property(ferroui_base::layout::Layoutable::width_property())
            .with_owned_root_property(ferroui_base::layout::Layoutable::height_property())
            .with_owned_root_property(ferroui_base::layout::Layoutable::margin_property());
        if let Some(page) = root.cast::<crate::pages::ExpanderPage>() {
            target = target.with_refresh(page.reload_refresh());
        }
        let target = Rc::new(target);
        let erased: Rc<dyn ReloadTarget> = target.clone();
        context.session.register_target(&erased);
        context.targets.push((target, root.downgrade()));
        eprintln!("Catalog reload tracking: {path}");
    });
}

#[path = "build/placeholder_geometry.rs"]
mod placeholder_geometry;
include!(concat!(env!("OUT_DIR"), "/hot_reload_geometries.rs"));
fn normalize(mut source: String) -> Result<String, String> {
    if cfg!(feature = "placeholder-branding") {
        for (key, data) in RELOAD_GEOMETRIES {
            placeholder_geometry::replace_geometry(&mut source, key, data)?;
        }
    }
    Ok(source)
}
