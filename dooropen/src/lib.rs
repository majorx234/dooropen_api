use rppal::gpio::{InputPin, Trigger};
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

pub mod server;

enum PinChange {
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
    activated_pin_dict: Arc<Mutex<HashMap<usize, bool>>>,
    stop: Arc<AtomicBool>,
    sender: Sender<(usize, PinChange)>,
    receive_thread: &JoinHandle<()>,
}

impl PinInterrupter {
    ///Creates an object and starts the receive-thread for incomming signals
    /** Initialises a thread-save dictionary, and creates a channel for thread-communication.
     *
     * Also starts a thread for receiving incomming signals and changes the dictionary entry as
     * follows:
     *      - if no entry could be found on that pin, then the inserted value is corrosponding to
     *      Rise(true) and Fall(false)
     *      - if an entry could be found, the entry state is changed (true -> false; false ->
     *      true), intependant of the Rise and Fall
     */
    // TODO: I dont like the behavior of incomming messages <23-02-23, Nikl> //
    pub fn new() -> Self {
        let (s, r): (Sender<(usize, PinChange)>, Receiver<(usize, PinChange)>) = channel();
        let pin_dir = Arc::new(Mutex::new(HashMap::new()));
        let st = Arc::new(AtomicBool::new(false));
        let th: &JoinHandle<()>;

        // TODO: behavior maybe changeble or generic? <23-02-23, nikl> //
        {
            let r = r;
            let pin_dir = pin_dir.clone();
            let st = st.clone();
            th = &thread::spawn(move || {
                while st.load(Ordering::Relaxed) {
                    match r.recv_timeout(Duration::new(1,0)) {
                        Ok((pos,change)) => {
                            let dir = pin_dir.lock().unwrap();
                            match dir.get(&pos) {
                                Some(x) => dir.insert(pos,!x),
                                None => {
                                    match change {
                                        PinChange::Rise => dir.insert(pos,true),
                                        PinChange::Fall => dir.insert(pos,false),
                                    }
                                }
                            };
                            ()
                        },
                        Err(x) => print!("INFO,[x]: Nothing received or channel broke down in PinInterrupter receiving thread!")
                    }
                }
            });
        }
        Self {
            activated_pin_dict: Arc::clone(&pin_dir),
            sender: s.clone(),
            receive_thread: th,
            stop: Arc::clone(&st),
        }
    }
    /// Registers the interups to the pins given in the implemented InterruptablePin trait
    /** Uses the functions on_low_signal and on_high_signal from InterruptablePin to give the pin a
     * reference to the sender which can then be used to send a PinChange::Rise or PinChange::Fall
     *
     */
    pub fn register_pin(&mut self, mut pin: impl InterruptablePin) {
        let pin_id = pin.get_id();
        pin.on_low_signal(self.sender.clone());
        pin.on_high_signal(self.sender.clone());
    }

    ///Returns the optional Value of the pin dictionary, if pin is active
    pub fn get_pin_state(&self, pin: usize) -> Option<bool> {
                    self.activated_pin_dict.lock().unwrap().clone().get(&pin).copied()
    }
}

/// Implementing Drop for a save stop of the background thread
impl Drop for PinInterrupter {
    /// Drops the Pininterrupter and stops the receive thread for the incomming signals of the pins or prints an error if
    /// something go's wrong
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        match self.receive_thread.join() {
            Ok(x) => x,
            Err(_) => print!("ERROR: Couldn't join the receiving thread from PinInterrupter!"),
        }
    }
}

impl InterruptablePin for InputPin {
    fn on_low_signal(&mut self, sender: Sender<(usize, PinChange)>) {
        self.set_async_interrupt(Trigger::RisingEdge, move |_| {
            sender.send((self.get_id(), PinChange::Rise));
            ()
        });
    }
    fn on_high_signal(&mut self, sender: Sender<(usize, PinChange)>) {
        self.set_async_interrupt(Trigger::RisingEdge, move |_| {
            sender.send((self.get_id(), PinChange::Fall));
            ()
        });
    }
    fn get_id(&self) -> usize {
        self.pin() as usize
    }
}
