//! Contains traits that can be implemented by Step/Dir drivers

use embedded_hal::digital::OutputPin;
use embedded_time::duration::Nanoseconds;

use crate::StepMode;

/// Enable microstepping mode control for a driver
///
/// The `Resources` type parameter defines the hardware resources required for
/// controlling microstepping mode.
pub trait EnableStepModeControl<Resources> {
    /// The type of the driver once microstepping mode control has been enabled
    type WithStepModeControl: SetStepMode;

    /// Enable microstepping mode control
    fn enable_step_mode_control(
        self,
        res: Resources,
    ) -> Self::WithStepModeControl;
}

/// Implemented by drivers that support controlling the microstepping mode
pub trait SetStepMode {
    /// The time the mode signals need to be held before re-enabling the driver
    const SETUP_TIME: Nanoseconds;

    /// The time the mode signals need to be held after re-enabling the driver
    const HOLD_TIME: Nanoseconds;

    /// The error that can occur while using this trait
    type Error;

    /// The type that defines the microstepping mode
    ///
    /// This crate includes a number of enums that can be used for this purpose.
    type StepMode: StepMode;

    /// Apply the new step mode configuration
    ///
    /// Typically this puts the driver into reset and sets the mode pins
    /// according to the new step mode.
    fn apply_mode_config(
        &mut self,
        step_mode: Self::StepMode,
    ) -> Result<(), Self::Error>;

    /// Re-enable the driver after the mode has been set
    fn enable_driver(&mut self) -> Result<(), Self::Error>;
}

/// Enable direction control for a driver
///
/// The `Resources` type parameter defines the hardware resources required for
/// direction control.
pub trait EnableDirectionControl<Resources> {
    /// The type of the driver once direction control has been enabled
    type WithDirectionControl: SetDirection;

    /// Enable direction control
    fn enable_direction_control(
        self,
        res: Resources,
    ) -> Self::WithDirectionControl;
}

/// Implemented by drivers that support controlling the DIR signal
pub trait SetDirection {
    /// The time that the DIR signal must be held for a change to apply
    const SETUP_TIME: Nanoseconds;

    /// The type of the DIR pin
    type Dir: OutputPin<Error = Self::Error>;

    /// The error that can occur while using this trait
    type Error;

    /// Provides access to the DIR pin
    fn dir(&mut self) -> &mut Self::Dir;
}

/// Enable step control for a driver
///
/// The `Resources` type parameter defines the hardware resources required for
/// step control.
pub trait EnableStepControl<Resources> {
    /// The type of the driver once direction control has been enabled
    type WithStepControl: Step;

    /// Enable step control
    fn enable_step_control(self, res: Resources) -> Self::WithStepControl;
}

/// Implemented by drivers that support controlling the STEP signal
pub trait Step {
    /// The minimum length of a STEP pulse
    const PULSE_LENGTH: Nanoseconds;

    /// The type of the STEP pin
    type Step: OutputPin<Error = Self::Error>;

    /// The error that can occur while using this trait
    type Error;

    /// Provides access to the STEP pin
    fn step(&mut self) -> &mut Self::Step;
}