# The WinUI 3 composition API

This document is the inventory and the plan of the port of the WinUI 3 composition API
(`Microsoft.UI.Composition`) and of the XAML pieces that sit on it in microsoft-ui-xaml
(`ConnectedAnimationService`, `ConnectedAnimation`, `ElementCompositionPreview`, the `Translation`
facade of `UIElement`) to FerroUI. It records what FerroUI already has, what the port adds, in which
order, and every place where the result differs from WinUI.

## 1. Sources

- **The compositor of FerroUI.** `src/FerroUI.Base/rendering/composition` is the port of the
  compositor of the upstream framework: `Compositor`, `CompositionVisual` and its container,
  solid colour, surface, draw list and custom visuals, `CompositionPropertySet`,
  `ExpressionAnimation`, the key frame animations of the schema, implicit animations, animation
  groups, the expression language, the composition brushes, the server/transport split and the
  drawing surfaces. Its object model is generated from `scripts/composition-schema.xml` by
  `scripts/generate_composition.py`. The port builds on it and does not duplicate any of it.
- **The connected animations of the upstream framework.** `src/FerroUI.Controls/animation` is the
  port of the upstream `ConnectedAnimationService`, which flies a control over the overlay layer
  with UI-thread animations. The WinUI port replaces its engine with the composition engine of
  WinUI and keeps its public members (section 5.1).
- **microsoft-ui-xaml**, branch `winui3/main`, commit `7e4b0c2096be402f51cc8f7d203ebf67cff183d0`
  (local checkout `/Users/wieslawsoltes/GitHub/microsoft-ui-xaml`, read-only, MIT). The XAML-side
  code is ported from it:
  - `dxaml/xcp/components/animation/ConnectedAnimation.cpp`, `ConnectedAnimationService.cpp` and
    their headers: the state machine, the rectangles, the snapshot brushes and the sprite
    animations of a connected animation;
  - `dxaml/xcp/dxaml/lib/ConnectedAnimation*_partial.cpp`, `GravityConnectedAnimationConfiguration_partial.cpp`,
    `DirectConnectedAnimationConfiguration_partial.cpp`, `BasicConnectedAnimationConfiguration_partial.h`:
    the public classes and the three configurations;
  - `dxaml/xcp/core/core/elements/ConnectedAnimationRoot.cpp`: the layer the animations fly in;
  - `dxaml/xcp/dxaml/lib/ElementCompositionPreview.cpp`: the `ElementCompositionPreview` statics.
- **`Microsoft.UI.Composition`** is closed source (it ships in the Windows App SDK). It is ported
  from its public API reference (learn.microsoft.com, the `Microsoft.UI.Composition` namespace of
  the Windows App SDK 1.6) and from how microsoft-ui-xaml uses it (`DCompTreeHost`, `HWCompNode`,
  the connected animation code). Where the reference does not say how something behaves, the port
  says what it chose in the deviations (section 7).

The upstream framework name never appears in `src`, `samples` or `tests`; this page and the
`scripts/` folder may name it.

## 2. How the WinUI types map onto FerroUI

| WinUI | FerroUI |
|---|---|
| `Microsoft.UI.Composition` | `ferroui_base::rendering::composition` (the compositor and its objects) |
| `Microsoft.UI.Xaml.Hosting.ElementCompositionPreview` | `ferroui_controls::hosting::ElementCompositionPreview` (it reaches `ScrollViewer`, a control); the visual-tree half stays `ferroui_base::rendering::composition::ElementComposition` |
| `Microsoft.UI.Xaml.Media.Animation.ConnectedAnimation*` | `ferroui_controls::animation` |
| `UIElement` | `Visual` (the element that has a composition visual) |
| handoff visual (`GetElementVisual`) | the `CompositionDrawListVisual` the compositing renderer creates for a visual |
| hand-in visual (`SetElementChildVisual`) | the child composition visual of a visual, shown after its children |
| `ICompositionEasingFunction` | `ferroui_base::animation::easings::Easing` (`Rc<dyn IEasing>` inside); `CompositionEasingFunction` has the WinUI factories |
| `Vector2`, `Vector3`, `Vector4`, `Quaternion`, `Matrix3x2`, `Matrix4x4` | `ferroui_base::numerics` (the expression language types); visual properties keep the types of the compositor (`Vector`, `Vector3D`, `Matrix`) |
| `TimeSpan` | `std::time::Duration` |
| events (`Completed`) | handler lists with a disposable registration, as elsewhere in FerroUI |

Property names used by `StartAnimation` and by expressions are the WinUI names (`Offset`,
`Opacity`, `Scale`, `Translation`, `Offset.X`, ...), so an expression or a target written for
WinUI works unchanged.

## 3. Inventory

Status: **has** (present with the WinUI members and behaviour), **partial** (present, some
members or behaviour missing, said in the notes), **missing** (not in FerroUI before this port).
The last column is the phase of section 4 that brings the type to **has**, or the decision when
it does not.

### 3.1 Objects, compositor, property sets

| Type / member | Status | Notes | Phase |
|---|---|---|---|
| `CompositionObject.Compositor`, `Dispose`, `ImplicitAnimations` | has | `CompositionObject` | - |
| `CompositionObject.StartAnimation`, `StopAnimation`, `StartAnimationGroup`, `StopAnimationGroup` | partial | Whole properties only; a sub-channel target (`Offset.X`, `Scale.XY`) is not accepted. | 1 |
| `CompositionObject.Properties` | missing | Every WinUI object has its own property set, reachable by expressions as `obj.Name`. | 3 |
| `CompositionObject.Comment` | missing | | 3 |
| `CompositionObject.TryGetAnimationController`, `AnimationController` | missing | Pause, resume, progress and playback rate of a running animation. | 3 |
| `CompositionPropertySet` `Insert*`, `TryGet*` | has | Scalar, vector, matrix, quaternion, colour, boolean. | - |
| `Compositor.Create*` for the types below | partial | The factories of the existing types exist; the new types bring theirs. | 1-6 |
| `Compositor.CreatePropertySet` | partial | `CompositionPropertySet::new(compositor)`; the factory is added. | 1 |
| `Compositor.CreateScopedBatch`, `GetCommitBatch`, `CompositionScopedBatch`, `CompositionCommitBatch` | missing | Completion of a group of animations: `End`, `Suspend`, `Resume`, `IsActive`, `IsEnded`, `Completed`. | 1 |
| `Compositor.RequestCommitAsync` | has | `request_commit_async`. | - |
| `Compositor.GlobalPlaybackRate`, `MinGlobalPlaybackRate`, `MaxGlobalPlaybackRate` | missing | | 3 |

### 3.2 Visuals

| Type / member | Status | Notes | Phase |
|---|---|---|---|
| `Visual.Offset`, `Size`, `Opacity`, `IsVisible`, `AnchorPoint`, `CenterPoint`, `RotationAngle`, `Orientation`, `Scale`, `Parent` | has | `CompositionVisual`; `IsVisible` is `visible`. | - |
| `Visual.Translation` (property set entry in WinUI) | has | A real animatable property of every visual. | - |
| `Visual.TransformMatrix` | partial | A 2D `Matrix`, not a `Matrix4x4`; the 3D part is ignored by a 2D renderer. | deviation |
| `Visual.Clip` (`CompositionClip`) | partial | Only a geometry clip, internal. The WinUI clip objects are added. | 1 |
| `Visual.RotationAngleInDegrees`, `RotationAxis` | missing | Degrees: a view of `RotationAngle`. Axis: only the Z axis rotates in 2D. | 2 |
| `Visual.RelativeOffsetAdjustment`, `RelativeSizeAdjustment` | missing | Offset and size relative to the size of the parent. | 2 |
| `Visual.IsHitTestVisible`, `IsPixelSnappingEnabled`, `BorderMode`, `BackfaceVisibility`, `CompositeMode`, `ParentForTransform` | missing | `ParentForTransform` is close to the existing adorned visual. Backface and 3D modes have no meaning in a 2D renderer. | 2 / deviation |
| `ContainerVisual.Children` (`InsertAbove`, `InsertBelow`, `InsertAtTop`, `InsertAtBottom`, `Remove`, `RemoveAll`, `Count`) | has | `CompositionVisualCollection`. | - |
| `SpriteVisual` (`Brush`, `Shadow`) | missing | The existing `CompositionSolidColorVisual` is a sprite with a colour. | 1 |
| `ShapeVisual` (`Shapes`, `ViewBox`) | missing | | 2 |
| `LayerVisual` (`Effect`, `Shadow`) | missing | Children composed into a layer, then the effect or shadow. | 2 |
| `RedirectVisual` (`Source`) | missing | Shows another visual's subtree. | 2 |

### 3.3 Brushes and surfaces

| Type / member | Status | Notes | Phase |
|---|---|---|---|
| `CompositionColorBrush` (`Color`) | has | `CompositionSolidColorBrush`; `create_color_brush` is added under the WinUI name. | 1 |
| `CompositionGradientBrush` (`ColorStops`, `ExtendMode`) | partial | Stops and spread method exist. Missing: `MappingMode`, `InterpolationSpace`, the brush transform (`AnchorPoint`, `CenterPoint`, `Offset`, `RotationAngle`, `Scale`, `TransformMatrix`). | 2 |
| `CompositionLinearGradientBrush` (`StartPoint`, `EndPoint`) | partial | As relative points, not `Vector2` with a mapping mode. | 2 |
| `CompositionRadialGradientBrush` (`EllipseCenter`, `EllipseRadius`, `GradientOriginOffset`) | partial | `Center`, `RadiusX/Y`, `GradientOrigin` of the upstream brush. | 2 |
| `CompositionSurfaceBrush` (`Surface`, `Stretch`, alignment ratios, `BitmapInterpolationMode`, transform, `SnapToPixels`) | missing | | 1 |
| `CompositionNineGridBrush` | missing | | 2 |
| `CompositionMaskBrush` (`Source`, `Mask`) | missing | An opacity mask of the drawing context. | 2 |
| `CompositionBackdropBrush`, `CompositionHostBackdropBrush` | missing | Needs the pixels behind the visual. | 6 |
| `CompositionEffectBrush`, `CompositionEffectFactory`, `CompositionEffectSourceParameter` | missing | The Win2D effect graph. | 6 (scoped) |
| `ICompositionSurface` | partial | `CompositionSurface` (bitmap surfaces of the upstream framework). | 1 |
| `CompositionVisualSurface` (`SourceVisual`, `SourceOffset`, `SourceSize`) | missing | A live image of a visual subtree; the connected animation uses it. | 1 |
| `CompositionDrawingSurface` | partial | The upstream class takes imported GPU images; WinUI draws with Direct2D interop, which has no counterpart. | deviation |
| `LoadedImageSurface` (XAML) | missing | A surface from a bitmap; the connected animation uses it for the frozen source image. | 1 |

### 3.4 Shapes and geometry

| Type / member | Status | Notes | Phase |
|---|---|---|---|
| `CompositionShape` (`CenterPoint`, `Offset`, `RotationAngle`, `RotationAngleInDegrees`, `Scale`, `TransformMatrix`) | missing | | 2 |
| `CompositionSpriteShape` (`Geometry`, `FillBrush`, `StrokeBrush`, `StrokeThickness`, dash array, caps, join, miter limit, `IsStrokeNonScaling`) | missing | | 2 |
| `CompositionContainerShape` (`Shapes`) | missing | | 2 |
| `CompositionGeometry` (`TrimStart`, `TrimEnd`, `TrimOffset`) | missing | | 2 |
| `CompositionRectangleGeometry`, `CompositionRoundedRectangleGeometry`, `CompositionEllipseGeometry`, `CompositionLineGeometry`, `CompositionPathGeometry` + `CompositionPath` | missing | | 2 |
| `CompositionViewBox` | missing | | 2 |

### 3.5 Clips

| Type / member | Status | Notes | Phase |
|---|---|---|---|
| `CompositionClip` (`AnchorPoint`, `CenterPoint`, `Offset`, `RotationAngle`, `Scale`, `TransformMatrix`) | missing | | 1 |
| `InsetClip` (`LeftInset`, `TopInset`, `RightInset`, `BottomInset`) | missing | The connected animation animates one. | 1 |
| `RectangleClip` (edges and corner radii) | missing | | 1 |
| `GeometricClip` (`Geometry`, `ViewBox`) | missing | Needs the geometries of phase 2. | 2 |

### 3.6 Lights and shadows

| Type / member | Status | Notes | Phase |
|---|---|---|---|
| `DropShadow` (`BlurRadius`, `Color`, `Offset`, `Opacity`, `Mask`, `SourcePolicy`) | missing | On sprite and layer visuals. The gravity configuration of a connected animation casts one. | 1 (rectangular), 5 (masks) |
| `CompositionLight` (`Targets`, `ExclusionsFromTargets`, `IsEnabled`) | missing | | 5 |
| `AmbientLight`, `DistantLight`, `PointLight`, `SpotLight` | missing | Diffuse lighting of flat visuals. | 5 |

### 3.7 Animations and easing

| Type / member | Status | Notes | Phase |
|---|---|---|---|
| `ScalarKeyFrameAnimation`, `Vector2/3/4KeyFrameAnimation`, `ColorKeyFrameAnimation`, `QuaternionKeyFrameAnimation`, `BooleanKeyFrameAnimation` | has | From the schema. | - |
| `KeyFrameAnimation` `DelayBehavior`, `DelayTime`, `Direction`, `Duration`, `IterationBehavior`, `IterationCount`, `InsertExpressionKeyFrame` | has | | - |
| `KeyFrameAnimation.StopBehavior` | partial | Stored and not applied, as upstream. Applied when an animation is stopped. | 1 |
| `KeyFrameAnimation.KeyFrameCount` | missing | | 1 |
| `ColorKeyFrameAnimation.InterpolationColorSpace` | missing | | 3 |
| `PathKeyFrameAnimation` | missing | Needs `CompositionPath` (phase 2). | 3 |
| `ExpressionAnimation` (`Expression`) | has | | - |
| `CompositionAnimation.Set*Parameter`, `SetReferenceParameter`, `ClearParameter`, `ClearAllParameters`, `Target` | has | | - |
| `CompositionAnimation.SetBooleanParameter`, `SetExpressionReferenceParameter` | has | `set_boolean_parameter`; `set_expression_reference_parameter` takes a composition object (`Rc<dyn AsCompositionObject>`) and is resolved as `set_reference_parameter` (WinUI's `IAnimationObject` is not ported, see the deviations). | 3 |
| `CompositionAnimation.InitialValueExpressions` (`InitialValueExpressionCollection`) | partial | `initial_value_expressions()`: the map (`insert`, `lookup`, `has_key`, `remove`, `clear`, `size`, `get_view`), its parsing (`parse_all`) and the evaluation against the target and the parameters (`evaluate_parsed`, `this.StartingValue` = the property's value). Not yet applied when the animation starts: that step belongs to the animation start on the server. | 3 |
| `CompositionAnimationGroup` | has | | - |
| `ImplicitAnimationCollection` | has | | - |
| `NaturalMotionAnimation`, `SpringScalarNaturalMotionAnimation`, `SpringVector2/3NaturalMotionAnimation` | missing | Spring animations (`DampingRatio`, `Period`, `FinalValue`, `InitialValue`, `InitialVelocity`). | 3 |
| `CompositionScopedBatch`, `CompositionCommitBatch` | missing | See 3.1. | 1 |
| `CubicBezierEasingFunction` (`ControlPoint1`, `ControlPoint2`) | partial | `SplineEasing` is the same curve; the WinUI factory and members are added. | 1 |
| `LinearEasingFunction` | has | `LinearEasing`. | 1 (factory) |
| `StepEasingFunction` (`StepCount`, `InitialStep`, `FinalStep`, `IsInitialStepSingleFrame`, `IsFinalStepSingleFrame`) | missing | | 1 |
| `BackEasingFunction` (`Mode`, `Amplitude`), `BounceEasingFunction` (`Mode`, `Bounces`, `Bounciness`), `CircleEasingFunction` (`Mode`), `ElasticEasingFunction` (`Mode`, `Oscillations`, `Springiness`), `ExponentialEasingFunction` (`Mode`, `Exponent`), `PowerEasingFunction` (`Mode`, `Power`), `SineEasingFunction` (`Mode`) | partial | FerroUI has the in, out and in-out variants of these curves with fixed parameters (`BackEaseIn`, ...). The parametric WinUI functions are added. | 1 |

### 3.8 Expression language

| Feature | Status | Notes | Phase |
|---|---|---|---|
| Operators (`+ - * / %`, comparisons, `&& || !`, `?:`), literals, member access, swizzles (`.XY`), `this.StartingValue`, `this.CurrentValue`, `this.FinalValue`, `this.Target` | has | `expressions/` | - |
| Functions `Abs`, `ACos`, `ASin`, `ATan`, `Ceil`, `Clamp`, `ColorLerp`, `ColorLerpRGB`, `ColorRGB`, `Concatenate`, `Cos`, `Distance`, `DistanceSquared`, `Floor`, `Inverse`, `Length`, `LengthSquared`, `Lerp`, `Ln`, `Log10`, `Matrix3x2`, `Matrix4x4`, `Max`, `Min`, `Mod`, `Normalize`, `Pow`, `Quaternion`, `Round`, `Scale`, `Sin`, `Slerp`, `SmoothStep`, `Sqrt`, `Square`, `Tan`, `ToDegrees`, `ToRadians`, `Transform`, `Vector2`, `Vector3`, `Vector4` | has | Every overload of the reference ("Expression Functions per Type" of `ExpressionAnimation`), plus those the WinUI expression engine accepts without listing them (the expression builder of the Windows Community Toolkit emits them): scalar `Length`, `LengthSquared`, `Distance`, `DistanceSquared`, `Scale`; per-component `Mod` of vectors; `Transform(Vector4, Matrix4x4)`. Also `Lerp` of `Matrix3x2` and `Matrix4x4` (not in the reference). `Matrix3x2(...)` and `Matrix4x4(...)` take each component from its own argument (the upstream implementation repeated the fifth). | 3 |
| Function names | has | Case-insensitive (`lerp`, `LERP`, `Asin` and `ASin` are the same function), as in WinUI, whose own expression builder writes them in lower case; also the builder's names `ColorHslLerp` and `ColorRgbLerp`. Keywords (`this.Target`, `Pi`, `True`) were already case-insensitive. | 3 |
| `ColorHsl`, `ColorLerpHSL` | has | `ColorHsl(h, s, l)`: hue in radians (0 to 2 pi, wrapping), saturation and lightness 0..1 (clamped), opaque. `ColorLerpHSL(c1, c2, p)`: `c1` at 0, `c2` at 1 as `ColorLerp`; saturation, lightness, alpha linear, hue the short way round, an achromatic colour takes the other's hue. | 3 |
| `Matrix3x2.CreateFromScale`, `CreateFromTranslation`, `CreateRotation`, `CreateScale`, `CreateSkew`, `CreateTranslation`; `Matrix4x4.CreateFromAxisAngle`, `CreateFromScale`, `CreateFromTranslation`, `CreateScale`, `CreateTranslation`; `Quaternion.CreateFromAxisAngle` | has | The static factories of the reference, with its formulas; a dotted name is parsed as one function name. `CreateTranslation` is `CreateFromTranslation`, as the reference says (upstream created a scale). Also the `System.Numerics` overloads: `Matrix3x2.CreateScale(s)`, `(x, y)`, `(scale, center)`, `CreateTranslation(x, y)`, `CreateRotation(angle, center)`, `CreateSkew(x, y)`; `Matrix4x4.CreateScale(s)`, `(x, y, z)`, `(scale, center)`, `CreateTranslation(x, y, z)`, `CreateRotationX/Y/Z(angle)` and `(angle, center)` (not in the reference). `Matrix4x4.CreateFromQuaternion` and the yaw-pitch-roll factories are not in the reference and not added. | 3 |
| Member, swizzle and parameter names | partial | Case-sensitive here (`Offset.X`, `.XY`, parameter keys). Whether WinUI matches them ignoring case is not documented. | - |
| Reference to an `InteractionTracker`, a visual's property set (`visual.Translation` after `SetIsTranslationEnabled`), a pointer property set | partial | Visuals and property sets can be referenced; the tracker comes with phase 4. | 4 |

### 3.9 Interaction

| Type / member | Status | Notes | Phase |
|---|---|---|---|
| `InteractionTracker` (position, scale, velocities, bounds, resting values, decay rates, `TryUpdate*`, states, owner callbacks) | missing | | 4 |
| `IInteractionTrackerOwner` | missing | | 4 |
| `InteractionTrackerInertiaRestingValue`, `InteractionTrackerInertiaMotion`, `InteractionTrackerInertiaNaturalMotion`, `Configure*InertiaModifiers` | missing | | 4 |
| `VisualInteractionSource` (`TryRedirectForManipulation`, source modes, rails, chaining, wheel configuration) | missing | | 4 |

### 3.10 XAML pieces

| Type / member | Status | Notes | Phase |
|---|---|---|---|
| `ConnectedAnimationService.GetForCurrentView` | partial | The upstream service has `get_for_top_level`; `get_for_current_view` is added (the service of the thread's active top level). | 1 |
| `ConnectedAnimationService.DefaultDuration`, `DefaultEasingFunction` | partial | Present; the easing default becomes WinUI's cubic Bezier (0.8, 0), (0.2, 1). | 1 |
| `ConnectedAnimationService.PrepareToAnimate`, `GetAnimation` | has | Upstream members; the engine behind them changes. | 1 |
| `ConnectedAnimation.TryStart(destination)`, `TryStart(destination, coordinatedElements)` | partial | Upstream: a control flown over the overlay layer by UI-thread animations, coordinated elements faded in. WinUI: snapshot brushes on sprite visuals animated by the compositor, coordinated elements entering from where they would be relative to the source. | 1 |
| `ConnectedAnimation.Cancel`, `IsScaleAnimationEnabled`, `SetAnimationComponent`, `Configuration`, `Completed` | partial | `Configuration`, `Completed` exist; the others are added. | 1 |
| `ConnectedAnimationConfiguration`, `GravityConnectedAnimationConfiguration` (`IsShadowEnabled`), `DirectConnectedAnimationConfiguration`, `BasicConnectedAnimationConfiguration` | partial | The classes exist; the WinUI behaviour of each (the gravity dip and shadow, the 200 ms decelerating direct animation, the basic animation without effect) is ported. | 1 |
| `ConnectedAnimationComponent` (`OffsetX`, `OffsetY`, `CrossFade`, `Scale`) | missing | | 1 |
| `ElementCompositionPreview.GetElementVisual`, `GetElementChildVisual`, `SetElementChildVisual` | has | `ElementComposition`; the WinUI-named facade is added. | 1 |
| `ElementCompositionPreview.SetImplicitShowAnimation`, `SetImplicitHideAnimation` | missing | | 1 |
| `ElementCompositionPreview.SetIsTranslationEnabled` | missing | | 1 |
| `ElementCompositionPreview.GetScrollViewerManipulationPropertySet` | missing | | 1 |
| `ElementCompositionPreview.GetPointerPositionPropertySet` | missing | | 1 |
| `UIElement.Translation`, `TranslationTransition` (`Vector3Transition`) | has | `Visual::translation` (a `Vector3D`) writes the `Translation` of the element's composition visual; `translation_transition` animates it (see the transitions row). | 1 (`Translation`), 3 (transitions) |
| `UIElement.Rotation`, `RotationAxis`, `Scale`, `CenterPoint`, `TransformMatrix` | partial | Plain properties of `Visual` (`rotation`, `rotation_axis`, `scale`, `center_point`, `transform_matrix`, with the `numerics` types), written to the element's composition visual whenever the element is rendered again; a facade never set leaves its composition property to the application. `Rotation` around the Z axis is `RotationAngle` (radians), around another axis `Orientation` (a quaternion), of which the 2D renderers show only the projection; a zero axis is the Z axis. `TransformMatrix`: its 2D affine part (`M11`, `M12`, `M21`, `M22`, `M41`, `M42`) follows the render transform in the composition `TransformMatrix`, which the composition visual applies before `Scale`, the rotation and `Translation` (WinUI applies it after the other facades). | 3 |
| `UIElement.OpacityTransition`, `RotationTransition`, `ScaleTransition`; `ScalarTransition` (`Duration`), `Vector3Transition` (`Duration`, `Components`), `Vector3TransitionComponents` | has | `ferroui_base::{ScalarTransition, Vector3Transition, Vector3TransitionComponents}` (values, copied when assigned; 300 ms and all components by default) and `Visual::set_opacity_transition`, `set_rotation_transition`, `set_scale_transition`, `set_translation_transition`. An implicit animation collection on the element's composition visual (`Opacity`, `RotationAngle` and `Orientation`, `Scale`, `Translation`): a key frame animation to `this.FinalValue` over the duration with the compositor's default easing. A `Vector3Transition` of some components starts with a key frame at 0 that holds the animated components at `this.StartingValue` and sets the others to `this.FinalValue`, so those jump at once; one of no component installs nothing. The collection is installed after the first synchronization of a new composition visual (the initial values do not animate) and before the properties are written afterwards, and replaces implicit animations the application set on the element's visual. | 3 |

## 4. Plan

The phases are ordered by value to an application: what a page of an application uses most
(connected animations between a list and a detail view, the element APIs) comes first, then the
visual building blocks, then the animation and easing breadth, then the rarer pieces. Each phase is
implemented with unit tests next to `compositor_tests.rs` and `render_thread_tests.rs` (or in the
test file of its folder), lands in the pull request of the port, and updates the inventory above.

### Phase 1: connected animations and the element APIs

What a connected animation needs, and the `ElementCompositionPreview` statics.

1. **Completion of animations**: `CompositionScopedBatch` and `CompositionCommitBatch`. An
   animation started while batches are active takes a completion guard of each; the server
   releases the guard when a key frame animation ends, is stopped or is replaced, and the
   batch raises `Completed` on the UI thread once it is ended and all its guards are released.
   `StopBehavior` is applied when an animation is stopped. Sub-channel targets (`Offset.X`,
   `Scale.XY`) for `StartAnimation`.
2. **Easing**: `CompositionEasingFunction` with the WinUI factories (cubic Bezier, linear, step,
   back, bounce, circle, elastic, exponential, power, sine, each with its mode and parameters),
   producing the `Easing` objects the key frames take.
3. **Sprite visual and surfaces**: `SpriteVisual` (`Brush`, `Shadow`), `CompositionSurfaceBrush`,
   `CompositionVisualSurface` (live), `LoadedImageSurface` (from a bitmap), `DropShadow` for
   rectangular content, the clips (`CompositionClip`, `InsetClip`, `RectangleClip`) on every
   visual.
4. **`ElementCompositionPreview`**: the WinUI facade over `ElementComposition`, the implicit
   show and hide animations (also when the element leaves the tree), `SetIsTranslationEnabled`,
   the scroll viewer manipulation and pointer position property sets; the `Translation`
   property of a visual.
5. **Connected animations**: the WinUI engine (states, rectangles, snapshot and live brushes on a
   sprite visual in a connected animation layer at the top of the window, the offset, scale,
   clip and cross-fade animations, the gravity, direct and basic configurations,
   `IsScaleAnimationEnabled`, `SetAnimationComponent`, coordinated elements, the timeout,
   `Cancel`, `Completed`) behind the upstream service's API.
6. **Catalog**: a "Composition (WinUI)" page with a list and a detail view joined by a connected
   animation, implicit show and hide animations, a pointer-driven light-like effect from the
   pointer position property set and a parallax header from the scroll viewer property set.
   Verified on macOS with the Skia and Vello backends.

### Phase 2: visuals, brushes and shapes

`ShapeVisual`, `CompositionSpriteShape`, `CompositionContainerShape`, the geometries with trimming,
`CompositionViewBox`, `GeometricClip`; `LayerVisual`, `RedirectVisual`; `CompositionNineGridBrush`,
`CompositionMaskBrush`; the WinUI members of the gradient brushes (mapping mode, `Vector2` points,
ellipse centre and radius, the brush transform); the remaining `Visual` members
(`RotationAngleInDegrees`, `RotationAxis` around Z, the relative adjustments, `IsHitTestVisible`,
`IsPixelSnappingEnabled`, `ParentForTransform`).

### Phase 3: animation and expression breadth

Spring natural motion animations; `AnimationController` and the global playback rate;
`CompositionObject.Properties` and `Comment`; `SetBooleanParameter`, `SetExpressionReferenceParameter`,
`InitialValueExpressions`; `ColorKeyFrameAnimation.InterpolationColorSpace`;
`PathKeyFrameAnimation`; the missing expression functions (`ColorHsl`, `ColorLerpHSL`, the
`Matrix3x2`, `Matrix4x4` and `Quaternion` factories); the `UIElement` facades and their
transitions (`Rotation`, `Scale`, `CenterPoint`, `TransformMatrix`, `ScalarTransition`,
`Vector3Transition`).

### Phase 4: interaction

`InteractionTracker` with its owner callbacks, states, bounds, inertia (decay rates, resting value,
motion and natural motion modifiers) and its `TryUpdate*` requests, exposed to expressions as a
referenceable object (`tracker.Position`, `tracker.Scale`); `VisualInteractionSource` fed by the
pointer and wheel input of the element whose visual it is created for, after
`TryRedirectForManipulation`.

### Phase 5: shadows and lights

`DropShadow` with a mask and the `SourcePolicy`, shadows of shape and layer visuals; the lights
(`AmbientLight`, `DistantLight`, `PointLight`, `SpotLight`) with targets and exclusions, as diffuse
lighting of flat visuals.

### Phase 6: effects (scoped)

`CompositionEffectBrush` and `CompositionEffectFactory` for the effects the drawing contexts of
FerroUI can express (Gaussian blur, opacity, cross-fade, a drop shadow), and
`CompositionBackdropBrush` on the backdrop support of the acrylic visual. The full Win2D effect
graph (the dozens of `Microsoft.Graphics.Canvas.Effects` classes and arbitrary graphs of them) is
out of scope: FerroUI has no Win2D, its drawing context has a small set of image effects, and a
general effect graph needs a shader pipeline per backend (Skia and Vello). An unsupported effect
makes `CreateEffectFactory` report `CompositionEffectFactoryLoadStatus::EffectTypeNotSupported`, as
WinUI does for an effect it cannot load.

## 5. Design notes

### 5.1 Connected animations

The upstream service and its public members stay: `ConnectedAnimationService::get_for_top_level`,
`prepare_to_animate`, `get_animation`, `default_duration`, `default_easing_function`;
`ConnectedAnimation::try_start`, `try_start_with_coordinated_elements`, `configuration`,
`completed` (with the `cancelled` flag), `is_consumed`, `dispose`. WinUI's members are added
(`get_for_current_view`, `cancel`, `is_scale_animation_enabled`, `set_animation_component`). The
engine is WinUI's:

- `prepare_to_animate` records the clipped and unclipped rectangles of the source in the
  coordinates of the window and freezes an image of the source (WinUI freezes a visual surface
  of the source on the next frame and keeps the source in unloading storage until then; here the
  image is rendered at once, see the deviations).
- `try_start` moves the animation to *started*; the first frame after the destination's layout
  computes its rectangles, builds the host sprite in the connected animation layer of the
  window (a container visual above the content, WinUI's `ConnectedAnimationRoot`), hides the
  source and destination elements, and starts the offset, scale, clip and cross-fade animations
  in a scoped batch whose completion ends the animation.
- The gravity configuration adds its dip (offset and scale key frames through a property set
  combined with the primary animation by expressions, as `GetEffectPropertySet` does) and a
  shadow; the direct configuration uses 200 ms and a decelerating curve; the basic one adds
  nothing.

### 5.2 Element visual properties

The compositing renderer writes `Offset`, `Size`, `Opacity`, `IsVisible`, the clip and
`TransformMatrix` of an element's visual whenever the element is rendered again, as WinUI's layout
writes the handoff visual. `Translation`, `Scale`, `CenterPoint`, `RotationAngle`, `Orientation`
and `AnchorPoint` are left to the application. An implicit animation on `Offset` therefore animates
every layout move of the element, as in WinUI.

## 6. Status

| Phase | State | Pull request |
|---|---|---|
| 1 | in progress | draft |
| 2 | planned | |
| 3 | planned | |
| 4 | planned | |
| 5 | planned | |
| 6 | planned (scoped) | |

## 7. Deviations

Every difference from WinUI, in the format of `DEVIATIONS.md` (Representation, Behaviour,
Missing). `DEVIATIONS.md` points here for this area.

| WinUI | Port | Kind | Why | Since |
|---|---|---|---|---|
| `Visual.TransformMatrix` is a `Matrix4x4`. | A 2D `Matrix`. | Representation | The renderers of FerroUI are 2D; the property is the one the compositor already had. | phase 1 |
| `ConnectedAnimation` freezes a `CompositionVisualSurface` of the source on the next frame and keeps the source element in unloading storage until then. | The source is rendered to a bitmap when `prepare_to_animate` is called, and the bitmap is shown by a `LoadedImageSurface`. | Behaviour | A detached element's composition visual loses its content in the same batch, and the server applies every queued batch before it runs jobs, so a server-side freeze could see the detached state. The image is the same; it is taken a frame earlier. | phase 1 |
| `SetExpressionReferenceParameter(String, IAnimationObject)`. | Takes `Rc<dyn AsCompositionObject>` and is the same as `set_reference_parameter`; an element facade passes its visual. | Representation | `IAnimationObject` and `PopulatePropertyInfo` are not ported. | phase 3 |
| `InitialValueExpressions` is an `InitialValueExpressionCollection` that is itself a `CompositionObject` (`IMap<String, String>`). | A plain map owned by the animation, with the members of the map. | Representation | It needs no server counterpart or identity of its own. | phase 3 |
| The hue path of `ColorLerpHSL` is not documented. | The hue takes the short way round, and an achromatic colour takes the hue of the other colour. | Behaviour | The port had to choose. | phase 3 |
| The case of member, swizzle and parameter names in expressions is not documented. | Function names are matched ignoring case (as the expression builder of the Windows Community Toolkit writes them in lower case); member, swizzle and parameter names are matched with case. | Behaviour | Function names follow the toolkit; the rest is unverified and left as it was. | phase 3 |
| `Matrix4x4.CreateRotationX/Y/Z` and the `System.Numerics` overloads of `CreateScale`, `CreateTranslation`, `CreateRotation`, `CreateSkew` and `Lerp` of matrices are not in the expression reference. | Accepted. | Addition | They are the factories of the numeric types the expressions use; an expression valid in WinUI is valid here. | phase 3 |
| A `CompositionPropertySet` is a live object of the compositor whose entries can be animated. | The same. The parameters of an animation are a property set without a server counterpart, copied when the animation starts (the upstream behaviour). | Representation | See `DEVIATIONS.md` for the difference from upstream. | phase 1 |
