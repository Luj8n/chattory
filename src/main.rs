use rayon::prelude::ParallelIterator;
use rayon::str::ParallelString;
use std::collections::HashSet;
use std::fs;
use std::io::Read;
use twitch_irc::login::StaticLoginCredentials;
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
      .create(true)
      .write(true)
      .truncate(false)
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
    client
      .join(channel.clone())
      .unwrap_or_else(|_| panic!("! {channel}"));

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

        if msg.badges.iter().any(|b| b.name == "partner") && !channels.contains(&msg.sender.login) {
          match client.join(msg.sender.login.clone()) {
            Ok(_) => {
              channels.insert(msg.sender.login.clone());

              file_sender
                .send(file_manager::FileManagerMessage::AddChannel {
                  sender: msg.sender.login.clone(),
                })
                .await
                .unwrap();

              println!(
                "+ {} <- {}, {}",
                msg.sender.login,
                msg.channel_login,
                channels.len()
              );
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
