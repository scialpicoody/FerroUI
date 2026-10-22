use super::generated::{LayerVisualHooks, LayerVisualProps};
use super::server::ServerLayerVisual;
use super::visual::CompositionVisualKind;
use super::{CompositionVisual, Compositor, DropShadow, ICompositionObject};
use std::ops::Deref;
use std::rc::Rc;

impl LayerVisualHooks for CompositionVisual {}

/// A container visual whose children are composed into one offscreen layer,
/// which is then drawn with the opacity of the visual and its shadow (WinUI
/// `Microsoft.UI.Composition.LayerVisual`).
///
/// The shadow of a layer visual follows the alpha of the composed children
/// where the drawing context can draw a drop shadow of a layer (the Skia
/// and Vello backends); elsewhere it is the box shadow of the rectangle of
/// the visual, as the shadow of a sprite visual.
///
/// Deviation (winui-composition.md): `Effect` is not ported yet (the effect
/// brushes come with phase 6).
///
/// Like a sprite visual, a layer visual does not clip to its bounds when it
/// is created (`clip_to_bounds` is `false`).
#[derive(Clone)]
pub struct LayerVisual(Rc<CompositionVisual>);

impl Deref for LayerVisual {
    type Target = Rc<CompositionVisual>;

    fn deref(&self) -> &Rc<CompositionVisual> {
        &self.0
    }
}

impl LayerVisual {
    pub(crate) fn new(compositor: &Rc<Compositor>) -> LayerVisual {
        let visual = CompositionVisual::create(
            compositor,
            CompositionVisualKind::Layer(LayerVisualProps::new()),
            || Box::new(ServerLayerVisual::new()),
        );
        // A WinUI visual has no implicit clip.
        visual.set_clip_to_bounds(false);
        LayerVisual(visual)
    }

    /// The handle of `visual` as a layer visual, if it is one.
    pub fn from_visual(visual: &Rc<CompositionVisual>) -> Option<LayerVisual> {
        matches!(visual.kind, CompositionVisualKind::Layer(_)).then(|| LayerVisual(visual.clone()))
    }

    /// The visual as a composition visual.
    pub fn visual(&self) -> &Rc<CompositionVisual> {
        &self.0
    }

    fn props(&self) -> &LayerVisualProps {
        match &self.0.kind {
            CompositionVisualKind::Layer(props) => props,
            _ => unreachable!("a layer visual handle wraps a layer visual"),
        }
    }

    /// The shadow of the composed layer.
    pub fn shadow(&self) -> Option<Rc<DropShadow>> {
        self.props().shadow().and_then(|shadow| shadow.into_any_rc().downcast::<DropShadow>().ok())
    }

    pub fn set_shadow(&self, value: Option<Rc<DropShadow>>) {
        self.props().set_shadow(&*self.0, value.map(|shadow| shadow as Rc<dyn ICompositionObject>))
    }
}
