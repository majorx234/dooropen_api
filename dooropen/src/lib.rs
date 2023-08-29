use log::info;
use rppal::gpio::{InputPin, Trigger};
use std::{
    collections::HashMap,
    ops::Not,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{sync_channel, Receiver, Sender, SyncSender},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

pub mod server;

#[derive(Debug)]
pub enum PinChange {
    Rise,
    Fall,
}

/// Trait for the pins needed by PinInterrupter to register functions
pub trait InterruptablePin {
    /// registers a function with a sender who should send the enum PinChange which is then
    /// received by the corresponding PinInterrupter object
    fn on_low_signal(&mut self, sender: SyncSender<(usize, PinChange)>);

    /// registers a function with a sender who should send the enum PinChange which is then
    /// received by the corresponding PinInterrupter object
    fn on_high_signal(&mut self, sender: SyncSender<(usize, PinChange)>);

    /// returns the specific pin-idvivoactive
    fn get_id(&self) -> usize;
}

///Pin interrupt struct for async communication with the pins
#[derive(Clone)]
pub struct PinInterrupter {
    // receive_thread: JoinHandle<()>,
    // stop: Arc<AtomicBool>,
    activated_pin_dict: HashMap<usize, bool>,
    sender: SyncSender<(usize, PinChange)>,
}

impl PinInterrupter {
    pub fn new(sync_sender: SyncSender<(usize, PinChange)>) -> Self {
        let pin_dir = HashMap::<usize, bool>::new();

        Self {
            activated_pin_dict: pin_dir,
            sender: sync_sender,
        }
    }

    /// Registers the interups to the pins given in the implemented InterruptablePin trait
    /** Uses the functions on_low_signal and on_high_signal from InterruptablePin to give the pin a
     * reference to the sender which can then be used to send a PinChange::Rise or PinChange::Fall
     *
     */
    pub fn register_pin(&mut self, pin: &mut impl InterruptablePin) {
        // pin.on_low_signal(self.sender.clone());
        pin.on_high_signal(self.sender.clone());
        self.activated_pin_dict.insert(pin.get_id(), false);
        info!("Registered pin_id: {}", pin.get_id());
    }

    ///Returns the optional Value of the pin dictionary, if pin is active
    pub fn get_pin_state(&self, pin: usize) -> Option<bool> {
        self.activated_pin_dict.clone().get(&pin).copied()
    }

    // Start-function for the externaly run thread
    //  - uses stop to determine, when to end the loop
}
pub fn start_thread(
    pin_int: Arc<Mutex<PinInterrupter>>,
    stop: Arc<AtomicBool>,
    receiver: Receiver<(usize, PinChange)>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        while !stop.load(Ordering::Relaxed) {
            match receiver.recv() {
            Ok((pos,change)) => {
                info!("Received change: {:?} on Position: {:?}", change, pos);
                let dir: &mut HashMap<usize, bool> = &mut pin_int
                    .lock()
                    .expect("Couldn't lock on dictionary clone!")
                    .activated_pin_dict;
                match  &mut dir.get_mut(&( pos.clone() )) {
                    Some(x) => {
                        **x = x.not();
                        info!("Changed entry in {:?} to {:?}",pos,dir.get(&pos.clone()));
                    },
                    None => {
                        match change {
                            PinChange::Rise => { dir.insert(pos,true);
                            info!("Inserted {} into dir on pos: {}", true, pos);},
                            PinChange::Fall => {
                                dir.insert(pos,false);
                            info!("Inserted {} into dir on pos: {}", true, pos);
                            },
                        };
                        ()
                    }
                };
                ()
            },
            Err(x) => print!("INFO,{} Nothing received or channel broke down in PinInterrupter receiving thread!", x)
            }
        }
        info!("Thread stopped!");
    })
}

impl InterruptablePin for InputPin {
    fn on_low_signal(&mut self, sender: SyncSender<(usize, PinChange)>) {
        let i = self.get_id();
        self.set_async_interrupt(Trigger::FallingEdge, move |_| {
            sender
                .send((i, PinChange::Fall))
                .expect("Error on interupt FallingEdge: couldn't send over channel to hasmap");
            // println!("Falling edge on: {}", i);
            ()
        })
        .expect("Error on setting interupt for FallingEdge for InputPin");
        info!("Registered async interupt on: {}", i);
    }
    fn on_high_signal(&mut self, sender: SyncSender<(usize, PinChange)>) {
        let i = self.get_id();
        self.set_async_interrupt(Trigger::RisingEdge, move |_| {
            sender.send((i, PinChange::Rise)).unwrap_or_else(|e| {
                info!(
                    "couldn't send over channel to hasmap, disconnected?; send: {:?}, Err: {:?}",
                    i, e
                );
                ()
            });
            // .expect("Error on interupt RisingEdge: couldn't send over channel to hasmap");
            println!("Rising edge on: {}", i);
            ()
        })
        .expect("Error on setting interupt for RisingEdge for InputPin");
        info!("Registered async interupt on: {}", i);
    }
    fn get_id(&self) -> usize {
        self.pin() as usize
    }
}
