use std::ascii::escape_default;
use std::cell::RefCell;
use std::io::{BufReader, Read};
use std::rc::Rc;
use std::slice::Chunks;
use std::sync::{LockResult, Mutex};
use cgmath::vec1;
use itertools::Itertools;
use log::{info, Level, warn};
use miniz_oxide::inflate::decompress_to_vec;
use once_cell::sync::Lazy;

pub static mut HULL_V: Vec<u8> = vec![];
pub static mut HULL_I: Vec<u8> = vec![];


