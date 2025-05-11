use tokio::sync::{mpsc, oneshot};

use crate::{Bridge, Computer, Hardware};

pub struct ComputerActor {}

#[derive(Clone)]
pub struct ComputerActorHandle {
    tx: mpsc::UnboundedSender<ComputerActorMessage>,
}

impl ComputerActorHandle {
    pub async fn update(&self) {
        let (tx, rx) = oneshot::channel();
        self.tx.send(ComputerActorMessage::Update { tx }).unwrap();
        rx.await.unwrap();
    }

    pub async fn get_hardware(&self) -> Vec<Hardware> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(ComputerActorMessage::GetHardware { tx })
            .unwrap();
        rx.await.unwrap()
    }
}

pub enum ComputerActorMessage {
    Update { tx: oneshot::Sender<()> },
    GetHardware { tx: oneshot::Sender<Vec<Hardware>> },
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
