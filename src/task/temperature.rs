use defmt::debug;
use embassy_time::Timer;
use esp_hal::tsens::{TemperatureSensor};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};

#[embassy_executor::task]
pub async fn temp_task(tsens: TemperatureSensor<'static>, TEMP_C: &'static Signal<CriticalSectionRawMutex, i8>) {
    // datasheet recommends 200 µs after power-up
    esp_hal::delay::Delay::new().delay_micros(200);

    loop {
        let t = tsens.get_temperature().to_celsius();
        let c = (t + 0.5) as i8;

        debug!("chip temperature = {:?} °C", c);
        TEMP_C.signal(c);
        Timer::after_secs(2).await;
    }
}
