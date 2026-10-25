# macOS 26 theme recovery and verification

The continuation sources originally prepared against `e39a5e6` have been reconciled
with PR #340 at `d625ed5`. The branch already contains the progress-pulse lifetime,
independent switch-knob transition collections, inactive selection/button resources,
derived contrast handling and their six Rust regressions. Those files are preserved,
including the corrected public `DoubleTransition` import.

The remaining native-reference work is now source-controlled: schema 2 separates
requested activation from observed application/key/main state, checks capture-time
stability, rejects clipped controls/glass, and supports `--require-state-coverage`.
The offline validator distinguishes inventory, state coverage, and default-optics
prerequisites. None of these scopes asserts native visual or motion equivalence.

```sh
python3 -m unittest discover -s scripts/macos-theme -p 'test_*.py' -v
python3 scripts/macos-theme/validate_reference.py /path/to/native-reference.json \
  --scope state-coverage --report /path/to/reference-validation.json
cargo test --locked -p ferroui-themes-macos --lib
cargo test --locked -p control-catalog --features macos-theme,runtime-markup \
  --lib tests::macos_theme_review -- --nocapture
```

The retained native artifact from the earlier session passes inventory validation
but fails state coverage: its requested-active images were observed inactive.
It also cannot establish default-optics prerequisites because accessibility
preferences reduced motion/transparency and no WindowServer images were supplied.
The fixture never changes the user's preferences or grants capture permissions.

Local recovery validation: 30 validator unit tests pass; the Swift frontend parses
the fixture. AppKit type checking, Rust/XAML builds and rendered comparisons remain
CI/native gates, not claims implied by those local checks.

The subsequent review's ZIP was not present in the resumed runtime. Its documented
resource, live-metric and shadow findings are reconstructed against the published
sources rather than treating an unavailable archive as a verified commit.
