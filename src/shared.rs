use std::sync::{Condvar, Mutex};

pub trait IsDead {
    fn is_dead(&self) -> bool;
}

#[derive(Default)]
pub struct SharedState<S: std::cmp::PartialEq + std::cmp::Eq + IsDead + Copy> {
    state: Mutex<S>,
    cvar: Condvar,
}

impl<S: std::cmp::PartialEq + std::cmp::Eq + IsDead + Copy> SharedState<S> {
    pub fn new(state: S) -> Self {
        SharedState {
            state: Mutex::new(state),
            cvar: Condvar::new(),
        }
    }

    pub fn set_state(&self, state: S) {
        let mut state_m = self.state.lock().unwrap();
        *state_m = state;
        self.cvar.notify_all();
    }

    pub fn get_state(&self) -> S {
        let state_m = self.state.lock().unwrap();
        *state_m
    }

    pub fn await_state(&self, state: S) {
        let mut state_m = self.state.lock().unwrap();
        while *state_m != state && !state_m.is_dead() {
            state_m = self.cvar.wait(state_m).unwrap();
        }
    }

    pub fn await_until<F>(&self, condition: F)
    where
        F: Fn(S) -> bool,
    {
        let mut state_m = self.state.lock().unwrap();
        while !condition(*state_m) && !state_m.is_dead() {
            state_m = self.cvar.wait(state_m).unwrap();
        }
    }
}
