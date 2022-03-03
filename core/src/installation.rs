use lpm_io::file::copy_recursively;

use crate::pkg::LodPkg;

impl<'a> super::InstallationTasks for LodPkg<'a> {
    fn install_program(&self) -> Result<(), std::io::Error> {
        let src = super::EXTRACTION_OUTPUT_PATH.to_string()
            + "/"
            + self.path.file_stem().unwrap().to_str().unwrap()
            + "/program/";

        copy_recursively(&src, "/")?;

        Ok(())
    }
}

