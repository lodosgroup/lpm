use common::pkg::Stage1Script;
use std::path::Path;

pub fn get_scripts(scripts_dir: &Path) -> Vec<Stage1Script> {
    let mut scripts = vec![];

    {
        let pre_install = scripts_dir.join("pre_install");
        if pre_install.exists() {
            scripts.push(Stage1Script::PreInstall(pre_install));
        }
    }

    {
        let post_install = scripts_dir.join("post_install");
        if post_install.exists() {
            scripts.push(Stage1Script::PostInstall(post_install));
        }
    }

    {
        let pre_delete = scripts_dir.join("pre_delete");
        if pre_delete.exists() {
            scripts.push(Stage1Script::PreDelete(pre_delete));
        }
    }

    {
        let post_delete = scripts_dir.join("post_delete");
        if post_delete.exists() {
            scripts.push(Stage1Script::PostDelete(post_delete));
        }
    }

    {
        let pre_downgrade = scripts_dir.join("pre_downgrade");
        if pre_downgrade.exists() {
            scripts.push(Stage1Script::PreDowngrade(pre_downgrade));
        }
    }

    {
        let post_downgrade = scripts_dir.join("post_downgrade");
        if post_downgrade.exists() {
            scripts.push(Stage1Script::PostDowngrade(post_downgrade));
        }
    }

    {
        let pre_upgrade = scripts_dir.join("pre_upgrade");
        if pre_upgrade.exists() {
            scripts.push(Stage1Script::PreUpgrade(pre_upgrade));
        }
    }

    {
        let post_upgrade = scripts_dir.join("post_upgrade");
        if post_upgrade.exists() {
            scripts.push(Stage1Script::PostUpgrade(post_upgrade));
        }
    }

    scripts
}
