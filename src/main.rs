use std::fs;
use twitch_irc::login::StaticLoginCredentials;
use twitch_irc::{message, TwitchIRCClient};
use twitch_irc::{ClientConfig, SecureTCPTransport};

mod file_manager;

#[tokio::main]
pub async fn main() {
  let mut channels: Vec<String> = fs::read_dir("logs")
    .unwrap()
    .map(|p| p.unwrap().file_name().to_str().unwrap().to_owned())
    .collect();

  let config = ClientConfig::default();
  let (mut incoming_messages, client) =
    TwitchIRCClient::<SecureTCPTransport, StaticLoginCredentials>::new(config);

  for channel in &channels {
    client
      .join(channel.clone())
      .expect(&format!("Couldn't join '{channel}'"));

    println!("+ {}", channel);
  }

  let file_manager = file_manager::FileManager::new();
  let sender = file_manager.get_sender();

  let join_handle = tokio::spawn(async move {
    while let Some(message) = incoming_messages.recv().await {
      if let message::ServerMessage::Privmsg(msg) = message {
        let x = msg.sender.login.clone();
        if !channels.contains(&x) && channels.len() < 1000000 {
          match client.join(x.clone()) {
            Ok(_) => {
              channels.push(x.clone());

              fs::File::create(format!("logs/{}", x)).unwrap();

              println!("+ {} <- {}", x, msg.channel_login);
            }
            Err(_) => {
              println!("! {}", x);
            }
          };
        }

        let row_text = serde_json::to_string(&msg).unwrap() + "\n";

        sender
          .send(file_manager::FileManagerMessage::Append {
            path: format!("logs/{}", msg.channel_login),
            text: row_text,
          })
          .await
          .unwrap();
      }
    }
  });

  println!("Starting...");

  join_handle.await.unwrap();
}
