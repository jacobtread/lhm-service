use lhm_shared::{ComputerOptions, Hardware, HardwareType, Sensor, SensorType};
use tokio::sync::{mpsc, oneshot};

use lhm_sys::{Computer, SharedApi};

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
    pub fn create(bridge: SharedApi) -> ComputerActorHandle {
        let (tx, mut rx) = mpsc::unbounded_channel();

        std::thread::spawn(move || {
            // Run service
            let mut computer = match Computer::create(bridge) {
                Ok(value) => value,
                Err(err) => {
                    eprintln!("failed to start computer service: {err}");
                    return;
                }
            };

            while let Some(msg) = rx.blocking_recv() {
                match msg {
                    ComputerActorMessage::Update { tx } => {
                        computer.update();
                        _ = tx.send(())
                    }
                    ComputerActorMessage::GetHardware { tx } => {
                        fn create_hardware(hardware: lhm_sys::Hardware) -> Hardware {
                            Hardware {
                                name: hardware.get_name(),
                                ty: HardwareType::from(hardware.get_type()),
                                children: hardware
                                    .get_children()
                                    .into_iter()
                                    .map(create_hardware)
                                    .collect(),
                                sensors: hardware
                                    .get_sensors()
                                    .into_iter()
                                    .map(|sensor| Sensor {
                                        name: sensor.name(),
                                        ty: SensorType::from(sensor.get_type()),
                                        value: sensor.value(),
                                    })
                                    .collect(),
                            }
                        }

                        let items = computer
                            .hardware()
                            .into_iter()
                            .map(create_hardware)
                            .collect();

                        _ = tx.send(items);
                    }
                    ComputerActorMessage::SetOptions { options, tx } => {
                        computer.set_options(lhm_sys::ComputerOptions {
                            battery_enabled: options.battery_enabled,
                            controller_enabled: options.controller_enabled,
                            cpu_enabled: options.cpu_enabled,
                            gpu_enabled: options.gpu_enabled,
                            memory_enabled: options.memory_enabled,
                            motherboard_enabled: options.motherboard_enabled,
                            network_enabled: options.network_enabled,
                            psu_enabled: options.psu_enabled,
                            storage_enabled: options.storage_enabled,
                        });
                        _ = tx.send(());
                    }
                }
            }
        });

        ComputerActorHandle { tx }
    }
}
