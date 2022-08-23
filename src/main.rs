use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use twitch_irc::login::StaticLoginCredentials;
use twitch_irc::{message, TwitchIRCClient};
use twitch_irc::{ClientConfig, SecureTCPTransport};

#[tokio::main]
pub async fn main() {
  let channels: Vec<String> = fs::read_dir("logs")
    .unwrap()
    .map(|p| p.unwrap().file_name().to_str().unwrap().to_owned())
    .collect();

  let mut files = {
    let mut h = HashMap::new();
    for channel in &channels {
      h.insert(
        channel.clone(),
        fs::File::options()
          .write(true)
          .create(true)
          .append(true)
          .open(format!("logs/{channel}"))
          .unwrap(),
      );
    }
    h
  };

  let config = ClientConfig::default();
  let (mut incoming_messages, client) =
    TwitchIRCClient::<SecureTCPTransport, StaticLoginCredentials>::new(config);

  for channel in &channels {
    client
      .join(channel.clone())
      .expect(&format!("Couldn't join '{channel}'"));

    println!("Successfully added '{}'", channel);
  }

  let join_handle = tokio::spawn(async move {
    while let Some(message) = incoming_messages.recv().await {
      if let message::ServerMessage::Privmsg(msg) = message {
        if msg.channel_login == "luj8n" && msg.sender.login == "luj8n" {
          let words: Vec<_> = msg
            .message_text
            .split_whitespace()
            .map(str::to_string)
            .collect();

          if words.get(0) == Some(&"!chattory".to_string()) {
            if words.get(1) == Some(&"add".to_string()) {
              if let Some(new_channel) = words.get(2) {
                if files.contains_key(new_channel) {
                  println!("'{}' is already added", new_channel);
                } else {
                  match client.join(new_channel.clone()) {
                    Ok(_) => {
                      files.insert(
                        new_channel.clone(),
                        fs::File::options()
                          .write(true)
                          .create(true)
                          .append(true)
                          .open(format!("logs/{new_channel}"))
                          .unwrap(),
                      );

                      println!("Successfully added '{}'", new_channel);
                    }
                    Err(_) => {
                      println!("Couldn't add '{}'", new_channel);
                    }
                  };
                }
              }
            }
          }
        }

        let row_text = serde_json::to_string(&msg).unwrap();

        let mut file = &files[&msg.channel_login];

        file.write_all((row_text + "\n").as_bytes()).unwrap();
      }
    }
  });

  join_handle.await.unwrap();
}
