pub mod discord;
pub mod twitch;

/// Scuffed event system
pub mod bridge {
    use std::{collections::HashMap, sync::Arc};

    use tokio::sync::mpsc;

    use crate::config::ChannelId;

    pub struct BridgeBuilder {
        bridge: Bridge,
        platforms: HashMap<PlatformKind, Platform>,
        senders: HashMap<PlatformKind, mpsc::Sender<RawEvent>>,
    }

    impl BridgeBuilder {
        pub fn new() -> Self {
            Self {
                bridge: Bridge::new(),
                platforms: HashMap::new(),
                senders: HashMap::new(),
            }
        }

        pub fn bridge(&mut self, from: ChannelId, to: ChannelId) -> &mut Self {
            let kind = to.kind();
            self.plaftorm(kind.clone());

            self.bridge.insert(from, (to, kind));

            self
        }

        fn plaftorm(&mut self, kind: PlatformKind) {
            if let std::collections::hash_map::Entry::Vacant(e) = self.platforms.entry(kind.clone())
            {
                let (platform, s) = Platform::new();
                self.senders.insert(kind, s.clone());
                e.insert(platform);
            }
        }

        pub fn build(mut self) -> (Arc<Bridge>, HashMap<PlatformKind, Platform>) {
            for (kind, platform) in &mut self.platforms {
                let mut senders = self.senders.clone();
                senders.remove(kind); // remove itself
                platform.sender.senders = senders;
            }
            (Arc::new(self.bridge), self.platforms)
        }
    }

    pub type Bridge = HashMap<ChannelId, (ChannelId, PlatformKind)>;

    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
    pub enum PlatformKind {
        Twitch,
        Discord,
    }

    pub struct Platform {
        pub receiver: Receiver,
        pub sender: Sender,
    }

    impl Platform {
        fn new() -> (Self, mpsc::Sender<RawEvent>) {
            let (s, r) = mpsc::channel(100);
            (
                Self {
                    receiver: Receiver::new(r),
                    sender: Sender::new(),
                },
                s,
            )
        }
    }

    pub struct Receiver {
        pub receiver: mpsc::Receiver<RawEvent>,
    }

    impl Receiver {
        pub fn new(r: mpsc::Receiver<RawEvent>) -> Self {
            Self { receiver: r }
        }

        pub async fn recv(&mut self) -> Option<RawEvent> {
            self.receiver.recv().await
        }
    }

    pub struct Sender {
        senders: HashMap<PlatformKind, mpsc::Sender<RawEvent>>,
    }

    impl Sender {
        pub fn new() -> Self {
            Self {
                senders: HashMap::new(),
            }
        }

        pub async fn send(&mut self, ev: RawEvent) {
            if let Some(sender) = self.senders.get(&ev.to.kind()) {
                sender.send(ev).await.unwrap();
            }
        }
    }

    #[derive(Clone)]
    pub struct RawEvent {
        pub from: ChannelId,
        pub to: ChannelId,
        pub ev: Event,
    }

    #[derive(Clone)]
    pub enum Event {
        SendMessage { name: String, text: String },
    }
}
