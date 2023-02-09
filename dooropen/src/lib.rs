use rppal::gpio::{InputPin, Trigger};
use std::{
    collections::HashMap,
    sync::{
        mpsc::{channel, sync_channel, Receiver, Sender, SyncSender, RecvError},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

pub mod server;

/// Trait for the pins needed by PinInterrupter to reigister functions
pub trait InterruptablePin {
    /// registers a function with a simple interrupt function on the pin if it gets a low
    /// signal change
    fn on_low_signal<F>(&mut self, interupt: Box<F>)
    where
        F: FnMut() + std::marker::Send;

    /// registers a function with a simple interrupt function on the pin if it gets a high
    /// signal change
    fn on_high_signal<F>(&mut self, interupt: Box<F>)
    where
        F: FnMut() + std::marker::Send;

    /// Returns the specific pin-id
    fn get_id(&self) -> usize;
}

///Pin Interrupt struct for async communication with the pins
// TODO: dictionary needs to be interrupt-save, thread-save and accessable
// for the interrupt function  <22-01-23, Nikl> //
struct PinInterrupter {
    activated_pin_dict: Arc<Mutex<Vec<bool>>>,
}

impl PinInterrupter {
    pub fn new(channel_buffer_size: usize) -> Self {
        Self {
            activated_pin_dict: Arc::new(Mutex::new(Vec::new())),
        }
    }
    /// Registers the interups to the pins given in the InterruptablePin trait
    /** Uses the functions on_low_signal and on_high_signal from InterruptablePin to register a
     * specific behavior to the pin.
     *
     * On high the bool in the dictionary will be set(true) and on low the bool will be unset(false).
     */
    // TODO: Needs 2 closures or functions, that can access the dictionary. interrupt safety?? <22-01-23, Nikl> //
    fn register_pin(&mut self, mut pin: impl InterruptablePin) {
        let pin_id = pin.get_id();
        pin.on_low_signal(Box::new(|| {}));
        pin.on_high_signal(Box::new(|| {}));
    }

    ///Returns the optional Value of the pin dictionary, if pin is active
    pub fn get_pin_state(&self, pin: usize) -> bool {
        self.activated_pin_dict.lock().unwrap()[pin]
    }

    fn set_pin_state(&mut self, pin: usize, state: PinChange) {
        match state {
            PinChange::Fall => {},
            PinChange::Rise => {
                let current = self.get_pin_state(pin);
                self.activated_pin_dict.lock().unwrap()[pin]=!current;
            },
            PinChange::Stop => {},
        }
    }
}

enum PinChange {
    Rise,
    Fall,
    Stop,
}

struct PinSignalDispatcher {
    recv: Receiver<(usize,PinChange)>,
    sender: Sender<(usize,PinChange)>,
    fn_on_receive:Box<dyn Fn(usize, PinChange)>,
}

impl PinSignalDispatcher {
    pub fn new(on_receive: Box<dyn Fn(usize, PinChange)>) -> Self {
        let (send, rec): (Sender<(usize,PinChange)>, Receiver<(usize,PinChange)>) = channel();
        Self {
            recv: rec,
            sender: send,
            fn_on_receive: on_receive,
        }
    }

    pub fn receive(self) -> bool{
        match self.recv.recv() {
            Ok((pin_id,state_change)) => {
                match state_change {
                    PinChange::Rise | PinChange::Fall => {self.fn_on_receive.as_ref()(pin_id,state_change)},
                    PinChange::Stop => { return false },
                }
            },
            Err(x) => {return false},
        }
        true
    }

    pub fn get_sender(self) -> Sender<(usize,PinChange)> {
        self.sender.clone()
    }
}

//impl InterruptablePin for InputPin {
//    fn on_low_signal<F>(&mut self, interupt: Box<F>)
//    where
//        F: FnMut() + std::marker::Send,
//    {
//        self.set_async_interrupt(Trigger::RisingEdge, |_| interupt());
//    }
//    fn on_high_signal<F>(&mut self, interupt: Box<F>)
//    where
//        F: FnMut() + std::marker::Send,
//    {
//        self.set_async_interrupt(Trigger::RisingEdge, |_| interupt());
//    }
//    fn get_id(&self) -> usize {
//        self.pin() as usize
//    }
//}
