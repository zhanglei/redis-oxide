// use rand::Rng;
use crate::types::{ReturnValue, State};
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;
use std::task::Waker;

impl fmt::Display for ReturnValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ReturnValue::Ok => write!(f, "OK"),
            ReturnValue::StringRes(s) => write!(f, "{:?}", s),
            ReturnValue::IntRes(i) => write!(f, "{:?}", i),
            ReturnValue::MultiStringRes(ss) => write!(f, "{:?}", ss),
            ReturnValue::Nil => write!(f, "(nil)"),
            ReturnValue::Error(e) => write!(f, "ERR {:?}", e),
            ReturnValue::Array(a) => write!(f, "{:?}", a),
            // TODO: Figure out how make futures work
            // ReturnValue::FutureRes(v, _) => (*v).fmt(f),
            // ReturnValue::FutureResValue(_) => unreachable!(),
        }
    }
}

impl State {
    pub fn create_list_if_necessary(&self, list_key: &[u8]) {
        if !self.lists.read().contains_key(list_key) {
            self.lists
                .write()
                .insert(list_key.to_vec(), VecDeque::new());
        }
    }

    pub fn create_hashes_if_necessary(&self, hashes_key: &[u8]) {
        if !self.hashes.read().contains_key(hashes_key) {
            self.hashes
                .write()
                .insert(hashes_key.to_vec(), HashMap::new());
        }
    }

    pub fn create_set_if_necessary(&self, set_key: &[u8]) {
        if !self.sets.read().contains_key(set_key) {
            self.sets.write().insert(set_key.to_vec(), HashSet::new());
        }
    }

    pub fn create_zset_if_necessary(&self, set_key: &[u8]) {
        if !self.zsets.read().contains_key(set_key) {
            self.zsets
                .write()
                .insert(set_key.to_vec(), Default::default());
        }
    }

    pub fn wake_list(&self, list_key: &[u8]) {
        let mut list_wakers = self.list_wakers.lock();
        list_wakers.wake(&list_key);
    }

    pub fn sleep_list(&self, list_key: &[u8], waker: Waker) {
        let mut list_wakers = self.list_wakers.lock();
        list_wakers.add(&list_key, waker);
    }
}
