use tokio::sync::{mpsc, oneshot};

use crate::{Bridge, Computer, Hardware};

pub struct ComputerActor {}

#[derive(Clone)]
pub struct ComputerActorHandle {
    tx: mpsc::UnboundedSender<ComputerActorMessage>,
}

impl ComputerActorHandle {
    pub async fn update(&self) -> anyhow::Result<()> {
        let (tx, rx) = oneshot::channel();
        self.tx.send(ComputerActorMessage::Update { tx })?;
        rx.await.map_err(|err| err.into())
    }

    pub async fn get_hardware(&self) -> anyhow::Result<Vec<Hardware>> {
        let (tx, rx) = oneshot::channel();
        self.tx.send(ComputerActorMessage::GetHardware { tx })?;
        let value = rx.await??;
        Ok(value)
    }
}

pub enum ComputerActorMessage {
    Update {
        tx: oneshot::Sender<()>,
    },
    GetHardware {
        tx: oneshot::Sender<anyhow::Result<Vec<Hardware>>>,
    },
}

impl ComputerActor {
    pub fn create() -> ComputerActorHandle {
        let (tx, mut rx) = mpsc::unbounded_channel();

        std::thread::spawn(move || {
            // Run service
            let bridge = Bridge::init();
            let mut computer = Computer::create(&bridge);

            // Perform initial update
            computer.update();

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
                }
            }
        });

        ComputerActorHandle { tx }
    }
}
