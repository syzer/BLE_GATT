use embassy_futures::select::select;
use embassy_futures::join::join;
use embassy_time::Timer;
use trouble_host::prelude::*;
use heapless::Vec;

use embassy_executor::Spawner;
use embassy_executor::task;
use alloc::boxed::Box;

use defmt::info;
use defmt::warn;
use defmt::debug;

use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};

const MAC_ADDRESS: &str = env!("MAC_ADDRESS");
const GATT_NAME: &str = env!("GATT_NAME");

/// Max number of connections
const CONNECTIONS_MAX: usize = 2;

/// Max number of L2CAP channels.
const L2CAP_CHANNELS_MAX: usize = 3; // Signal + att

/// FD2B4448‑AA0F‑4A15‑A62F‑EB0BE77A0000 (little‑endian encoding)
const SERVICE_UUID: [u8; 16] = [0x00, 0x00, 0x7A, 0xE7, 0x0B, 0xEB, 0x2F, 0xA6,
    0x15, 0x4A, 0x0F, 0xAA, 0x48, 0x44, 0x2B, 0xFD];


// GATT Server definition
#[gatt_server(connections_max = CONNECTIONS_MAX)]
struct Server {
    cow_service: CowService,
}

/// Cow service
#[gatt_service(uuid = "fd2b4448-aa0f-4a15-a62f-eb0be77a0000")]
struct CowService {
    /// Temperature in °C (int8) // Temperature
    #[characteristic(uuid = "00000000-0000-0000-0000-fd2bcccb0001", read, notify, value = 0)]
    temperature: u8,

    /// Outbound ticket payloads (device → phone) // GetTickets
    #[characteristic(uuid = "00000000-0000-0000-0000-fd2bcccb000a", read, notify, value = [0; 20])]
    get_tickets: [u8; 20],

    /// Inbound ticket payloads (phone → device) // PostTickets
    #[characteristic(uuid = "00000000-0000-0000-0000-fd2bcccb000b", read, write, value = [0; 20])]
    post_tickets: [u8; 20],

    /// Login credentials sent from phone // PostLogin
    // #[characteristic(uuid = "00000000-0000-0000-0000-fd2bcccb0006", read, /*write,*/ notify, value = heapless::Vec::<u8, 128>::new())]
    // #[characteristic(uuid = "00000000-0000-0000-0000-fd2bcccb0006", read, /*write,*/ notify, value = [0; 128])]
    #[characteristic(uuid = "00000000-0000-0000-0000-fd2bcccb0006", read, write, notify, value = [0; 128])]
    // post_login: heapless::Vec<u8, 128>,
    post_login: [u8; 128],
}

/// Run the BLE stack.
pub async fn run<C>(controller: C, _spawner: &Spawner, temp_c: &'static Signal<CriticalSectionRawMutex, u8>)
where
    C: Controller + 'static,
{
    let parts = MAC_ADDRESS.split(":");
    let hexes: heapless::Vec<u8, 6> = parts.map(|f| u8::from_str_radix(f, 16).unwrap()).collect();

    let address = Address::random(hexes.into_array().unwrap());
    warn!("MAC address = {:?}", address);

    // Put the HostResources in a Box and leak it so it is &'static mut
    let resources: &'static mut HostResources<
        DefaultPacketPool,
        CONNECTIONS_MAX,
        L2CAP_CHANNELS_MAX,
    > = Box::leak(Box::new(HostResources::new()));

    // Stack builder must also live for 'static because GattConnection
    // holds references into it.
    let stack_builder = trouble_host::new(controller, resources).set_random_address(address);
    let stack_builder: &'static mut _ = Box::leak(Box::new(stack_builder));

    let Host {
        mut peripheral,
        runner,
        ..
    } = stack_builder.build();

    info!("Starting advertising and GATT service, with name: {}", GATT_NAME);
    let server = Server::new_with_config(GapConfig::Peripheral(PeripheralConfig {
        name: GATT_NAME,
        appearance: &appearance::access_control::GENERIC_ACCESS_CONTROL,
    }))
    .unwrap();

    // Leak the server so spawned tasks can hold a 'static reference to it
    let server: &'static Server = Box::leak(Box::new(server));

    #[task(pool_size = CONNECTIONS_MAX)]
    async fn connection_task(
        server: &'static Server<'static>,
        conn: GattConnection<'static, 'static, DefaultPacketPool>,
        temp_c: &'static Signal<CriticalSectionRawMutex, u8>, // TODO name this Signal
    ) {
        select(
            gatt_events_task(server, &conn),
            custom_task(server, &conn, temp_c),
        )
        .await;
    }

    let adv_loop = async {
        loop {
            match advertise("COW GATT", &mut peripheral, server).await {
                Ok(conn) => {
                    _spawner.must_spawn(connection_task(server, conn, &temp_c));
                }
                Err(_e) => {
                    #[cfg(feature = "defmt")]
                    warn!("[adv] advertise error: {:?}", defmt::Debug2Format(&_e));
                }
            }
        }
    };

    // Runner needs &mut self, so wrap it in an async block with a mutable binding.
    let run_task = async move {
        let mut r = runner;
        let _ = r.run().await; // ignore result; runner never returns on success
    };

    join(run_task, adv_loop).await;
}

/// Stream Events until the connection closes.
///
/// This function will handle the GATT events and process them.
/// This is how we interact with read and write requests.
async fn gatt_events_task(
    server: &Server<'static>,
    conn: &GattConnection<'static, 'static, DefaultPacketPool>,
) -> Result<(), Error> {
    let temperature   = server.cow_service.temperature;
    let post_login    = server.cow_service.post_login;
    let post_tickets  = server.cow_service.post_tickets;
    let reason = loop {
        match conn.next().await {
            GattConnectionEvent::Disconnected { reason } => break reason,
            GattConnectionEvent::Gatt { event } => {
                match &event {
                    GattEvent::Read(event) => {
                        if event.handle() == post_login.handle {
                            let value = server.get(&post_login);
                            info!("[gatt] Read Event to Post Login Characteristic: {:?}", value);
                        } else if event.handle() == temperature.handle {
                            let value = server.get(&temperature);
                            info!("[gatt] Read Event to Temperature Characteristic: {:?}", value);
                        } else if event.handle() == post_tickets.handle {
                            let value = server.get(&post_tickets);
                            info!("[gatt] Read Event to PostTickets Characteristic: {:?}", value);
                        }
                        if event.handle() == temperature.handle {
                            let value = server.get(&temperature);
                            info!("[gatt] Read Event to Temperature Characteristic: {:?}", value);
                        }
                    }
                    GattEvent::Write(event) => {
                        // TODO move to separate handler
                        if event.handle() == post_login.handle {
                            let data = event.data();
                            if let Ok(s) = core::str::from_utf8(data) {
                                info!("[gatt] PostLogin: {}", s);
                            } else {
                                info!("[gatt] PostLogin (raw): {:?}", data);
                            }

                            // Copy payload into a heapless Vec (truncate if >128)
                            // let mut vec = Vec::<u8, 128>::new();
                            // let _ = vec.extend_from_slice(data);
                            // let mut ack = [0u8; 128];
                            // ack[0..7].copy_from_slice(b"200 OK");


                            // Send ACK "200" back to the phone
                            // let mut ack = Vec::<u8, 128>::new();
                            // let _ = ack.extend_from_slice(b"200 OK");
                            // let _ = post_login.write(&ack).await;;
                            let mut ack = [0u8; 128];
                            let s = b"200 OK /PostLogin";
                            ack[..s.len()].copy_from_slice(s);


                            // Store the received JSON
                            if let Err(e) = server.set(&post_login, &ack) {
                                warn!("[gatt] failed to store post_login: {:?}", e);
                            }
                            if post_login.notify(conn, &ack).await.is_err() {
                                warn!("[gatt] failed to update post_login");
                            }
                            info!("Response ready: 200 OK");
                    } else if event.handle() == temperature.handle {
                            let raw = event.data();                       // &[u8]
                            info!("[gatt] Write Event to Temperature Characteristic: {:?}", raw);

                            if let Some(&temp) = raw.first() {
                                if let Err(e) = server.set(&temperature, &temp) {
                                    warn!("[gatt] failed to update temperature: {:?}", e);
                                }
                            } else {
                                warn!("[gatt] temperature write was empty");
                            }
                        } else if event.handle() == post_tickets.handle {
                            let data = event.data();
                            info!("[gatt] Write Event to PostTickets Characteristic: {:?}", data);

                            // copy into fixed 20‑byte buffer (truncate if longer)
                            let mut buf = [0u8; 20];
                            let n = core::cmp::min(buf.len(), data.len());
                            buf[..n].copy_from_slice(&data[..n]);

                            if let Err(e) = server.set(&post_tickets, &buf) {
                                warn!("[gatt] failed to update post_tickets: {:?}", e);
                            }
                        }
                        if event.handle() == temperature.handle {
                            info!(
                                "[gatt] Write Event to Temperature Characteristic: {:?}",
                                event.data()
                            );
                        }
                    }
                    _ => {}
                };
                // This step is also performed at drop(), but writing it explicitly is necessary
                // in order to ensure reply is sent.
                match event.accept() {
                    Ok(reply) => reply.send().await,
                    Err(e) => warn!("[gatt] error sending response: {:?}", e),
                };
            }
            _ => {} // ignore other Gatt Connection Events
        }
    };
    info!("[gatt] disconnected: {:?}", reason);
    Ok(())
}

/// Create an advertiser to use to connect to a BLE Central, and wait for it to connect.
async fn advertise<'values, 'server, C: Controller>(
    name: &'values str,
    peripheral: &mut Peripheral<'values, C, DefaultPacketPool>,
    server: &'server Server<'values>,
) -> Result<GattConnection<'values, 'server, DefaultPacketPool>, BleHostError<C::Error>> {
    let mut advertiser_data = [0; 31];

    let len_adv = AdStructure::encode_slice(
        &[
            AdStructure::Flags(LE_GENERAL_DISCOVERABLE | BR_EDR_NOT_SUPPORTED),
            AdStructure::ServiceUuids128(&[SERVICE_UUID]),
        ],
        &mut advertiser_data[..],
    )?;

    let mut scan_rsp_data = [0u8; 31];
    let len_scan = AdStructure::encode_slice(
        &[
            AdStructure::CompleteLocalName(name.as_bytes()),
            AdStructure::ManufacturerSpecificData { company_identifier: 0x0245, payload: &[] },
        ],
        &mut scan_rsp_data[..],
    )?;


    let advertiser = peripheral
        .advertise(
            &Default::default(),
            Advertisement::ConnectableScannableUndirected {
                adv_data: &advertiser_data[..len_adv],
                scan_data: &scan_rsp_data[..len_scan],
            },
        )
        .await?;

    info!("[adv] advertising");
    let conn = advertiser.accept().await?.with_attribute_server(server)?;
    info!("[adv] connection established");
    Ok(conn)
}

/// Example task to use the BLE notifier interface.
/// This task will notify the connected central of a counter value every 2 seconds.
/// It will also read the RSSI value every 2 seconds.
/// and will stop when the connection is closed by the central or an error occurs.
async fn custom_task(
    server: &Server<'static>,
    conn: &GattConnection<'static, 'static, DefaultPacketPool>,
    temp_c: &'static Signal<CriticalSectionRawMutex, u8>,
) {
    let mut tick: i8 = 0;
    let temperature = server.cow_service.temperature;
    let post_login = &server.cow_service.post_login;
    loop {
        tick = tick.wrapping_add(1);
        let c = temp_c.wait().await;           // blocks until new value
        debug!("[custom_task] notifying connection of tick {}", tick);
        debug!("[custom_task] notifying connection of temp {}", c);
        if temperature.notify(conn, &c).await.is_err() {
            warn!("[custom_task] error notifying connection");
            break;
        };

        // let mut ack = Vec::<u8, 128>::new();
        // let _ = ack.extend_from_slice(b"200 OK");
        let mut ack = [0u8; 128];
        let s = b"200 OK /PostLogin";
        ack[..s.len()].copy_from_slice(s);


        // debug!("[custom_task] sending ack: {:?}", ack);
        // if post_login.notify(conn, &ack).await.is_err() {
        //     warn!("[post_login] error notifying connection");
        //     break;
        // };

        Timer::after_secs(2).await;
    }
}
