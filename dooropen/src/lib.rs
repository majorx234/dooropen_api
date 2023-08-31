use log::info;
use pin_handle::{InterruptablePin, PinLevel};
use rppal::gpio::{InputPin, Trigger};
use std::sync::mpsc::SyncSender;

pub mod server;

pub mod pin_handle {
    use std::{
        collections::HashMap,
        sync::{
            atomic::{AtomicBool, Ordering},
            mpsc::{sync_channel, Receiver, SyncSender},
            Arc, Mutex,
        },
        thread::{self, JoinHandle},
    };

    use log::info;

    #[derive(Debug, Clone, Copy)]
    pub enum PinLevel {
        High,
        Low,
    }

    /// Trait for the pins needed by PinInterrupter to register functions
    pub trait InterruptablePin {
        /// registers a function with a sender who should send the enum PinChange which is then
        /// received by the corresponding PinInterrupter object
        fn register_signal(&mut self, sender: SyncSender<(usize, PinLevel)>);

        /// returns the specific pin-id
        fn get_id(&self) -> usize;
    }

    ///Pin interrupt struct for async communication with the pins
    pub struct PinInterrupter {
        // threadsave pin dictionary for saving the state of the currently registered pins
        activated_pin_dict: Arc<Mutex<HashMap<usize, PinLevel>>>,

        // sender for sending the state of the pins to the PinInterrupter thread, used with the
        // InterruptablePin trait
        sender: SyncSender<(usize, PinLevel)>,

        // stop flag for the thread
        stop: Arc<AtomicBool>,

        // receiver used in the PinInterrupter thread
        receiver: Receiver<(usize, PinLevel)>,
    }

    impl PinInterrupter {
        // creates the new PinInterrupter object -> creates a HashMap and opens a channel for
        // communication with the interups
        pub fn new() -> Self {
            let (sync_sender, receiver) = sync_channel(10);
            let pin_dir = Arc::new(Mutex::new(HashMap::<usize, PinLevel>::new()));
            Self {
                activated_pin_dict: pin_dir,
                sender: sync_sender,
                stop: Arc::new(AtomicBool::new(false)),
                receiver,
            }
        }

        /// Registers the interups to the pins given in the implemented InterruptablePin trait
        /** Uses the functions on_low_signal and on_high_signal from InterruptablePin to give the pin a
         * reference to the sender which can then be used to send a PinChange::Rise or PinChange::Fall
         */
        pub fn register_pin(&mut self, pin: &mut impl InterruptablePin) {
            pin.register_signal(self.sender.clone());
            info!("Registered pin_id: {}", pin.get_id());
        }

        // ///Returns the optional Value of the pin dictionary, if pin is active
        // pub fn get_pin_state(&self, pin: usize) -> Option<PinLevel> {
        //     self.activated_pin_dict
        //         .lock()
        //         .expect("Couldn't lock activated_pin_dict on getting pin state")
        //         .get(&pin)
        //         .copied()
        // }

        // Returns a reference to the internal dictionary with the in entry, accassable with the
        // InterruptablePin::get_id() function as key
        // TODO: maybe a wrapper for the dictonary for better access?
        pub fn get_pin_dictionary(self) -> Arc<Mutex<HashMap<usize, PinLevel>>> {
            self.activated_pin_dict.clone()
        }

        // starts a thread which receives the via interrupt send messages and stores them in the
        // dictionary
        pub fn start(&mut self) -> JoinHandle<()> {
            start_interupter_thread(self.activated_pin_dict, self.stop, self.receiver)
        }

        // sets the stop flag for savely stopping the thread
        // NOTE: currently a bit useless because thread can block if no message is received!
        pub fn stop(&mut self) {
            self.stop.store(true, Ordering::Relaxed);
        }
    }

    // Start-function for the externaly run thread
    //  - uses stop to determine, when to end the loop
    fn start_interupter_thread(
        pin_dict: Arc<Mutex<HashMap<usize, PinLevel>>>,
        stop: Arc<AtomicBool>,
        receiver: Receiver<(usize, PinLevel)>,
    ) -> JoinHandle<()> {
        thread::spawn(move || {
            while !stop.load(Ordering::Relaxed) {
                match receiver.recv() {
            Ok((pos,change)) => {
                info!("Received change: {:?} on Position: {:?}", change, pos);
                let dict = &mut pin_dict
                    .lock()
                    .expect("Couldn't lock on dictionary clone!");
                // let dict_entry = &mut dir.get_mut(&( pos.clone() ));
                match  dict.get_mut(&pos){
                    Some(x) => {
                        // info!("Changed entry in {:?} from {:?} to {:?}",pos,dict_entry, change);
                        *x = change;
                    },
                    None => {
                             dict.insert(pos,change);
                            info!("Inserted {:?} into dir on pos: {}", change, pos);
                            ()
                            },
                };
                ()
            },
            Err(x) => info!("INFO,{} Nothing received or channel broke down in PinInterrupter receiving thread!", x)
            }
            }
            info!("Thread stopped!");
        })
    }
} /* pin_handle */

// Implementation for GPIO Inputpin which registers a function for the pin which sends a signal to
// the thread that registers the entry
impl InterruptablePin for InputPin {
    fn register_signal(&mut self, sender: SyncSender<(usize, PinLevel)>) {
        let i = self.get_id();
        self.set_async_interrupt(Trigger::Both, move |l| {
            sender.send((i, PinLevel::High)).unwrap_or_else(|e| {
                info!(
                    "couldn't send over channel, disconnected?; send: {:?}, Err: {:?}",
                    i, e
                );
            });
        })
        .expect("Error on setting interupt for RisingEdge for InputPin");
        info!("Registered async interupt on: {}", i);
    }

    fn get_id(&self) -> usize {
        self.pin() as usize
    }
}
