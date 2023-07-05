use rppal::gpio::{InputPin, Trigger};
use std::{
    collections::HashMap,
    ops::Not,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    time::Duration,
};

pub mod server;

pub enum PinChange {
    Rise,
    Fall,
}

/// Trait for the pins needed by PinInterrupter to register functions
pub trait InterruptablePin {
    /// registers a function with a sender who should send the enum PinChange which is then
    /// received by the corresponding PinInterrupter object
    fn on_low_signal(&mut self, sender: Sender<(usize, PinChange)>);

    /// registers a function with a sender who should send the enum PinChange which is then
    /// received by the corresponding PinInterrupter object
    fn on_high_signal(&mut self, sender: Sender<(usize, PinChange)>);

    /// returns the specific pin-id
    fn get_id(&self) -> usize;
}

///Pin interrupt struct for async communication with the pins
struct PinInterrupter {
    // receive_thread: JoinHandle<()>,
    // stop: Arc<AtomicBool>,
    activated_pin_dict: Arc<Mutex<HashMap<usize, bool>>>,
    sender: Sender<(usize, PinChange)>,
    receiver: Receiver<(usize, PinChange)>,
}

impl PinInterrupter {
    pub fn new() -> Self {
        let (s, r): (Sender<(usize, PinChange)>, Receiver<(usize, PinChange)>) = channel();
        let pin_dir = Arc::new(Mutex::new(HashMap::<usize, bool>::new()));

        Self {
            activated_pin_dict: pin_dir,
            sender: s,
            receiver: r,
        }
    }

    /// Registers the interups to the pins given in the implemented InterruptablePin trait
    /** Uses the functions on_low_signal and on_high_signal from InterruptablePin to give the pin a
     * reference to the sender which can then be used to send a PinChange::Rise or PinChange::Fall
     *
     */
    pub fn register_pin(&mut self, mut pin: impl InterruptablePin) {
        pin.on_low_signal(self.sender.clone());
        pin.on_high_signal(self.sender.clone());
    }

    ///Returns the optional Value of the pin dictionary, if pin is active
    pub fn get_pin_state(&self, pin: usize) -> Option<bool> {
        self.activated_pin_dict
            .lock()
            .unwrap()
            .clone()
            .get(&pin)
            .copied()
    }

    pub fn start_thread(&mut self, stop: Arc<AtomicBool>) {
        while stop.load(Ordering::Relaxed) {
            match self.receiver.recv_timeout(Duration::new(1,0)) {
            Ok((pos,change)) => {
                let mut dir = self.activated_pin_dict.lock().expect("Error on lock");
                match  dir.get_mut(&( pos.clone() )) {
                    Some(x) => *x = x.not(),
                    None => {
                        match change {
                            PinChange::Rise => dir.insert(pos,true),
                            PinChange::Fall => dir.insert(pos,false),
                        };
                        ()
                    }
                };
                ()
            },
            Err(x) => print!("INFO,{} Nothing received or channel broke down in PinInterrupter receiving thread!", x)
            }
        }
    }
}

impl InterruptablePin for InputPin {
    fn on_low_signal(&mut self, sender: Sender<(usize, PinChange)>) {
        let i = self.get_id();
        self.set_async_interrupt(Trigger::FallingEdge, move |_| {
            sender
                .send((i, PinChange::Fall))
                .expect("Error on interupt FallingEdge: couldn't send over channel to hasmap");
            ()
        })
        .expect("Error on setting interupt for FallingEdge for InputPin");
    }
    fn on_high_signal(&mut self, sender: Sender<(usize, PinChange)>) {
        let i = self.get_id();
        self.set_async_interrupt(Trigger::RisingEdge, move |_| {
            sender
                .send((i, PinChange::Rise))
                .expect("Error on interupt RisingEdge: couldn't send over channel to hasmap");
            ()
        })
        .expect("Error on setting interupt for RisingEdge for InputPin");
    }
    fn get_id(&self) -> usize {
        self.pin() as usize
    }
}
