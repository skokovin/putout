use std::collections::{HashSet, VecDeque};

pub mod hull_state;
pub mod common_state;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RemoteCommand {
    MoveCameraToStartPos,
    MoveCameraToOID(i32),
}

pub struct CommandState {
    pub values: VecDeque<RemoteCommand>,
}

impl CommandState {
    pub fn new() -> Self {
        Self {
            values: VecDeque::new(),
        }
    }
    pub fn get_first(&mut self)->Option<RemoteCommand>{
        self.values.remove(0)
    }

}

pub struct HashI32State {
    pub values: HashSet<i32>,
    pub is_dirty: bool,
}

impl HashI32State {
    pub fn new() -> Self {
        Self {
            values: HashSet::new(),
            is_dirty: false,
        }
    }
}
pub struct ArrayF32State {
    pub values: Vec<f32>,
    pub is_dirty: bool,
}
impl ArrayF32State {
    pub fn new() -> Self {
        Self {
            values: vec![],
            is_dirty: false,
        }
    }
}

pub struct RemoteMeshData{
    pub is_dirty:bool,
    pub decoded_v: Vec<u8>,
    pub decoded_i: Vec<u8>,
    pub decoded_b: Vec<u8>,
    pub decoded_t: Vec<u8>,
}
impl RemoteMeshData {
    pub fn new() -> Self {
        Self {
            is_dirty: false,
            decoded_v: vec![],
            decoded_i: vec![],
            decoded_b: vec![],
            decoded_t: vec![],
        }
    }
    pub fn clean(&mut self){
        self.is_dirty= false;
        self.decoded_v= vec![];
        self.decoded_i= vec![];
        self.decoded_b= vec![];
        self.decoded_t= vec![];
    }
}