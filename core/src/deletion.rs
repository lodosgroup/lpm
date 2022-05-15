use common::pkg::LodPkg;
use ehandle::RuntimeError;

pub trait DeletionTasks {
    fn start_deletion(&mut self) -> Result<(), RuntimeError>;
}

impl<'a> DeletionTasks for LodPkg<'a> {
    fn start_deletion(&mut self) -> Result<(), RuntimeError> {
        todo!()
    }
}
