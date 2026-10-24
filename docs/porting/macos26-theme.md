# macOS 26 theme

## Goal

Create a separately selectable `ferroui-themes-macos` theme, copied from the working FerroUI WinUI 3 compiled-XAML theme at `98a56f09459c8ae02f07a2bcd235c0e87b4d44b3`, then adapt its resources, templates, motion and application chrome to macOS 26.

## Implementation work

- Preserve the existing WinUI theme and FerroUI controls, template parts, bindings and accessibility peers.
- Give the copied theme its own assembly, resource URIs, light/dark palettes, semantic colors, system typography, control metrics and macOS templates.
- Implement reduced motion, reduced transparency and increased contrast as explicit, live theme settings.
- Use FerroUI rendering and composition facilities for glass surfaces and motion; document backend differences against native AppKit.
- Exercise the complete inherited control inventory, compiled/runtime XAML parity, resource resolution, state transitions and representative layouts.
- Add a runnable gallery and repeatable comparison procedure for macOS 26.

## Fidelity evidence

The target is macOS 26, not the newest macOS design revision. Reference Apple's WWDC25 Liquid Glass and AppKit material. Numerical metrics and motion constants implemented here must not be described as Apple specifications without measured evidence. Native visual/animation comparison is a distinct acceptance gate; a theme loading successfully does not establish pixel-identical system rendering.

## References

- https://developer.apple.com/videos/play/wwdc2025/219/
- https://developer.apple.com/videos/play/wwdc2025/310/
- https://developer.apple.com/design/human-interface-guidelines/materials
- https://developer.apple.com/design/human-interface-guidelines/motion

## Status

Implementation started. This document will be updated with the implemented surface and actual validation results as commits land in the draft PR.
