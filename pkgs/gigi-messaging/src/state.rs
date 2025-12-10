//! P2P State Management
//!
//! This module provides the state management for the P2P services.

use std::sync::Mutex;

/// P2P Service State
pub struct P2pState {
    // This will hold the P2P service instance when properly implemented
    pub service: Option<Mutex<String>>, // Simplified to just hold a string for now
}

impl P2pState {
    pub fn new() -> Self {
        Self { service: None }
    }
}

impl Default for P2pState {
    fn default() -> Self {
        Self::new()
    }
}

// Make it thread-safe
unsafe impl Send for P2pState {}
unsafe impl Sync for P2pState {}
