pub mod md5;
pub mod padding;

pub trait ComputeHash {
    fn process(msg: Vec<u8>) -> String;
}
