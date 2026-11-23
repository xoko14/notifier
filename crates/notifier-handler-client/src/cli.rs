use clap::{Arg, Command};

pub fn main_command() -> Command {
    Command::new("nhc")
        .subcommand_required(true)
        .subcommand(config_command())
        .subcommand(clients_command())
        .subcommand(send_command())
}

pub fn config_command() -> Command {
    Command::new("config").subcommand(Command::new("set").subcommands(vec![
        Command::new("server").arg(Arg::new("value").required(true)),
        Command::new("alias").arg(Arg::new("value").required(true)),
    ]))
}

pub fn clients_command() -> Command {
    Command::new("clients")
}

pub fn send_command() -> Command {
    Command::new("send")
        .arg(Arg::new("recipient").required(true))
        .arg(Arg::new("message").required(true))
}
