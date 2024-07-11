pub mod discord;
pub mod twitch;

/// Scuffed event system
pub mod bridge {
    use std::collections::HashMap;

    use tokio::sync::mpsc;

    use crate::config::ChannelId;

    pub struct Bridge {
        receiver: Receiver,
        raw_sender: mpsc::Sender<(ChannelId, Event)>,
        listeners: HashMap<ChannelId, Vec<Sender>>,
    }

    impl Bridge {
        pub fn new() -> Self {
            let (sender, receiver) = mpsc::channel(100);
            let receiver = Receiver { receiver };
            Self {
                receiver,
                raw_sender: sender,
                listeners: HashMap::new(),
            }
        }

        pub async fn recv(&mut self) -> Option<(ChannelId, Event)> {
            self.receiver.recv().await
        }

        pub fn sender(&self) -> Sender {
            Sender {
                sender: self.raw_sender.clone(),
            }
        }

        pub fn listen(&mut self, id: ChannelId, sender: Sender) {
            if let Some(senders) = self.listeners.get_mut(&id) {
                senders.push(sender);
            } else {
                self.listeners.insert(id, vec![sender]);
            }
        }

        pub fn get_listeners(&self, id: &ChannelId) -> Option<&Vec<Sender>> {
            self.listeners.get(id)
        }

        pub async fn send(&mut self, ev: Event) {
            for (id, senders) in &mut self.listeners {
                for sender in senders.iter_mut() {
                    sender.send(id.clone(), ev.clone()).await;
                }
            }
        }

        pub fn channel_ids(&self) -> Vec<&ChannelId> {
            self.listeners.keys().collect()
        }
    }

    #[derive(Clone)]
    pub struct Sender {
        sender: mpsc::Sender<(ChannelId, Event)>,
    }

    impl Sender {
        pub async fn send(&mut self, id: ChannelId, ev: Event) {
            self.sender.send((id, ev)).await.unwrap();
        }
    }

    pub struct Receiver {
        receiver: mpsc::Receiver<(ChannelId, Event)>,
    }

    impl Receiver {
        pub async fn recv(&mut self) -> Option<(ChannelId, Event)> {
            self.receiver.recv().await
        }
    }

    #[derive(Clone)]
    pub enum Event {
        SendMessage { name: String, text: String },
    }
}
