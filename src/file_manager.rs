use std::io::Write;
use std::{fs, io::BufWriter};
use tokio::sync::mpsc::{self, Sender};
use twitch_irc::message::PrivmsgMessage;

use crate::{CHANNELS, LOGS};

pub struct FileManager {
  sender: Sender<FileManagerMessage>,
}

#[derive(Debug)]
pub enum FileManagerMessage {
  Append { message: Box<PrivmsgMessage> },
  AddChannel { sender: String },
}

impl FileManager {
  pub fn new() -> Self {
    let (tx, rx) = mpsc::channel::<FileManagerMessage>(32);

    tokio::spawn(async move {
      let mut rx = rx;
      let mut logs = BufWriter::new(fs::File::options().append(true).open(LOGS).unwrap());
      let mut channels = fs::File::options().append(true).open(CHANNELS).unwrap();

      while let Some(message) = rx.recv().await {
        use FileManagerMessage::*;

        match message {
          Append { message } => {
            let text = serde_json::to_string(&message).unwrap();
            logs.write_all((text + "\n").as_bytes()).unwrap();
          }
          AddChannel { sender: channel } => {
            channels.write_all((channel + "\n").as_bytes()).unwrap();
          }
        };
      }

      logs.flush().unwrap();
    });

    FileManager { sender: tx }
  }

  pub fn get_sender(&self) -> Sender<FileManagerMessage> {
    self.sender.clone()
  }
}
