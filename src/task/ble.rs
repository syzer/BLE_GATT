use embassy_futures::select::select;
use embassy_time::Timer;
use heapless::Vec;
use core::cell::RefCell;

use embassy_executor::Spawner;
use embassy_executor::task;
use alloc::boxed::Box;

use defmt::info;
use defmt::warn;
use defmt::debug;

use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};

use bleps::{
    ad_structure::{
        AdStructure,
        BR_EDR_NOT_SUPPORTED,
        LE_GENERAL_DISCOVERABLE,
        create_advertising_data,
    },
    async_attribute_server::AttributeServer,
    asynch::Ble,
    attribute_server::NotificationData,
    gatt,
};

const MAC_ADDRESS: &str = env!("MAC_ADDRESS");

// Service UUID: FD2B4448-AA0F-4A15-A62F-EB0BE77A0000
// PostLogin UUID: 00000000-0000-0000-0000-fd2bcccb0006

// We'll define our GATT service using the bleps gatt! macro
// This will be used in the run function

// Define static temperature value
static mut TEMPERATURE: i8 = 0;

/// Run the BLE stack.
pub async fn run(controller: esp_wifi::ble::controller::BleConnector<'_>, _spawner: &Spawner, TEMP_C: &'static Signal<CriticalSectionRawMutex, i8>)
{
    let parts = MAC_ADDRESS.split(":");
    let hexes: heapless::Vec<u8, 6> = parts.map(|f| u8::from_str_radix(f, 16).unwrap()).collect();

    warn!("MAC address = {:?}", hexes);

    // Create a timestamp function
    let now = || esp_hal::time::Instant::now().duration_since_epoch().as_millis();

    // Initialize BLE
    let mut ble = Ble::new(controller, now);
    info!("BLE connector created");

    // Temperature value reference
    let temp_ref = RefCell::new(0i8);
    let temp_ref = &temp_ref;

    // Initialize BLE
    info!("Initializing BLE...");
    match ble.init().await {
        Ok(_) => info!("BLE initialized successfully"),
        Err(e) => {
            warn!("BLE initialization error: {:?}", e);
            return;
        }
    }

    // Set advertising parameters
    info!("Setting advertising parameters...");
    match ble.cmd_set_le_advertising_parameters().await {
        Ok(_) => info!("Advertising parameters set successfully"),
        Err(e) => {
            warn!("Failed to set advertising parameters: {:?}", e);
            return;
        }
    }

    // Set advertising data
    info!("Setting advertising data...");
    match ble.cmd_set_le_advertising_data(
        create_advertising_data(&[
            AdStructure::Flags(LE_GENERAL_DISCOVERABLE | BR_EDR_NOT_SUPPORTED),
            AdStructure::CompleteLocalName("COW2GATT"),
        ]).unwrap()
    ).await {
        Ok(_) => info!("Advertising data set successfully"),
        Err(e) => {
            warn!("Failed to set advertising data: {:?}", e);
            return;
        }
    }

    // Enable advertising
    info!("Enabling advertising...");
    match ble.cmd_set_le_advertise_enable(true).await {
        Ok(_) => info!("Advertising enabled successfully"),
        Err(e) => {
            warn!("Failed to enable advertising: {:?}", e);
            return;
        }
    }

    info!("Started advertising");

    // Define read and write handlers for characteristics
    let mut temperature_read = |_offset: usize, data: &mut [u8]| {
        data[0] = unsafe { TEMPERATURE } as u8;
        1
    };

    let mut post_login_read = |_offset: usize, data: &mut [u8]| {
        data[..5].copy_from_slice(b"Login");
        5
    };

    let mut post_login_write = |offset: usize, data: &[u8]| {
        if let Ok(txt) = core::str::from_utf8(data) {
            info!("[gatt] PostLogin JSON: {}", txt);
            // TODO: parse JSON and push to queue if needed
        } else {
            warn!("[gatt] PostLogin not UTF‑8: {:?}", data);
        }
    };

    // Define GATT service and characteristics
    gatt!([service {
        uuid: "FD2B4448-AA0F-4A15-A62F-EB0BE77A0000",
        characteristics: [
            characteristic {
                name: "temperature",
                uuid: "00000000-0000-0000-0000-fd2bcccb0001",
                read: temperature_read,
                notify: true,
            },
            characteristic {
                name: "post_login",
                uuid: "00000000-0000-0000-0000-fd2bcccb0006",
                read: post_login_read,
                write: post_login_write,
                notify: true,
            },
        ],
    },]);

    // Create attribute server
    let mut rng = bleps::no_rng::NoRng;
    let mut srv = AttributeServer::new(&mut ble, &mut gatt_attributes, &mut rng);

    // Create a notifier for temperature updates
    let mut notifier = || {
        async {
            // Wait for temperature update
            let temp = TEMP_C.wait().await;

            // Update the static temperature value
            unsafe { TEMPERATURE = temp; }

            // Create notification data
            let mut data = [0u8; 1];
            data[0] = temp as u8;

            info!("Notifying temperature: {}", temp);
            // Use the handle for the temperature characteristic
            // The handle is 3 for the first characteristic (service handle is 1, service value is 2, characteristic is 3)
            NotificationData::new(3, &data)
        }
    };

    // Run the attribute server
    info!("Running BLE attribute server...");
    match srv.run(&mut notifier).await {
        Ok(_) => info!("BLE attribute server completed"),
        Err(_) => warn!("BLE attribute server error"),
    }
}
