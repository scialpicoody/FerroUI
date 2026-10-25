# macOS 26 resource and popup follow-up

Base: `a6f91dbd758c2a121a46c6d9735511a8b87da993`, PR #340.

The previous motion recovery and native reference capture/validator are already
published at this base. This change leaves them, the corrected DoubleTransition
import and generated-source validation checks intact.

## Contracts changed

The resource-key check compares every appearance dictionary with each other
appearance individually. A union comparison allowed a resource in Light and
Default to remain absent from HighContrast. MacOSMaterials now explicitly defines
all eight appearance-specific keys in the named high-contrast dictionary, using
semantic system colors. A regression tests the formerly invisible omission.

ToolTip receives an independently owned, fully initialized opacity transition
collection at its resource host. Reduce Motion changes the duration between the
existing 100 ms calibration and zero. This avoids asking a transition buried in
a setter value to resolve a live resource without a host. Switch behavior and its
published 180 ms calibration remain unchanged.

ToolTip uses the same semantic shadow before and after its open-state selector
activates. ToolTip and the switch knob no longer embed shadow literals. Explicit
macOS and inherited decorative shadow keys resolve to no shadow under increased
contrast and the named/derived contrast variants; this is a FerroUI theme policy,
not a claim that every AppKit rendering matches that policy pixel-for-pixel.

Popup and window backing insets are calculated from BoxShadows.transform_bounds,
with outward rounding, rather than copied independently from a smaller shadow.
The menu recipe needs 24/16/24/32 points; window chrome needs 36/24/36/48; the tooltip
recipe needs 6/4/6/8. Insets become zero when the effective theme shadow is empty.
Application overrides outside the theme may need corresponding inset overrides;
this provider does not observe an arbitrary descendant's local shadow override.

## Validation and native fidelity

Seven new runtime tests cover transition ownership/lifetime, live preference
changes, shadow bounds, contrast inheritance and instantiated controls. The
resource comparison helper has an additional regression. These tests must run
with the full theme library suite; static XML/source checks are not substitutes.

Regenerate compiled_xaml.rs, compiled_xaml.xamlmeta and the theme audit report
using the existing generators. Keep their byte-for-byte checks enabled. Exact
native typography, WindowServer optics, activation and animated trajectories
still require matched macOS 26 evidence. No native-fidelity percentage is claimed.
