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
    // Start-function for the externaly run thread
    //  - uses stop to determine, when to end the loop
    fn start_interupter_thread(
        pin_dict: Arc<Mutex<HashMap<usize, PinLevel>>>,
        stop: Arc<AtomicBool>,
        receiver: Arc<Mutex<Receiver<(usize, PinLevel)>>>,
    ) -> JoinHandle<()> {
        thread::spawn(move || {
            while !stop.load(Ordering::Relaxed) {
                match receiver.lock().expect("Couldn't lock receiver in interrupt thread of PinHandle").recv() {
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
                            },
                };
            },
            Err(x) => info!("INFO,{} Nothing received or channel broke down in PinInterrupter receiving thread!", x)
            }
            }
            info!("Thread stopped!");
            ()
        })
    }

    ///Pin interrupt struct for async communication with the pins
    #[derive(Debug)]
    pub struct PinRegistry<T: InterruptablePin> {
        // threadsave pin dictionary for saving the state of the currently registered pins
        pin_level_dictionary: Arc<Mutex<HashMap<usize, PinLevel>>>,

        pin_dictionary: Arc<Mutex<HashMap<usize, T>>>,

        // sender for sending the state of the pins to the thread, used with the
        // InterruptablePin trait
        sender: SyncSender<(usize, PinLevel)>,
    }

    impl<T: InterruptablePin> Clone for PinRegistry<T> {
        fn clone_from(&mut self, source: &Self) {
            *self = source.clone()
        }

        fn clone(&self) -> Self {
            Self {
                pin_level_dictionary: self.pin_level_dictionary.clone(),
                pin_dictionary: self.pin_dictionary.clone(),
                sender: self.sender.clone(),
            }
        }
    }

    impl<T> PinRegistry<T>
    where
        T: InterruptablePin,
    {
        // creates the new PinRegistry object -> creates the HasmMaps and takes a sender for
        // regitering the interrupts
        fn new(sender: SyncSender<(usize, PinLevel)>) -> Self {
            Self {
                pin_level_dictionary: Arc::new(Mutex::new(HashMap::<usize, PinLevel>::new())),
                pin_dictionary: Arc::new(Mutex::new(HashMap::<usize, T>::new())),
                sender,
            }
        }

        // returns the pin level if pin with pin_id exists, concurrency save access
        pub fn get_pin_level(&self, pin_id: usize) -> Option<PinLevel> {
            match self
                .pin_level_dictionary
                .lock()
                .expect("Lock error on locking the dictionary while getting a pin level")
                .get(&pin_id)
            {
                None => None,
                Some(pin_level) => Some(pin_level.clone()),
            }
        }

        //NOTE: Maybe only allow to a remove a pin and return it instead of a reference (maybe
        //reference useless....)
        pub fn remove_pin(&self, pin_id: usize) -> Option<T> {
            self.pin_dictionary
                .lock()
                .expect("Lock error on locking the dictionary while getting the pin")
                .remove(&pin_id)
        }

        /// Registers the interrupt to the pins given in the implemented InterruptablePin trait
        /** Uses the functions on_low_signal and on_high_signal from InterruptablePin to give the pin a
         * reference to the sender which can then be used to send a PinChange::Rise or PinChange::Fall
         *
         * Return the old pin, if a pin was already registered
         */
        pub fn register_pin(&mut self, mut pin: T) -> Option<T> {
            pin.register_signal(self.sender.clone());
            info!("Registered pin_id: {}", pin.get_id());
            self.set_pin(pin.get_id(), pin)
        }

        fn set_pin_level(&mut self, pin_id: usize, pin_level: PinLevel) -> Option<PinLevel> {
            self.pin_level_dictionary
                .lock()
                .expect("Lock error on locking the dictionary while setting a pin level")
                .insert(pin_id, pin_level)
        }

        fn set_pin(&mut self, pin_id: usize, pin: T) -> Option<T> {
            self.pin_dictionary
                .lock()
                .expect("Lock error on lockig the dictionary while setting a pin")
                .insert(pin_id, pin)
        }
    }

    // struct for handeling the receiving thread of the pin library
    pub struct PinHandle {
        // receiver used in the thread and used to restart it
        receiver: Arc<Mutex<Receiver<(usize, PinLevel)>>>,

        // stop flag for the thread
        stop: Arc<AtomicBool>,

        // handle for the started thread
        receive_thread_handle: JoinHandle<()>,

        pin_level_dictionary: Arc<Mutex<HashMap<usize, PinLevel>>>,
    }

    impl PinHandle {
        // creates an object and starts the thread
        fn new(
            pin_level_dictionary: Arc<Mutex<HashMap<usize, PinLevel>>>,
            receiver: Arc<Mutex<Receiver<(usize, PinLevel)>>>,
        ) -> Self {
            let stop = Arc::new(AtomicBool::new(false));
            let receive_thread_handle =
                start_interupter_thread(pin_level_dictionary.clone(), stop.clone(), receiver.clone());

            Self {
                pin_level_dictionary,
                receiver,
                stop,
                receive_thread_handle,
            }
        }

        // sets the stop flag for savely stopping the thread
        // NOTE: currently a bit useless because thread can block if no message is received!
        pub fn stop_thread(&mut self) {
            self.stop.store(true, Ordering::Relaxed);
        }

        // starts a thread which receives the via interrupt send messages and stores them in the
        // dictionary
        pub fn start_thread(
            &mut self,
            pin_dict: Arc<Mutex<HashMap<usize, PinLevel>>>,
        ) -> thread::Result<bool> {
            if self.receive_thread_handle.is_finished() {
                // self.receive_thread_handle.join()?;
                self.receive_thread_handle =
                    start_interupter_thread(pin_dict, self.stop.clone(), self.receiver.clone());
                Ok(true)
            } else {
                Ok(false)
            }
        }
    }

    // initialise the PinHandle used for the thread handle and the Pin registry library for
    // getting the
    pub fn init_handle<T: InterruptablePin>() -> (PinHandle, PinRegistry<T>) {
        let (sender, receiver) = sync_channel(20);
        let receiver = Arc::new(Mutex::new(receiver));
        let pin_registry = PinRegistry::new(sender);
        let pin_handle = PinHandle::new(pin_registry.pin_level_dictionary.clone(), receiver);
        (pin_handle, pin_registry)
    }
} /* pin_handle */

// ----------- Implementation for GPIO Pins -----------

use log::info;
use pin_handle::{InterruptablePin, PinLevel};
use rppal::gpio::{InputPin, Trigger};
use std::sync::mpsc::SyncSender;

pub mod server;

// Implementation for GPIO Inputpin which registers a function for the pin which sends a signal to
// the thread that registers the entry
impl InterruptablePin for InputPin {
    fn register_signal(&mut self, sender: SyncSender<(usize, PinLevel)>) {
        let i = self.get_id();
        self.set_async_interrupt(Trigger::Both, move |l| {
            let pl = match l {
                rppal::gpio::Level::Low => PinLevel::Low,
                rppal::gpio::Level::High => PinLevel::High,
            };
            sender.send((i, pl)).unwrap_or_else(|e| {
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
