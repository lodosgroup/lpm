pub mod meta;
pub mod system;
pub mod version;

pub trait ParserTasks {
    fn deserialize(path: &str) -> Self;
}
