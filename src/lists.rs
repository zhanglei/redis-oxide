use crate::types::{Count, Index, InteractionRes, Key, State, StateInteration, Value};
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub enum ListOps {
    // List Operations
    LIndex(Key, Index),
    LLen(Key),
    LPop(Key),
    LPush(Key, Vec<Value>),
    LPushX(Key, Value),
    LRange(Key, Index, Index),
    LSet(Key, Index, Value),
    LTrim(Key, Index, Index),
    RPop(Key),
    RPush(Key, Vec<Value>),
    RPushX(Key, Value),
    RPopLPush(Key, Key),
}

macro_rules! read_lists {
    ($state:expr) => {
        $state.lists.read().unwrap()
    };
    ($state: expr, $key: expr) => {
        $state.lists.read().unwrap().get($key)
    };
}

macro_rules! write_lists {
    ($state:expr) => {
        $state.lists.write().unwrap()
    };
    ($state: expr, $key: expr) => {
        $state.lists.write().unwrap().get_mut($key)
    };
    ($state: expr, $key:expr, $var_name:ident) => {
        let mut __temp_name = $state.lists.write().unwrap();
        let $var_name = __temp_name.get_mut($key).unwrap();
    };
}

impl StateInteration for ListOps {
    #[allow(clippy::cognitive_complexity)]
    fn interact(self, state: State) -> InteractionRes {
        match self {
            ListOps::LPush(key, vals) => {
                state.create_list_if_necessary(&key);
                write_lists!(state, &key, list);
                for val in vals {
                    list.push_front(val)
                }
                InteractionRes::IntRes(list.len() as Count)
            }
            ListOps::LPushX(key, val) => {
                if !read_lists!(state).contains_key(&key) {
                    return InteractionRes::IntRes(0);
                }
                state.create_list_if_necessary(&key);
                write_lists!(state, &key, list);
                list.push_front(val);
                InteractionRes::IntRes(list.len() as Count)
            }
            ListOps::LLen(key) => match read_lists!(state, &key) {
                Some(l) => InteractionRes::IntRes(l.len() as Count),
                None => InteractionRes::IntRes(0),
            },
            ListOps::LPop(key) => match write_lists!(state, &key).and_then(VecDeque::pop_front) {
                Some(v) => InteractionRes::StringRes(v),
                None => InteractionRes::Nil,
            },
            ListOps::RPop(key) => match write_lists!(state, &key).and_then(VecDeque::pop_back) {
                Some(v) => InteractionRes::StringRes(v),
                None => InteractionRes::Nil,
            },
            ListOps::RPush(key, vals) => {
                state.create_list_if_necessary(&key);
                write_lists!(state, &key, list);
                for val in vals {
                    list.push_back(val)
                }
                InteractionRes::IntRes(list.len() as Count)
            }
            ListOps::RPushX(key, val) => {
                if !read_lists!(state).contains_key(&key) {
                    return InteractionRes::IntRes(0);
                }
                state.create_list_if_necessary(&key);
                write_lists!(state, &key, list);
                list.push_back(val);
                InteractionRes::IntRes(list.len() as Count)
            }
            ListOps::LIndex(key, index) => match write_lists!(state, &key) {
                Some(list) => {
                    let llen = list.len() as i64;
                    let real_index = if index < 0 { llen + index } else { index };
                    if !(0 <= real_index && real_index < llen) {
                        return InteractionRes::Error(b"Bad Range!");
                    }
                    let real_index = real_index as usize;
                    InteractionRes::StringRes(list[real_index].to_vec())
                }
                None => InteractionRes::Nil,
            },
            ListOps::LSet(key, index, value) => match write_lists!(state, &key) {
                Some(list) => {
                    let llen = list.len() as i64;
                    let real_index = if index < 0 { llen + index } else { index };
                    if !(0 <= real_index && real_index < llen) {
                        return InteractionRes::Error(b"Bad Range!");
                    }
                    let real_index = real_index as usize;
                    list[real_index] = value;
                    InteractionRes::Ok
                }
                None => InteractionRes::Error(b"No list at key!"),
            },
            ListOps::LRange(key, start_index, end_index) => match read_lists!(state, &key) {
                Some(list) => {
                    let start_index =
                        std::cmp::max(0, if start_index < 0 { 0 } else { start_index } as usize);
                    let end_index = std::cmp::min(
                        list.len(),
                        if end_index < 0 {
                            list.len() as i64 + end_index
                        } else {
                            end_index
                        } as usize,
                    );
                    let mut ret = Vec::new();
                    for (index, value) in list.iter().enumerate() {
                        if start_index <= index && index <= end_index {
                            ret.push(value.clone());
                        }
                        if index > end_index {
                            break;
                        }
                    }
                    InteractionRes::MultiStringRes(ret)
                }
                None => InteractionRes::MultiStringRes(vec![]),
            },
            ListOps::LTrim(key, start_index, end_index) => {
                match write_lists!(state, &key) {
                    Some(list) => {
                        let start_index = std::cmp::max(
                            0,
                            if start_index < 0 { 0 } else { start_index } as usize,
                        );
                        let end_index = std::cmp::min(
                            list.len(),
                            if end_index < 0 {
                                list.len() as i64 + end_index
                            } else {
                                end_index
                            } as usize,
                        ) + 1;
                        // Deal with right side
                        list.truncate(end_index);
                        // Deal with left side
                        for _ in 0..start_index {
                            list.pop_front();
                        }
                        InteractionRes::Ok
                    }
                    None => InteractionRes::Ok,
                }
            }
            ListOps::RPopLPush(source, dest) => {
                if source != dest {
                    state.create_list_if_necessary(&dest);
                }
                let mut lists = write_lists!(state);
                match lists.get_mut(&source) {
                    None => InteractionRes::Nil,
                    Some(source_list) => match source_list.pop_back() {
                        None => InteractionRes::Nil,
                        Some(value) => {
                            if source == dest {
                                source_list.push_back(value.clone());
                            } else {
                                lists.get_mut(&dest).unwrap().push_back(value.clone());
                            }
                            InteractionRes::StringRes(value)
                        }
                    },
                }
            }
        }
    }
}
