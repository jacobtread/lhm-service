use lhm_shared::{ComputerOptions, Hardware};
use tokio::sync::{mpsc, oneshot};

use lhm_sys::{Bridge, Computer};

pub struct ComputerActor {}

#[derive(Clone)]
pub struct ComputerActorHandle {
    tx: mpsc::UnboundedSender<ComputerActorMessage>,
}

impl ComputerActorHandle {
    pub async fn set_options(&self, options: ComputerOptions) -> anyhow::Result<()> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(ComputerActorMessage::SetOptions { options, tx })?;
        rx.await.map_err(|err| err.into())
    }

    pub async fn update(&self) -> anyhow::Result<()> {
        let (tx, rx) = oneshot::channel();
        self.tx.send(ComputerActorMessage::Update { tx })?;
        rx.await.map_err(|err| err.into())
    }

    pub async fn get_hardware(&self) -> anyhow::Result<Vec<Hardware>> {
        let (tx, rx) = oneshot::channel();
        self.tx.send(ComputerActorMessage::GetHardware { tx })?;
        let value = rx.await?;
        Ok(value)
    }
}

pub enum ComputerActorMessage {
    SetOptions {
        options: ComputerOptions,
        tx: oneshot::Sender<()>,
    },

    Update {
        tx: oneshot::Sender<()>,
    },
    GetHardware {
        tx: oneshot::Sender<Vec<Hardware>>,
    },
}

impl ComputerActor {
    pub fn create(bridge: Bridge, options: ComputerOptions) -> ComputerActorHandle {
        let (tx, mut rx) = mpsc::unbounded_channel();

        std::thread::spawn(move || {
            // Run service
            let mut computer = Computer::create(&bridge, options);

            while let Some(msg) = rx.blocking_recv() {
                match msg {
                    ComputerActorMessage::Update { tx } => {
                        computer.update();
                        _ = tx.send(())
                    }
                    ComputerActorMessage::GetHardware { tx } => {
                        let hardware = computer.get_hardware();
                        _ = tx.send(hardware);
                    }
                    ComputerActorMessage::SetOptions { options, tx } => {
                        computer.update_options(options);
                        _ = tx.send(());
                    }
                }
            }
        });

        ComputerActorHandle { tx }
    }
}
