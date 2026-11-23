use std::{
    fs::{self, create_dir_all},
    path::PathBuf,
    str::FromStr,
    sync::LazyLock,
};

use clap::ArgMatches;
use color_eyre::Result;
use directories::ProjectDirs;
use kdl::{KdlDocument, KdlEntry, KdlNode, KdlValue};
use serde::{Deserialize, Serialize};

use crate::client::NotifierClient;

mod cli;
mod client;

static PROJECT_DIRS: LazyLock<ProjectDirs> =
    LazyLock::new(|| ProjectDirs::from("com", "shoudev", "nhc").unwrap());

fn main() {
    let command = cli::main_command();
    let matches = command.get_matches();

    match matches.subcommand() {
        Some(("config", matches)) => config(matches),
        Some(("clients", matches)) => clients(matches),
        Some(("send", matches)) => send(matches),
        _ => unreachable!(),
    };
}

fn config(matches: &ArgMatches) {
    create_dir_all(PROJECT_DIRS.config_dir()).unwrap();

    let mut config = get_config_or_default();

    match matches.subcommand() {
        Some(("set", matches)) => {
            let (setting, matches) = matches.subcommand().unwrap();
            let value: &String = matches.get_one("value").unwrap();
            config_set_prop(&mut config, setting, value.to_owned());
        }
        None => {
            let server_url: String = dialoguer::Input::new()
                .with_prompt("Server URL")
                .allow_empty(false)
                .interact_text()
                .unwrap();

            let alias: String = dialoguer::Input::new()
                .with_prompt("Alias")
                .allow_empty(false)
                .interact_text()
                .unwrap();

            config_set_prop(&mut config, "server_url", server_url);
            config_set_prop(&mut config, "alias", alias);
        }
        _ => unreachable!(),
    }

    match write_config(&config) {
        Ok(_) => {}
        Err(_) => eprintln!("Could not write config!"),
    };
}

fn clients(_matches: &ArgMatches) {
    let config = match get_config() {
        Some(c) => c,
        None => {
            eprintln!("No config found!");
            return;
        }
    };
    let server_url = config_get_prop(&config, "server_url").unwrap_or_default();

    let client = NotifierClient::new(&server_url);

    let clients = refresh_and_get_clients(&client);

    for client in clients {
        println!("{}", client);
    }
}

fn send(matches: &ArgMatches) {
    let recipient: &String = matches.get_one("recipient").unwrap();
    let message: &String = matches.get_one("message").unwrap();

    let config = match get_config() {
        Some(c) => c,
        None => {
            eprintln!("No config found!");
            return;
        }
    };

    let server_url = config_get_prop(&config, "server_url").unwrap_or_default();
    let alias = config_get_prop(&config, "alias").unwrap_or_default();

    let client = NotifierClient::new(&server_url);

    let clients = refresh_and_get_clients(&client);

    let target = match clients.into_iter().find(|c| c.contains(recipient)) {
        Some(t) => t,
        None => {
            eprintln!("No client found that matches \"{}\"!", recipient);
            return;
        }
    };

    client.send(&target, &alias, message.as_str()).unwrap();
}

fn refresh_and_get_clients(client: &NotifierClient) -> Vec<String> {
    client.refresh_clients().unwrap();
    client.refresh_clients().unwrap();
    client.get_clients().unwrap()
}

fn get_config_path() -> PathBuf {
    let mut config_path = PROJECT_DIRS.config_dir().to_path_buf();
    config_path.push("config.kdl");
    config_path
}

fn get_config() -> Option<KdlDocument> {
    let contents = match fs::read_to_string(get_config_path()) {
        Ok(c) => c,
        Err(_) => return None,
    };

    match KdlDocument::from_str(&contents) {
        Ok(config) => Some(config),
        Err(_) => None,
    }
}

fn get_config_or_default() -> KdlDocument {
    get_config().unwrap_or_default()
}

fn write_config(config: &KdlDocument) -> Result<()> {
    let serialized = config.to_string();
    fs::write(get_config_path(), serialized)?;
    Ok(())
}

fn config_set_prop(config: &mut KdlDocument, prop_name: &str, value: String) {
    match config.get_mut(prop_name) {
        Some(prop) => {
            let entries = prop.entries_mut();
            entries.clear();
            entries.push(KdlEntry::new(value));
        }
        None => {
            let mut node = KdlNode::new(prop_name);
            node.entries_mut().push(KdlEntry::new(value));
            config.nodes_mut().push(node);
        }
    }
}

fn config_get_prop(config: &KdlDocument, prop_name: &str) -> Option<String> {
    match config.get(prop_name) {
        Some(prop) => prop
            .entry(0)
            .map(|e| e.value().as_string().map(|s| s.to_owned()))
            .flatten(),
        None => return None,
    }
}
