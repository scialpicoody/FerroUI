//! Material 3 Expressive motion, connected to FerroUI's transition and compositor APIs.
//!
//! The spring constants follow AndroidX Material3 1.5.0-beta02, commit
//! `e4d933c98df7baa7ea2b83eea6e086915961e0ff`, `ExpressiveMotionTokens.kt` and
//! `StandardMotionTokens.kt` (Copyright The Android Open Source Project, Apache-2.0).
//! The analytical evaluator and the FerroUI adapters are native Rust implementations.
//!
//! Spatial motion may overshoot. Effects motion is critically damped: use it for colors,
//! opacity and the button's pressed shape. In particular, the upstream button intentionally
//! uses `DefaultEffects`, while the switch thumb uses `FastSpatial`.

use ferroui_base::animation::easings::{Easing, IEasing, SharedEasing};
use ferroui_base::animation::TimeSpan;
use ferroui_base::platform::IPlatformSettings;
use ferroui_base::{FerroLocator, LocatorExtensions};
use std::sync::Arc;

/// The two motion schemes published by Material Design.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum MaterialMotionScheme {
    /// Subtle motion for utilitarian interfaces.
    Standard,
    /// The Material 3 Expressive spring scheme.
    #[default]
    Expressive,
}

ferroui_base::ferro_markup_enum!(
    MaterialMotionScheme { Standard, Expressive },
    { namespace: "FerroUI.Themes.Material" }
);

/// A semantic motion role. Spatial and effects roles must not be interchanged.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MaterialMotionRole {
    FastSpatial,
    DefaultSpatial,
    SlowSpatial,
    FastEffects,
    DefaultEffects,
    SlowEffects,
}

impl MaterialMotionRole {
    /// All the semantic roles, in their resource-installation order.
    pub const ALL: [Self; 6] = [
        Self::FastSpatial,
        Self::DefaultSpatial,
        Self::SlowSpatial,
        Self::FastEffects,
        Self::DefaultEffects,
        Self::SlowEffects,
    ];

    /// The role's suffix in `MaterialMotion{Role}Duration` and `MaterialMotion{Role}Easing`.
    pub const fn resource_name(self) -> &'static str {
        match self {
            Self::FastSpatial => "FastSpatial",
            Self::DefaultSpatial => "DefaultSpatial",
            Self::SlowSpatial => "SlowSpatial",
            Self::FastEffects => "FastEffects",
            Self::DefaultEffects => "DefaultEffects",
            Self::SlowEffects => "SlowEffects",
        }
    }
}

impl MaterialMotionScheme {
    /// The exact unit-mass spring parameters of the upstream motion role.
    pub const fn spring(self, role: MaterialMotionRole) -> MaterialSpringSpec {
        use MaterialMotionRole::*;
        let (damping, stiffness) = match (self, role) {
            (Self::Expressive, FastSpatial) => (0.6, 800.0),
            (Self::Expressive, DefaultSpatial) => (0.8, 380.0),
            (Self::Expressive, SlowSpatial) => (0.8, 200.0),
            (Self::Standard, FastSpatial) => (0.9, 1400.0),
            (Self::Standard, DefaultSpatial) => (0.9, 700.0),
            (Self::Standard, SlowSpatial) => (0.9, 300.0),
            (_, FastEffects) => (1.0, 3800.0),
            (_, DefaultEffects) => (1.0, 1600.0),
            (_, SlowEffects) => (1.0, 800.0),
        };
        MaterialSpringSpec::new(damping, stiffness)
    }

    /// The finite duration used to host this spring in FerroUI's duration-based transitions.
    ///
    /// This is a settling-time adapter, not a Material duration token: the easing evaluates
    /// the source spring in real seconds and the duration only bounds its imperceptible tail.
    /// Install this value with the matching [`Self::easing`] resource.
    pub fn duration(self, role: MaterialMotionRole, reduced_motion: bool) -> TimeSpan {
        if reduced_motion {
            TimeSpan::ZERO
        } else {
            TimeSpan::from_seconds(self.spring(role).settling_duration())
        }
    }

    /// A real FerroUI easing for `DoubleTransition`, `BrushTransition`, shape transitions,
    /// keyframe animations and compositor animations.
    ///
    /// The adapter additionally reads `IPlatformSettings::animations_enabled` on the UI
    /// thread. Its render-thread snapshot is immutable and contains no thread-local access.
    pub fn easing(self, role: MaterialMotionRole, reduced_motion: bool) -> Easing {
        Easing::new(MaterialSpringEasing::new(
            self.spring(role),
            self.duration(role, reduced_motion),
            reduced_motion,
        ))
    }
}

/// The position and velocity of a spring. Velocity is measured in value units per second.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MaterialSpringSample {
    pub value: f64,
    pub velocity: f64,
}

/// Parameters of a unit-mass damped spring: `x'' + 2 ζ √k x' + k (x-target) = 0`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MaterialSpringSpec {
    damping_ratio: f64,
    stiffness: f64,
}

impl MaterialSpringSpec {
    /// Creates a spring. Stiffness must be positive and finite; damping must be finite and
    /// nonnegative. A zero-damping spring oscillates forever and has no settling duration.
    pub const fn new(damping_ratio: f64, stiffness: f64) -> Self {
        assert!(
            damping_ratio.is_finite() && damping_ratio >= 0.0,
            "invalid spring damping ratio"
        );
        assert!(
            stiffness.is_finite() && stiffness > 0.0,
            "invalid spring stiffness"
        );
        Self {
            damping_ratio,
            stiffness,
        }
    }

    pub const fn damping_ratio(self) -> f64 {
        self.damping_ratio
    }

    pub const fn stiffness(self) -> f64 {
        self.stiffness
    }

    /// Evaluates both position and velocity analytically, independent of display refresh rate.
    ///
    /// All three damping regimes are supported. The overdamped case uses a stable slow root
    /// and `expm1` to avoid subtracting nearly equal exponentials near critical damping.
    /// The underdamped case uses `sinc`, so its limit at critical damping is also well behaved.
    /// A negative/NaN elapsed time leaves the initial state unchanged; positive infinity is
    /// treated as the settled state. Initial and target values must be finite.
    pub fn sample(
        self,
        elapsed: f64,
        from: f64,
        target: f64,
        initial_velocity: f64,
    ) -> MaterialSpringSample {
        assert!(
            from.is_finite() && target.is_finite() && initial_velocity.is_finite(),
            "non-finite spring state"
        );
        if elapsed.is_nan() || elapsed <= 0.0 {
            return MaterialSpringSample {
                value: from,
                velocity: initial_velocity,
            };
        }
        if !elapsed.is_finite() {
            return MaterialSpringSample {
                value: target,
                velocity: 0.0,
            };
        }

        let displacement = from - target;
        let omega = self.stiffness.sqrt();
        let zeta = self.damping_ratio;
        let alpha = zeta * omega;

        let (position, velocity) = if zeta < 1.0 {
            let decay = (-alpha * elapsed).exp();
            if decay == 0.0 {
                return MaterialSpringSample {
                    value: target,
                    velocity: 0.0,
                };
            }
            let frequency = omega * ((1.0 - zeta) * (1.0 + zeta)).sqrt();
            let raw_angle = frequency * elapsed;
            // A finite elapsed time can overflow the phase of an undamped spring. Reduce
            // time by its period in that case, so a suspended clock cannot create NaNs.
            let angle = if raw_angle.is_finite() {
                raw_angle
            } else {
                (elapsed % (std::f64::consts::TAU / frequency)) * frequency
            };
            let cosine = angle.cos();
            let sine_over_frequency = if raw_angle.abs() < 1.0e-4 {
                elapsed * sinc(angle)
            } else {
                angle.sin() / frequency
            };
            (
                decay
                    * (displacement * cosine
                        + (initial_velocity + alpha * displacement) * sine_over_frequency),
                decay
                    * (initial_velocity * cosine
                        - (zeta * initial_velocity + omega * displacement)
                            * (omega * sine_over_frequency)),
            )
        } else if zeta == 1.0 {
            let decay = (-omega * elapsed).exp();
            if decay == 0.0 {
                return MaterialSpringSample {
                    value: target,
                    velocity: 0.0,
                };
            }
            let coefficient = initial_velocity + omega * displacement;
            (
                (displacement + coefficient * elapsed) * decay,
                (initial_velocity - omega * coefficient * elapsed) * decay,
            )
        } else {
            // (zeta + sqrt(zeta^2-1))/zeta, without overflowing either zeta^2 or
            // zeta + sqrt(zeta^2-1). The reciprocal slow root avoids cancellation.
            let factor = 1.0 + (((zeta - 1.0) / zeta) * (1.0 + zeta.recip())).sqrt();
            let slow = -omega / zeta / factor;
            let fast = -omega * zeta * factor;
            // Divide by the fast root first; even a fast root beyond f64's range
            // then has the correct finite limiting coefficient.
            let coefficient = (displacement - initial_velocity / fast) / (1.0 - slow / fast);
            let slow_decay = (slow * elapsed).exp();
            let fast_decay = (fast * elapsed).exp();
            let difference = slow_decay * -((fast - slow) * elapsed).exp_m1();
            (
                displacement * fast_decay + coefficient * difference,
                initial_velocity * fast_decay + slow * coefficient * difference,
            )
        };

        MaterialSpringSample {
            value: target + position,
            velocity,
        }
    }

    /// A conservative finite tail for the unit step, with position tolerance `0.001` and
    /// velocity tolerance `0.001 * sqrt(stiffness)`. Returns infinity for an undamped spring.
    pub fn settling_duration(self) -> f64 {
        const TOLERANCE: f64 = 0.001;
        let omega = self.stiffness.sqrt();
        if self.damping_ratio == 0.0 {
            return f64::INFINITY;
        }
        if self.damping_ratio < 1.0 {
            let amplitude = 1.0 / ((1.0 - self.damping_ratio) * (1.0 + self.damping_ratio)).sqrt();
            return (amplitude / TOLERANCE).ln() / (self.damping_ratio * omega);
        }

        // For the nonoscillatory unit step, displacement decreases monotonically. Once its
        // displacement is this small, velocity is also on its decreasing branch. Bracket then
        // bisect rather than advancing an Euler simulation with a refresh-rate-dependent step.
        let settled = |t| {
            let sample = self.sample(t, 0.0, 1.0, 0.0);
            (1.0 - sample.value).abs() <= TOLERANCE && sample.velocity.abs() <= TOLERANCE * omega
        };
        let mut lower = 0.0;
        let mut upper = 1.0 / omega;
        while !settled(upper) {
            lower = upper;
            upper *= 2.0;
            if !upper.is_finite() {
                return f64::INFINITY;
            }
        }
        for _ in 0..48 {
            let middle = (lower + upper) * 0.5;
            if settled(middle) {
                upper = middle;
            } else {
                lower = middle;
            }
        }
        upper
    }
}

fn sinc(value: f64) -> f64 {
    if value.abs() < 1.0e-4 {
        let squared = value * value;
        1.0 - squared / 6.0 + squared * squared / 120.0
    } else {
        value.sin() / value
    }
}

/// A velocity-continuous spring for interactive controls whose target may change mid-flight.
///
/// Call [`Self::advance`] with frame time, then [`Self::retarget`] on an input change. Retargeting
/// preserves both the current position and its physical velocity, including when the target
/// reverses direction. No allocations or numerical integration are involved.
#[derive(Clone, Copy, Debug)]
pub struct MaterialSpringState {
    spec: MaterialSpringSpec,
    origin: MaterialSpringSample,
    current: MaterialSpringSample,
    target: f64,
    elapsed: f64,
    reduced_motion: bool,
}

impl MaterialSpringState {
    pub fn new(spec: MaterialSpringSpec, value: f64) -> Self {
        assert!(value.is_finite(), "non-finite spring value");
        let initial = MaterialSpringSample {
            value,
            velocity: 0.0,
        };
        Self {
            spec,
            origin: initial,
            current: initial,
            target: value,
            elapsed: 0.0,
            reduced_motion: false,
        }
    }

    pub fn current(&self) -> MaterialSpringSample {
        self.current
    }

    pub fn target(&self) -> f64 {
        self.target
    }

    pub fn retarget(&mut self, target: f64) {
        assert!(target.is_finite(), "non-finite spring target");
        if self.target == target {
            return;
        }
        self.origin = self.current;
        self.target = target;
        self.elapsed = 0.0;
        if self.reduced_motion {
            self.finish();
        }
    }

    /// Changes the motion scheme without discontinuously resetting the live state.
    pub fn set_spec(&mut self, spec: MaterialSpringSpec) {
        self.origin = self.current;
        self.spec = spec;
        self.elapsed = 0.0;
    }

    pub fn set_reduced_motion(&mut self, reduced_motion: bool) {
        self.reduced_motion = reduced_motion;
        if reduced_motion {
            self.finish();
        }
    }

    pub fn advance(&mut self, seconds: f64) -> MaterialSpringSample {
        if self.reduced_motion {
            self.finish();
        } else if !seconds.is_nan() && seconds > 0.0 {
            self.elapsed += seconds;
            self.current = self.spec.sample(
                self.elapsed,
                self.origin.value,
                self.target,
                self.origin.velocity,
            );
        }
        self.current
    }

    pub fn is_at_rest(&self, position_threshold: f64, velocity_threshold: f64) -> bool {
        (self.current.value - self.target).abs() <= position_threshold.max(0.0)
            && self.current.velocity.abs() <= velocity_threshold.max(0.0)
    }

    pub fn finish(&mut self) {
        self.current = MaterialSpringSample {
            value: self.target,
            velocity: 0.0,
        };
        self.origin = self.current;
        self.elapsed = 0.0;
    }
}

/// Bridges a physical spring to FerroUI's normalized-progress `IEasing` interface.
///
/// Use the matching duration; the same spring with an unrelated duration changes its physical
/// speed. This adapter starts with zero velocity because `IEasing` has no velocity channel.
/// Controls needing interruption continuity use [`MaterialSpringState`] instead.
#[derive(Clone, Copy, Debug)]
pub struct MaterialSpringEasing {
    spec: MaterialSpringSpec,
    seconds: f64,
    reduced_motion: bool,
    check_platform: bool,
}

impl MaterialSpringEasing {
    pub fn new(spec: MaterialSpringSpec, duration: TimeSpan, reduced_motion: bool) -> Self {
        assert!(duration >= TimeSpan::ZERO, "negative spring duration");
        Self {
            spec,
            seconds: duration.total_seconds(),
            reduced_motion,
            check_platform: true,
        }
    }

    fn evaluate(&self, progress: f64, reduced_motion: bool) -> f64 {
        if reduced_motion || self.seconds == 0.0 {
            return 1.0;
        }
        if progress.is_nan() || progress <= 0.0 {
            return 0.0;
        }
        if progress >= 1.0 {
            return 1.0;
        }
        self.spec
            .sample(progress * self.seconds, 0.0, 1.0, 0.0)
            .value
    }
}

impl IEasing for MaterialSpringEasing {
    fn ease(&self, progress: f64) -> f64 {
        let reduced = self.reduced_motion || (self.check_platform && !animations_enabled(false));
        self.evaluate(progress, reduced)
    }

    fn to_shared(&self) -> Arc<SharedEasing> {
        Arc::new(Self {
            reduced_motion: self.reduced_motion
                || (self.check_platform && !animations_enabled(false)),
            check_platform: false,
            ..*self
        })
    }
}

pub(crate) fn animations_enabled(reduced_motion: bool) -> bool {
    !reduced_motion
        && FerroLocator::current()
            .get_service::<dyn IPlatformSettings>()
            .is_none_or(|settings| settings.animations_enabled())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(expected: f64, actual: f64, epsilon: f64) {
        assert!(
            (expected - actual).abs() <= epsilon,
            "expected {expected}, got {actual}"
        );
    }

    #[test]
    fn source_motion_tokens_are_preserved() {
        let expressive = MaterialMotionScheme::Expressive;
        assert_eq!(
            MaterialSpringSpec::new(0.6, 800.0),
            expressive.spring(MaterialMotionRole::FastSpatial)
        );
        assert_eq!(
            MaterialSpringSpec::new(0.8, 380.0),
            expressive.spring(MaterialMotionRole::DefaultSpatial)
        );
        assert_eq!(
            MaterialSpringSpec::new(0.8, 200.0),
            expressive.spring(MaterialMotionRole::SlowSpatial)
        );
        for role in [
            MaterialMotionRole::FastEffects,
            MaterialMotionRole::DefaultEffects,
            MaterialMotionRole::SlowEffects,
        ] {
            assert_eq!(
                expressive.spring(role),
                MaterialMotionScheme::Standard.spring(role)
            );
            assert_eq!(1.0, expressive.spring(role).damping_ratio());
        }
    }

    #[test]
    fn critical_and_undamped_solutions_match_closed_form_reference() {
        let critical = MaterialSpringSpec::new(1.0, 1600.0).sample(0.05, 0.0, 1.0, 0.0);
        close(0.5939941502901619, critical.value, 1e-12);
        close(10.826822658929016, critical.velocity, 1e-12);
        let undamped = MaterialSpringSpec::new(0.0, 100.0).sample(0.1, 0.0, 1.0, 0.0);
        close(0.45969769413186023, undamped.value, 1e-12);
        close(8.414709848078965, undamped.velocity, 1e-12);
    }

    #[test]
    fn position_and_velocity_are_continuous_across_critical_damping() {
        let center = MaterialSpringSpec::new(1.0, 380.0);
        for time in [0.00001, 0.01, 0.1, 1.0] {
            let expected = center.sample(time, 2.0, -4.0, 17.0);
            for damping in [1.0 - 1e-9, 1.0 + 1e-9] {
                let sample = MaterialSpringSpec::new(damping, 380.0).sample(time, 2.0, -4.0, 17.0);
                close(expected.value, sample.value, 1e-7);
                close(expected.velocity, sample.velocity, 1e-6);
            }
        }
    }

    #[test]
    fn velocity_is_the_derivative_in_every_damping_regime() {
        for damping in [0.0, 0.6, 0.999999, 1.0, 1.000001, 2.0, 20.0] {
            let spring = MaterialSpringSpec::new(damping, 380.0);
            let step = 1e-6;
            let left = spring.sample(0.15 - step, 12.0, -3.0, 5.0);
            let right = spring.sample(0.15 + step, 12.0, -3.0, 5.0);
            let center = spring.sample(0.15, 12.0, -3.0, 5.0);
            close(
                center.velocity,
                (right.value - left.value) / (2.0 * step),
                1e-5,
            );
        }
    }

    #[test]
    fn retargeting_preserves_incoming_velocity_and_is_refresh_rate_independent() {
        let spec = MaterialMotionScheme::Expressive.spring(MaterialMotionRole::FastSpatial);
        let mut spring = MaterialSpringState::new(spec, 0.0);
        spring.retarget(100.0);
        let before = spring.advance(0.08);
        spring.retarget(-20.0);
        assert_eq!(before, spring.current());
        assert_eq!(before, spring.advance(0.0));
        let mut single_step = spring;
        for _ in 0..30 {
            spring.advance(1.0 / 120.0);
        }
        let expected = single_step.advance(0.25);
        close(expected.value, spring.current().value, 1e-10);
        close(expected.velocity, spring.current().velocity, 1e-10);
    }

    #[test]
    fn spatial_motion_overshoots_but_effects_do_not() {
        let scheme = MaterialMotionScheme::Expressive;
        let spatial = scheme.spring(MaterialMotionRole::FastSpatial);
        assert!((1..100).any(|i| spatial.sample(i as f64 / 100.0, 0.0, 1.0, 0.0).value > 1.01));
        for role in [
            MaterialMotionRole::FastEffects,
            MaterialMotionRole::DefaultEffects,
            MaterialMotionRole::SlowEffects,
        ] {
            let effect = scheme.spring(role);
            let mut previous = 0.0;
            for step in 0..100 {
                let value = effect.sample(step as f64 / 100.0, 0.0, 1.0, 0.0).value;
                assert!((previous..=1.0).contains(&value));
                previous = value;
            }
        }
    }

    #[test]
    fn transition_duration_preserves_physical_time_and_settles() {
        for scheme in [
            MaterialMotionScheme::Standard,
            MaterialMotionScheme::Expressive,
        ] {
            for role in MaterialMotionRole::ALL {
                let spec = scheme.spring(role);
                let duration = scheme.duration(role, false);
                let easing = MaterialSpringEasing::new(spec, duration, false);
                let expected = spec.sample(0.05, 0.0, 1.0, 0.0).value;
                close(
                    expected,
                    easing.ease(0.05 / duration.total_seconds()),
                    1e-12,
                );
                close(easing.ease(0.5), easing.to_shared().ease(0.5), 1e-12);
                close(
                    1.0,
                    spec.sample(duration.total_seconds(), 0.0, 1.0, 0.0).value,
                    0.00101,
                );
                assert_eq!(0.0, easing.ease(0.0));
                assert_eq!(1.0, easing.ease(1.0));
            }
        }
    }

    #[test]
    fn reduced_motion_snaps_existing_and_new_targets() {
        let scheme = MaterialMotionScheme::Expressive;
        let role = MaterialMotionRole::DefaultSpatial;
        assert_eq!(TimeSpan::ZERO, scheme.duration(role, true));
        assert_eq!(1.0, scheme.easing(role, true).ease(0.0));
        let mut state = MaterialSpringState::new(scheme.spring(role), 0.0);
        state.retarget(40.0);
        state.advance(0.01);
        state.set_reduced_motion(true);
        assert_eq!(
            MaterialSpringSample {
                value: 40.0,
                velocity: 0.0
            },
            state.current()
        );
        state.retarget(90.0);
        assert_eq!(
            MaterialSpringSample {
                value: 90.0,
                velocity: 0.0
            },
            state.current()
        );
    }

    #[test]
    fn invalid_elapsed_time_does_not_poison_a_live_spring() {
        let mut state = MaterialSpringState::new(MaterialSpringSpec::new(0.8, 380.0), 10.0);
        state.retarget(20.0);
        let initial = state.current();
        assert_eq!(initial, state.advance(f64::NAN));
        assert_eq!(initial, state.advance(-1.0));
        assert_eq!(
            MaterialSpringSample {
                value: 20.0,
                velocity: 0.0
            },
            state.advance(f64::INFINITY)
        );
    }

    #[test]
    fn extreme_finite_clock_values_do_not_overflow_spring_phases() {
        for damping in [0.6, 1.0, 2.0] {
            let sample = MaterialSpringSpec::new(damping, 380.0).sample(f64::MAX, 12.0, -3.0, 5.0);
            assert_eq!(
                MaterialSpringSample {
                    value: -3.0,
                    velocity: 0.0
                },
                sample
            );
        }
        let undamped = MaterialSpringSpec::new(0.0, 1.0e308).sample(f64::MAX, 12.0, -3.0, 5.0);
        assert!(undamped.value.is_finite());
        assert!(undamped.velocity.is_finite());
        let overdamped = MaterialSpringSpec::new(1.0e308, 380.0).sample(0.25, 12.0, -3.0, 5.0);
        assert!(overdamped.value.is_finite());
        assert!(overdamped.velocity.is_finite());
        close(12.0, overdamped.value, 1e-12);
    }

    #[test]
    #[should_panic(expected = "invalid spring stiffness")]
    fn rejects_invalid_stiffness() {
        MaterialSpringSpec::new(0.8, f64::NAN);
    }
}
