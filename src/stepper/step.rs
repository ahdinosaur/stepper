use core::{
    convert::{TryFrom, TryInto as _},
    task::Poll,
};

use embedded_hal::{digital::blocking::OutputPin, timer::nb as timer};
use embedded_time::duration::Nanoseconds;

use crate::traits::Step;

use super::SignalError;

/// The "future" returned by [`Stepper::step`]
///
/// Please note that this type provides a custom API and does not implement
/// [`core::future::Future`]. This might change, when using futures for embedded
/// development becomes more practical.
///
/// [`Stepper::step`]: crate::Stepper::step
#[must_use]
pub struct StepFuture<Driver, Timer> {
    driver: Driver,
    timer: Timer,
    state: State,
}

impl<Driver, Timer> StepFuture<Driver, Timer>
where
    Driver: Step,
    Timer: timer::CountDown,
    Timer::Time: TryFrom<Nanoseconds>,
{
    /// Create new instance of `StepFuture`
    ///
    /// This constructor is public to provide maximum flexibility for
    /// non-standard use cases. Most users can ignore this and just use
    /// [`Stepper::step`] instead.
    ///
    /// [`Stepper::step`]: crate::Stepper::step
    pub fn new(driver: Driver, timer: Timer) -> Self {
        Self {
            driver,
            timer,
            state: State::Initial,
        }
    }

    /// Poll the future
    ///
    /// The future must be polled for the operation to make progress. The
    /// operation won't start, until this method has been called once. Returns
    /// [`Poll::Pending`], if the operation is not finished yet, or
    /// [`Poll::Ready`], once it is.
    ///
    /// If this method returns [`Poll::Pending`], the user can opt to keep
    /// calling it at a high frequency (see [`Self::wait`]) until the operation
    /// completes, or set up an interrupt that fires once the timer finishes
    /// counting down, and call this method again once it does.
    pub fn poll(
        &mut self,
    ) -> Poll<
        Result<
            (),
            SignalError<
                Driver::Error,
                <Driver::Step as OutputPin>::Error,
                <Timer::Time as TryFrom<Nanoseconds>>::Error,
                Timer::Error,
            >,
        >,
    > {
        match self.state {
            State::Initial => {
                // Start step pulse
                self.driver
                    .step()
                    .map_err(|err| SignalError::PinUnavailable(err))?
                    .set_high()
                    .map_err(|err| SignalError::Pin(err))?;

                let ticks: Timer::Time = Driver::PULSE_LENGTH
                    .try_into()
                    .map_err(|err| SignalError::NanosecondsToTicks(err))?;
                self.timer
                    .start(ticks)
                    .map_err(|err| SignalError::Timer(err))?;

                self.state = State::PulseStarted;
                Poll::Pending
            }
            State::PulseStarted => {
                match self.timer.wait() {
                    Ok(()) => {
                        // End step pulse
                        self.driver
                            .step()
                            .map_err(|err| SignalError::PinUnavailable(err))?
                            .set_low()
                            .map_err(|err| SignalError::Pin(err))?;

                        self.state = State::Finished;
                        Poll::Ready(Ok(()))
                    }
                    Err(nb::Error::Other(err)) => {
                        self.state = State::Finished;
                        Poll::Ready(Err(SignalError::Timer(err)))
                    }
                    Err(nb::Error::WouldBlock) => Poll::Pending,
                }
            }
            State::Finished => Poll::Ready(Ok(())),
        }
    }

    /// Wait until the operation completes
    ///
    /// This method will call [`Self::poll`] in a busy loop until the operation
    /// has finished.
    pub fn wait(
        &mut self,
    ) -> Result<
        (),
        SignalError<
            Driver::Error,
            <Driver::Step as OutputPin>::Error,
            <Timer::Time as TryFrom<Nanoseconds>>::Error,
            Timer::Error,
        >,
    > {
        loop {
            if let Poll::Ready(result) = self.poll() {
                return result;
            }
        }
    }

    /// Drop the future and release the resources that were moved into it
    pub fn release(self) -> (Driver, Timer) {
        (self.driver, self.timer)
    }
}

enum State {
    Initial,
    PulseStarted,
    Finished,
}
