use parking_lot::RwLock;
use rdev::{listen, EventType};
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivationState {
    Inactive,
    Active,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivationSource {
    LeftChord,
    RightChord,
    EitherChord,
}

pub struct Keys {
    meta_left: AtomicBool,
    meta_right: AtomicBool,
    alt_pressed: AtomicU8,
    state: RwLock<ActivationState>,
    source: RwLock<Option<ActivationSource>>,
    enabled_left: AtomicBool,
    enabled_right: AtomicBool,
    callback: RwLock<Option<Box<dyn Fn(ActivationState, Option<ActivationSource>) + Send + Sync>>>,
}

impl Keys {
    pub fn new() -> Self {
        Self {
            meta_left: AtomicBool::new(false),
            meta_right: AtomicBool::new(false),
            alt_pressed: AtomicU8::new(0),
            state: RwLock::new(ActivationState::Inactive),
            source: RwLock::new(None),
            enabled_left: AtomicBool::new(true),
            enabled_right: AtomicBool::new(true),
            callback: RwLock::new(None),
        }
    }

    pub fn set_enabled(&self, left: bool, right: bool) {
        self.enabled_left.store(left, Ordering::SeqCst);
        self.enabled_right.store(right, Ordering::SeqCst);
    }

    pub fn on_activation(
        &self,
        callback: Box<dyn Fn(ActivationState, Option<ActivationSource>) + Send + Sync>,
    ) {
        *self.callback.write() = Some(callback);
    }

    fn check_chord(&self) -> Option<ActivationSource> {
        let meta_left = self.meta_left.load(Ordering::SeqCst);
        let meta_right = self.meta_right.load(Ordering::SeqCst);
        let alt = self.alt_pressed.load(Ordering::SeqCst) > 0;

        if !alt {
            return None;
        }

        if meta_left && self.enabled_left.load(Ordering::SeqCst) {
            Some(ActivationSource::LeftChord)
        } else if meta_right && self.enabled_right.load(Ordering::SeqCst) {
            Some(ActivationSource::RightChord)
        } else {
            None
        }
    }

    fn handle_key(&self, key: rdev::Key, pressed: bool) {
        match key {
            rdev::Key::MetaLeft => self.meta_left.store(pressed, Ordering::SeqCst),
            rdev::Key::MetaRight => self.meta_right.store(pressed, Ordering::SeqCst),
            rdev::Key::Alt => {
                if pressed {
                    self.alt_pressed.fetch_add(1, Ordering::SeqCst);
                } else {
                    self.alt_pressed.fetch_sub(1, Ordering::SeqCst);
                }
            }
            _ => return,
        }

        let chord = self.check_chord();
        let current_state = *self.state.read();

        if pressed {
            if current_state == ActivationState::Inactive && chord.is_some() {
                *self.state.write() = ActivationState::Active;
                *self.source.write() = chord;
                if let Some(ref cb) = *self.callback.read() {
                    cb(ActivationState::Active, chord);
                }
            }
        } else {
            if current_state == ActivationState::Active && chord.is_none() {
                *self.state.write() = ActivationState::Inactive;
                let src = self.source.read().clone();
                *self.source.write() = None;
                if let Some(ref cb) = *self.callback.read() {
                    cb(ActivationState::Inactive, src);
                }
            }
        }
    }

    pub fn get_state(&self) -> ActivationState {
        *self.state.read()
    }

    pub fn get_source(&self) -> Option<ActivationSource> {
        *self.source.read()
    }

    pub fn start_listening(self: &Arc<Self>) -> Result<(), rdev::ListenError> {
        let keys = Arc::clone(self);
        listen(move |event| match event.event_type {
            EventType::KeyPress(key) => {
                keys.handle_key(key, true);
            }
            EventType::KeyRelease(key) => {
                keys.handle_key(key, false);
            }
            _ => {}
        })
    }
}

impl Default for Keys {
    fn default() -> Self {
        Self::new()
    }
}

pub struct KeysHandle {
    keys: Arc<Keys>,
}

impl KeysHandle {
    pub fn new() -> Result<Self, rdev::ListenError> {
        let keys = Arc::new(Keys::new());
        let keys_clone = Arc::clone(&keys);
        keys.start_listening()?;
        Ok(Self { keys: keys_clone })
    }

    pub fn set_enabled(&self, left: bool, right: bool) {
        self.keys.set_enabled(left, right);
    }

    pub fn on_activation<F>(&self, callback: F)
    where
        F: Fn(ActivationState, Option<ActivationSource>) + Send + Sync + 'static,
    {
        self.keys.on_activation(Box::new(callback));
    }

    pub fn get_state(&self) -> ActivationState {
        self.keys.get_state()
    }

    pub fn get_source(&self) -> Option<ActivationSource> {
        self.keys.get_source()
    }
}

impl Default for KeysHandle {
    fn default() -> Self {
        Self::new().expect("Failed to initialize keyboard listener")
    }
}
