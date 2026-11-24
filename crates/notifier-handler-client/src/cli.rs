use clap::{Arg, ArgAction, Command};

pub fn main_command() -> Command {
    Command::new("nhc")
        .subcommand_required(true)
        .subcommand(config_command())
        .subcommand(clients_command())
        .subcommand(send_command())
        .subcommand(puppyfier_command())
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

pub fn puppyfier_command() -> Command {
    Command::new("puppyfier")
        .subcommand_required(true)
        .subcommand(Command::new("list"))
        .subcommand(
            Command::new("set")
                .subcommand_required(true)
                .arg(Arg::new("alias").required(true))
                .subcommand(Command::new("probability").arg(Arg::new("value").required(true)))
                .subcommand(
                    Command::new("words")
                        .arg(Arg::new("values").action(ArgAction::Append).required(true)),
                ),
        )
        .subcommand(
            Command::new("enable")
                .arg(Arg::new("target").required(true))
                .arg(Arg::new("alias").required(true)),
        )
        .subcommand(
            Command::new("disable")
                .arg(Arg::new("target").required(true))
                .arg(Arg::new("alias").required(true)),
        )
        .subcommand(Command::new("view").arg(Arg::new("alias").required(true)))
}
