pub mod discord;
pub mod twitch;

/// Scuffed event system
pub mod bridge {
    use std::{collections::HashMap, sync::Arc};

    use tokio::sync::mpsc;

    use crate::config::ChannelId;

    #[derive(Default)]
    pub struct BridgeBuilder {
        bridge: Bridge,
        platforms: HashMap<PlatformKind, Platform>,
        senders: HashMap<PlatformKind, mpsc::Sender<RawEvent>>,
    }

    impl BridgeBuilder {
        pub fn bridge(
            &mut self,
            from: ChannelId,
            to: ChannelId,
            symmetric: bool,
            translate_from: Option<HashMap<String, String>>,
            translate_to: Option<HashMap<String, String>>,
        ) -> &mut Self {
            if symmetric {
                self.add_bridge(to.clone(), from.clone(), translate_to);
            }

            self.add_bridge(from, to, translate_from);

            self
        }

        fn add_bridge(
            &mut self,
            a: ChannelId,
            b: ChannelId,
            translate: Option<HashMap<String, String>>,
        ) {
            let kind = a.kind();
            self.plaftorm(kind.clone());

            if let Some(t) = translate {
                self.bridge.translate.insert(a.clone(), t);
            };

            if let Some(channels) = self.bridge.channels.get_mut(&a) {
                channels.push((b, kind));
            } else {
                self.bridge.channels.insert(a, vec![(b, kind)]);
            }
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
            for platform in &mut self.platforms.values_mut() {
                platform.sender.senders.clone_from(&self.senders);
            }
            (Arc::new(self.bridge), self.platforms)
        }
    }

    #[derive(Default)]
    pub struct Bridge {
        channels: HashMap<ChannelId, Vec<(ChannelId, PlatformKind)>>,
        translate: HashMap<ChannelId, HashMap<String, String>>,
    }

    impl Bridge {
        pub fn get(&self, id: &ChannelId) -> Option<&Vec<(ChannelId, PlatformKind)>> {
            self.channels.get(id)
        }

        pub fn translate(&self, id: &ChannelId, mut msg: String) -> String {
            if let Some(t) = self.translate.get(id) {
                msg = msg
                    .split_whitespace()
                    .map(|w| t.get(w).map(String::as_str).unwrap_or(w).to_owned() + " ")
                    .collect::<String>()
                    .trim_end()
                    .into();
            }
            msg
        }
    }

    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
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

        pub async fn send(
            &mut self,
            from: ChannelId,
            channels: &Vec<(ChannelId, PlatformKind)>,
            ev: Event,
        ) {
            for (to, _) in channels {
                if let Some(sender) = self.senders.get(&to.kind()) {
                    sender
                        .send(RawEvent {
                            from: from.clone(),
                            to: to.clone(),
                            ev: ev.clone(),
                        })
                        .await
                        .unwrap();
                }
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
