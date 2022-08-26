use std::fs;
use std::io::Write;
use tokio::sync::mpsc::{self, Sender};

pub struct FileManager {
  sender: Sender<FileManagerMessage>,
}

#[derive(Debug)]
pub enum FileManagerMessage {
  Append { path: String, text: String },
  // Create // TODO
}

impl FileManager {
  pub fn new() -> Self {
    let (tx, rx) = mpsc::channel::<FileManagerMessage>(32);

    tokio::spawn(async move {
      let mut rx = rx;

      while let Some(message) = rx.recv().await {
        use FileManagerMessage::*;

        match message {
          Append { path, text } => {
            let mut file = fs::File::options().append(true).open(path).unwrap();

            file.write_all(text.as_bytes()).unwrap();
          }
        };
      }
    });

    FileManager { sender: tx }
  }

  pub fn get_sender(&self) -> Sender<FileManagerMessage> {
    self.sender.clone()
  }
}
