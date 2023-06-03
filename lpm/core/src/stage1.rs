use common::pkg::{ScriptPhase, Stage1Script};
use ehandle::{lpm::LpmError, pkg::PackageErrorKind, ErrorCommons, MainError};
use std::{
    fs::File,
    io::{self, Read},
    path::Path,
    process::Command,
};

pub const PKG_SCRIPTS_DIR: &str = "/var/lib/lpm/pkg";

pub(crate) trait Stage1Tasks {
    fn execute_script(&self, caller_phase: ScriptPhase) -> Result<(), LpmError<MainError>>;
}

impl Stage1Tasks for Vec<Stage1Script> {
    #[allow(unused_variables)]
    fn execute_script(&self, caller_phase: ScriptPhase) -> Result<(), LpmError<MainError>> {
        fn prepare_script(script: &Stage1Script) -> String {
            format!(
                r#"
                set -e

                {}
                "#,
                &script.contents
            )
        }

        if let Some(script) = self.iter().find(|s| s.phase == caller_phase) {
            let output = Command::new("bash")
                .arg("-c")
                .arg(prepare_script(script))
                .output()?;

            if !output.status.success() {
                return Err(PackageErrorKind::FailedExecutingStage1Script {
                    script_name: script.path.to_string_lossy().to_string(),
                    output: String::from_utf8_lossy(&output.stderr).to_string(),
                }
                .to_lpm_err()
                .into());
            }
        }

        Ok(())
    }
}

pub fn get_scripts(scripts_dir: &Path) -> Result<Vec<Stage1Script>, LpmError<io::Error>> {
    let mut scripts = vec![];

    {
        let path = scripts_dir.join("pre_install");
        if let Ok(mut file) = File::open(&path) {
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;

            scripts.push(Stage1Script {
                contents,
                path,
                phase: ScriptPhase::PreInstall,
            });
        }
    }

    {
        let path = scripts_dir.join("post_install");
        if let Ok(mut file) = File::open(&path) {
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;

            scripts.push(Stage1Script {
                contents,
                path,
                phase: ScriptPhase::PostInstall,
            });
        }
    }

    {
        let path = scripts_dir.join("pre_delete");
        if let Ok(mut file) = File::open(&path) {
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;

            scripts.push(Stage1Script {
                contents,
                path,
                phase: ScriptPhase::PreDelete,
            });
        }
    }

    {
        let path = scripts_dir.join("post_delete");
        if let Ok(mut file) = File::open(&path) {
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;

            scripts.push(Stage1Script {
                contents,
                path,
                phase: ScriptPhase::PostDelete,
            });
        }
    }

    {
        let path = scripts_dir.join("pre_downgrade");
        if let Ok(mut file) = File::open(&path) {
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;

            scripts.push(Stage1Script {
                contents,
                path,
                phase: ScriptPhase::PreDowngrade,
            });
        }
    }

    {
        let path = scripts_dir.join("post_downgrade");
        if let Ok(mut file) = File::open(&path) {
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;

            scripts.push(Stage1Script {
                contents,
                path,
                phase: ScriptPhase::PostDowngrade,
            });
        }
    }

    {
        let path = scripts_dir.join("pre_upgrade");
        if let Ok(mut file) = File::open(&path) {
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;

            scripts.push(Stage1Script {
                contents,
                path,
                phase: ScriptPhase::PreUpgrade,
            });
        }
    }

    {
        let path = scripts_dir.join("post_upgrade");
        if let Ok(mut file) = File::open(&path) {
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;

            scripts.push(Stage1Script {
                contents,
                path,
                phase: ScriptPhase::PostUpgrade,
            });
        }
    }

    Ok(scripts)
}
