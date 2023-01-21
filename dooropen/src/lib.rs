use std::collections::HashMap;

pub mod server;

///Pin Interrupt struct for async communication with the pins
// TODO: dictionary needs to be interrupt-save, thread-save and accessable 
// for the interrupt function  <22-01-23, Nikl> //
struct PinInterrupter {
    activated_pin_dict: HashMap<i32, bool>,
}

impl PinInterrupter {
    /// Registers the interups to the pins given in the InterruptablePin trait 
    /** Uses the functions on_low_signal and on_high_signal from InterruptablePin to register a
     * specific behavior to the pin.
     *
     * On high the bool in the dictionary will be set and on low the bool will be unset.
     */
    // TODO: Needs 2 closures or functions, that can access the dictionary. interrupt safety?? <22-01-23, Nikl> //
    fn register_interupt(pin: &impl InterruptablePin) {}
}

/// Trait for the pins needed by PinInterrupter to reigister functions
pub trait InterruptablePin {
    /// registers a function with a simple interrupt function on the pin if it gets a low
    /// signal change
    fn on_low_signal<F>(&self, interupt: F)
    where
        F: Fn();

    /// registers a function with a simple interrupt function on the pin if it gets a high
    /// signal change
    fn on_high_signal<F>(&self, interupt: F)
    where
        F: Fn();
}
