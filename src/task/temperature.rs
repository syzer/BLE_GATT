#[no_std]
use defmt::info;
use embassy_time::Timer;
use esp_hal::tsens::{Config as TsensConfig, TemperatureSensor};

#[embassy_executor::task]
pub async fn temp_task(tsens: TemperatureSensor<'static>) {
    // datasheet recommends 200 µs after power-up
    esp_hal::delay::Delay::new().delay_micros(200);

    loop {
        let t = tsens.get_temperature();
        let c = t.to_celsius();
        info!("chip temperature = {:?} °C", c);
        Timer::after_secs(2).await;
    }
}