use std::{
    fs::{self, create_dir_all},
    path::PathBuf,
    str::FromStr,
    sync::LazyLock,
};

use clap::ArgMatches;
use color_eyre::Result;
use directories::ProjectDirs;
use kdl::{KdlDocument, KdlEntry, KdlNode};
use notifier_common::{PuppyInsertionMode, PuppyModeConfig};

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
        Some(("puppyfier", matches)) => puppyfier(matches),
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

fn puppyfier(matches: &ArgMatches) {
    match matches.subcommand() {
        Some(("list", _)) => puppyfier_list(),
        Some(("set", matches)) => puppyfier_set(matches),
        Some(("enable", matches)) => puppyfier_enable(matches, true),
        Some(("disable", matches)) => puppyfier_enable(matches, false),
        Some(("view", matches)) => puppyfier_view(matches),
        _ => unreachable!(),
    }
}

fn puppyfier_list() {
    let mut config = get_config_or_default();
    for alias in config_get_puppy_config_aliases(&mut config) {
        println!("{}", alias);
    }
}

fn puppyfier_set(matches: &ArgMatches) {
    let alias: &String = matches.get_one("alias").unwrap();
    let mut config = get_config_or_default();
    let mut puppy_config = config_get_puppy_config(&mut config, alias);

    match matches.subcommand() {
        Some(("probability", matches)) => {
            let value: &String = matches.get_one("value").unwrap();
            config_set_prop(&mut puppy_config, "probability", value.to_owned());
        }
        Some(("words", matches)) => {
            let values = matches
                .get_many::<String>("values")
                .unwrap_or_default()
                .map(|v| v.to_owned())
                .collect::<Vec<_>>();
            config_set_prop_many(&mut puppy_config, "words", values);
        }
        _ => unreachable!(),
    };

    write_config(&config).unwrap();
}

fn puppyfier_enable(matches: &ArgMatches, enabled: bool) {
    let alias: &String = matches.get_one("alias").unwrap();
    let target: &String = matches.get_one("target").unwrap();

    let mut config = get_config_or_default();
    let puppy_config = config_get_puppy_config(&mut config, alias);
    if puppy_config.nodes().is_empty() {
        eprintln!("No valid configuration found for alias \"{}\"!", alias);
        return;
    }
    let mut puppy_mode_request = config_map_puppy_mode_request(puppy_config);
    puppy_mode_request.enabled = enabled;

    let base_url = config_get_prop(&config, "server_url").unwrap_or_default();
    let client = NotifierClient::new(&base_url);
    let clients = refresh_and_get_clients(&client);
    let target = match clients.into_iter().find(|c| c.contains(target)) {
        Some(t) => t,
        None => {
            eprintln!("No client found that matches \"{}\"!", target);
            return;
        }
    };

    client.set_puppy_mode(&target, puppy_mode_request).unwrap();
}

fn puppyfier_view(matches: &ArgMatches) {
    let alias: &String = matches.get_one("alias").unwrap();
    let mut config = get_config_or_default();
    let puppy_config = config_get_puppy_config(&mut config, alias);

    let probability = config_get_prop(&puppy_config, "probability").unwrap_or_default();
    let words = config_get_prop_many(&puppy_config, "words");

    println!("{}:", alias);
    println!("  - probability: {}", probability);
    println!("  - words: {}", words.join(", "));
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

fn config_set_prop_many(config: &mut KdlDocument, prop_name: &str, values: Vec<String>) {
    match config.get_mut(prop_name) {
        Some(prop) => {
            let entries = prop.entries_mut();
            entries.clear();
            for value in values {
                entries.push(KdlEntry::new(value));
            }
        }
        None => {
            let mut node = KdlNode::new(prop_name);
            for value in values {
                node.entries_mut().push(KdlEntry::new(value));
            }
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

fn config_get_prop_many(config: &KdlDocument, prop_name: &str) -> Vec<String> {
    match config.get(prop_name) {
        Some(prop) => prop
            .entries()
            .iter()
            .map(|e| e.value().as_string().unwrap_or_default().to_owned())
            .collect(),
        None => vec![],
    }
}

fn config_get_puppy_config<'a>(config: &'a mut KdlDocument, alias: &str) -> &'a mut KdlDocument {
    if let None = config.get("puppy_config") {
        let node = KdlNode::new("puppy_config");
        config.nodes_mut().push(node);
    }
    let section = config.get_mut("puppy_config").unwrap();

    if let None = section.children() {
        section.set_children(KdlDocument::new());
    }
    let children = section.children_mut().as_mut().unwrap();

    if let None = children.get(alias) {
        let node = KdlNode::new(alias);
        children.nodes_mut().push(node);
    }
    let alias_node = children.get_mut(alias).unwrap();

    if let None = alias_node.children() {
        alias_node.set_children(KdlDocument::new());
    }
    alias_node.children_mut().as_mut().unwrap()
}

fn config_get_puppy_config_aliases(config: &mut KdlDocument) -> Vec<String> {
    if let None = config.get("puppy_config") {
        let node = KdlNode::new("puppy_config");
        config.nodes_mut().push(node);
    }
    let section = config.get_mut("puppy_config").unwrap();

    if let None = section.children() {
        section.set_children(KdlDocument::new());
    }
    let children = section.children().unwrap();

    children
        .nodes()
        .iter()
        .map(|n| n.name().value().to_owned())
        .collect()
}

fn config_map_puppy_mode_request(puppy_config: &KdlDocument) -> PuppyModeConfig {
    let percentage = config_get_prop(puppy_config, "probability").unwrap_or(String::new());
    let words = config_get_prop_many(puppy_config, "words");
    let percentage = percentage.parse().unwrap_or(0);

    PuppyModeConfig {
        enabled: false,
        mode: notifier_common::PuppyMode::Insertion(PuppyInsertionMode {
            probability: percentage,
            puppy_words: words,
        }),
    }
}
