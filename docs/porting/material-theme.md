# Android Material 3 Expressive theme

## Reference and scope

This work adds `ferroui-themes-material`, an independent theme derived from the
existing WinUI 3 theme implementation. It preserves FerroUI controls, template
contracts, resource lookup, compiled XAML, accessibility and layout behavior,
then replaces the visual system with Android Material 3 Expressive.

Reference checked on 2026-10-24:

- [Material 3](https://developer.android.com/develop/ui/compose/designsystems/material3)
- [AndroidX Material3 releases](https://developer.android.com/jetpack/androidx/releases/compose-material3):
  stable 1.4.0; latest Expressive preview 1.5.0-beta02 (2026-10-21).
- [Material 3 Expressive](https://m3.material.io/blog/building-with-m3-expressive)

The latest Expressive reference is a preview API surface. The implementation
will record source revisions and retain upstream license notices.

## Implementation plan

1. Fork the WinUI theme infrastructure into a separate compiled Material theme.
2. Implement Material color roles, light/dark schemes, typography, shape,
   elevation, state layers, touch targets and accessible focus states.
3. Replace control templates and styling for Material components; retain
   FerroUI's functional template contracts for keyboard, pointer, touch and RTL.
4. Implement expressive motion and reduced-motion behavior.
5. Add a Material gallery and automated resource/template/interaction coverage.
6. Validate compilation, control behavior and rendered output.

## Fidelity reporting

The goal is fidelity to the pinned Android Material reference. Completion and
pixel accuracy must be measured against native Android captures; the presence
of a resource or control template alone is not proof of visual parity.
Implementation status and known platform differences will be recorded here
as the work proceeds.

