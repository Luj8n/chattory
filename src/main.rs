use rayon::prelude::IntoParallelRefIterator;
use rayon::prelude::ParallelIterator;
use rayon::prelude::*;
use rayon::str::ParallelString;
use std::collections::HashSet;
use std::fs;
use std::io::{Read, Write};
use twitch_irc::login::StaticLoginCredentials;
use twitch_irc::message::PrivmsgMessage;
use twitch_irc::{message, TwitchIRCClient};
use twitch_irc::{ClientConfig, SecureTCPTransport};

mod file_manager;

pub const CHANNELS: &str = "data/channels";
pub const LOGS: &str = "data/logs";

#[tokio::main]
pub async fn main() {
  let mut channels: HashSet<String> = {
    let mut buf = String::new();
    fs::File::options()
      .read(true)
      .open(CHANNELS)
      .unwrap()
      .read_to_string(&mut buf)
      .unwrap();

    buf.par_lines().map(|x| x.to_string()).collect()
  };

  let config = ClientConfig::default();
  let (mut incoming_messages, client) =
    TwitchIRCClient::<SecureTCPTransport, StaticLoginCredentials>::new(config);

  for channel in &channels {
    client.join(channel.clone()).expect(&format!("! {channel}"));

    println!("+ {}", channel);
  }

  let file_manager = file_manager::FileManager::new();
  let file_sender = file_manager.get_sender();

  let join_handle = tokio::spawn(async move {
    while let Some(message) = incoming_messages.recv().await {
      if let message::ServerMessage::Privmsg(msg) = message {
        if msg.sender.login == "luj8n" {
          println!("!!! Got my own message");
        }

        if !channels.contains(&msg.sender.login) && channels.len() < 1000000 {
          match client.join(msg.sender.login.clone()) {
            Ok(_) => {
              channels.insert(msg.sender.login.clone());

              file_sender
                .send(file_manager::FileManagerMessage::AddChannel {
                  sender: msg.sender.login.clone(),
                })
                .await
                .unwrap();

              println!("+ {} <- {}", msg.sender.login, msg.channel_login);
            }
            Err(_) => {
              println!("! {}", msg.sender.login);
            }
          };
        }

        file_sender
          .send(file_manager::FileManagerMessage::Append {
            message: Box::new(msg),
          })
          .await
          .unwrap();
      }
    }
  });

  println!("Starting...");

  // TODO: add a grace shutdown method - flush buffers
  join_handle.await.unwrap();
}

pub fn old_to_new() {
  let channels: Vec<String> = fs::read_dir("../logs")
    .unwrap()
    .map(|p| p.unwrap().file_name().to_str().unwrap().to_owned())
    .collect();

  let mut files: Vec<String> = vec![];

  for channel in &channels {
    let mut file = fs::File::options()
      .read(true)
      .open(format!("../logs/{channel}"))
      .unwrap();
    let mut buf = String::new();

    file.read_to_string(&mut buf).unwrap();

    files.push(buf);
  }

  let mut rows: Vec<&str> = files
    .par_iter()
    .flat_map(|f| f.par_lines().collect::<Vec<&str>>())
    .collect();

  rows.par_sort_by_cached_key(|x| {
    serde_json::from_str::<PrivmsgMessage>(x)
      .unwrap()
      .server_timestamp
  });

  let mut new_file = fs::File::create("data/logs").unwrap();
  let mut channel_file = fs::File::create("data/channels").unwrap();

  let text = rows.join("\n") + "\n";
  let channels = channels.join("\n") + "\n";

  new_file.write_all(text.as_bytes()).unwrap();
  channel_file.write_all(channels.as_bytes()).unwrap();
}

// fn main() {
//   old_to_new();
// }
