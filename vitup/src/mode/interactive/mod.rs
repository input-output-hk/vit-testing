mod args;
mod controller;

pub use args::{describe, show};
pub use controller::VitUserInteractionController;
use hersir::controller::interactive::args::explorer;
use hersir::controller::interactive::args::send;
use hersir::controller::UserInteractionController;
use jortestkit::prelude::ConsoleWriter;
use jortestkit::prelude::InteractiveCommandError;
use jortestkit::prelude::InteractiveCommandExec;
use std::ffi::OsStr;
use structopt::{clap::AppSettings, StructOpt};

pub struct VitInteractiveCommandExec {
    pub vit_controller: VitUserInteractionController,
    pub controller: UserInteractionController,
}

impl VitInteractiveCommandExec {
    pub fn vit_controller_mut(&mut self) -> &mut VitUserInteractionController {
        &mut self.vit_controller
    }

    pub fn controller_mut(&mut self) -> &mut UserInteractionController {
        &mut self.controller
    }
}

impl VitInteractiveCommandExec {
    pub fn tear_down(self) {
        self.vit_controller.finalize();
        // TODO: what happend to this?
        // self.controller.finalize();
    }
}

impl InteractiveCommandExec for VitInteractiveCommandExec {
    fn parse_and_exec(
        &mut self,
        tokens: Vec<String>,
        console: ConsoleWriter,
    ) -> std::result::Result<(), InteractiveCommandError> {
        match VitInteractiveCommand::from_iter_safe(&mut tokens.iter().map(OsStr::new)) {
            Ok(interactive) => {
                if let Err(err) = {
                    match interactive {
                        VitInteractiveCommand::Show(show) => {
                            show.exec(self);
                            Ok(())
                        }
                        VitInteractiveCommand::Exit => Ok(()),
                        VitInteractiveCommand::Describe(describe) => describe.exec(self),
                        VitInteractiveCommand::Send(send) => {
                            send.exec(self.controller_mut()).map_err(Into::into)
                        }
                        VitInteractiveCommand::Explorer(explorer) => {
                            explorer.exec(self.controller_mut()).map_err(Into::into)
                        }
                    }
                } {
                    console.format_error(InteractiveCommandError::UserError(err.to_string()));
                }
            }
            Err(err) => console.show_help(InteractiveCommandError::UserError(err.to_string())),
        }
        Ok(())
    }
}

#[derive(StructOpt, Debug)]
#[structopt(setting = AppSettings::NoBinaryName)]
pub enum VitInteractiveCommand {
    // Prints nodes related data, like stats,fragments etc.
    Show(show::Show),
    /// Sends Explorer queries
    Explorer(explorer::Explorer),
    /// Exit interactive mode
    Exit,
    /// Prints wallets, nodes which can be used. Draw topology
    Describe(describe::Describe),
    /// send fragments
    Send(send::Send),
}
