use crate::builders::post_deployment::DeploymentTree;
use crate::builders::utils::{io::read_genesis_yaml, ContextExtension};
use crate::builders::VitBackendSettingsBuilder;
use crate::config::Initials;
use crate::Result;
use jormungandr_lib::interfaces::Initial;
use jormungandr_scenario_tests::Context;
use jortestkit::prelude::read_file;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(setting = structopt::clap::AppSettings::ColoredHelp)]
pub struct SnapshotCommandArgs {
    /// Careful! directory would be removed before export
    #[structopt(long = "root-dir", default_value = "./data")]
    pub output_directory: PathBuf,

    /// how many addresses to generate
    #[structopt(long = "count")]
    pub initials: Option<usize>,

    #[structopt(long = "initials", conflicts_with = "count")]
    pub initials_mapping: Option<PathBuf>,

    #[structopt(long = "global-pin", default_value = "1234")]
    pub global_pin: String,

    #[structopt(long = "skip-qr-generation")]
    pub skip_qr_generation: bool,
}

impl SnapshotCommandArgs {
    pub fn exec(self) -> Result<()> {
        std::env::set_var("RUST_BACKTRACE", "full");

        let context = Context::empty_from_dir(&self.output_directory);

        let mut quick_setup = VitBackendSettingsBuilder::new();

        if let Some(mapping) = self.initials_mapping {
            let content = read_file(mapping);
            let initials: Initials =
                serde_json::from_str(&content).expect("JSON was not well-formatted");
            quick_setup.initials(initials);
        } else {
            quick_setup.initials_count(self.initials.unwrap(), &self.global_pin);
        }

        if self.skip_qr_generation {
            quick_setup.skip_qr_generation();
        }

        if !self.output_directory.exists() {
            std::fs::create_dir_all(&self.output_directory)?;
        } else {
            std::fs::remove_dir_all(&self.output_directory)?;
        }

        let deployment_tree = DeploymentTree::new(&self.output_directory, quick_setup.title());

        let (_, controller, _, _) = quick_setup.build(context)?;

        let genesis_yaml = deployment_tree.genesis_path();

        //remove all files except qr codes and genesis
        for entry in std::fs::read_dir(&deployment_tree.root_path())? {
            let entry = entry?;
            let md = std::fs::metadata(entry.path()).unwrap();
            if md.is_dir() {
                continue;
            }

            if entry.path() == genesis_yaml {
                continue;
            }

            //skip secret key generation
            if entry.file_name().to_str().unwrap().contains("wallet") {
                continue;
            }

            std::fs::remove_file(entry.path())?;
        }

        if !self.skip_qr_generation {
            //rename qr codes to {address}_{pin}.png syntax
            let qr_codes = deployment_tree.qr_codes_path();
            let mut i = 1;
            for entry in std::fs::read_dir(&qr_codes)? {
                let entry = entry?;
                let path = entry.path();

                let file_name = path
                    .file_stem()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .replace(&format!("_{}", self.global_pin), "");

                let wallet = controller.wallet(&file_name)?;
                let new_file_name = format!("{}_{}_{}.png", i, wallet.address(), self.global_pin);
                i += 1;
                std::fs::rename(
                    path.clone(),
                    std::path::Path::new(path.parent().unwrap()).join(&new_file_name),
                )?;
            }
            println!("Qr codes dumped into {:?}", qr_codes);
        }

        // write snapshot.json
        let config = read_genesis_yaml(&genesis_yaml)?;

        let initials: Vec<Initial> = config
            .initial
            .iter()
            .filter(|x| matches!(x, Initial::Fund { .. }))
            .cloned()
            .collect();

        let snapshot = Snapshot { initial: initials };
        let snapshot_ser = serde_json::to_string_pretty(&snapshot)?;

        let mut file = std::fs::File::create(deployment_tree.root_path().join("snapshot.json"))?;
        file.write_all(snapshot_ser.as_bytes())?;
        std::fs::remove_file(genesis_yaml)?;

        println!("Snapshot dumped into {:?}", file);

        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
struct Snapshot {
    pub initial: Vec<Initial>,
}