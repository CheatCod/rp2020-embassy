#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use core::pin::pin;

use cortex_m::delay::Delay;
use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::{
    gpio::{self, AnyPin, Input, Pin, Pull},
    Peripheral, Peripherals,
};
use embassy_sync::{
    blocking_mutex::raw::{NoopRawMutex, ThreadModeRawMutex},
    mutex::Mutex,
    pubsub::PubSubChannel,
};
use embassy_time::{Duration, Timer};
use futures::future::select;
use gpio::{Level, Output};
use {defmt_rtt as _, panic_probe as _};

static channel: PubSubChannel<ThreadModeRawMutex, Event, 16, 16, 16> = PubSubChannel::new();

static PRESSED_COUNTER: Mutex<ThreadModeRawMutex, u32> = Mutex::new(0);

// static PRESSED_COUNTER: ThreadModeRawMutex<u32> = ThreadModeRawMutex::new();

#[derive(Clone)]
enum Event {
    // if pressed for less than 100 ms
    ButtonPressed,
    // if pressed for more than 100 ms
    ButtonHeldStart,
    ButtonHeldEnd,
}

struct BinaryLedArray<'d, const N: usize> {
    pub leds: [Output<'d, AnyPin>; N],
}

impl<'d, const N: usize> BinaryLedArray<'d, N> {
    pub fn set(&mut self, value: u64) {
        for i in 0..N {
            if value & (1 << i) != 0 {
                self.leds[i].set_high();
            } else {
                self.leds[i].set_low();
            }
        }
    }

    pub fn clear(&mut self) {
        for i in 0..N {
            self.leds[i].set_low();
        }
    }

    pub fn all(&mut self) {
        for i in 0..N {
            self.leds[i].set_high();
        }
    }
}

#[embassy_executor::task]
async fn button_event_task(pin: AnyPin) {
    let mut button = Input::new(pin, Pull::Down);
    let publisher = channel.publisher().unwrap();

    loop {
        button.wait_for_high().await;
        // sleep for 300 ms or until button is released
        if let futures::future::Either::Left(_) = select(
            pin!(button.wait_for_low()),
            Timer::after(Duration::from_millis(250)),
        )
        .await
        {
            publisher.publish(Event::ButtonPressed).await;
            continue;
        }
        publisher.publish(Event::ButtonHeldStart).await;
        button.wait_for_low().await;
        publisher.publish(Event::ButtonHeldEnd).await;
    }
}

#[embassy_executor::task]
async fn event_handler(pin: AnyPin, mut leds: BinaryLedArray<'static, 5>) {
    let mut subscriber = channel.subscriber().unwrap();
    let mut led = Output::new(pin, Level::Low);

    loop {
        match subscriber.next_message_pure().await {
            Event::ButtonPressed => {
                led.toggle();
                let mut lock = PRESSED_COUNTER.lock().await;
                *lock += 1;
                leds.set(*lock as u64);
                info!("Button pressed");
            }
            Event::ButtonHeldStart => {
                for _ in 0..8 {
                    led.toggle();
                    Timer::after(Duration::from_millis(100)).await;
                }
                led.set_low();
                leds.clear();
                info!("Button pressed {} times", *PRESSED_COUNTER.lock().await);
                *PRESSED_COUNTER.lock().await = 0;
            }
            Event::ButtonHeldEnd => {}
        }
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let Peripherals {
        PIN_14,
        PIN_5,
        PIN_6,
        PIN_7,
        PIN_8,
        PIN_9,
        PIN_15,
        ..
    } = embassy_rp::init(Default::default());
    let leds = BinaryLedArray {
        leds: [
            Output::new(PIN_9.degrade(), Level::Low),
            Output::new(PIN_8.degrade(), Level::Low),
            Output::new(PIN_7.degrade(), Level::Low),
            Output::new(PIN_6.degrade(), Level::Low),
            Output::new(PIN_5.degrade(), Level::Low),
        ],
    };

    spawner.spawn(button_event_task(PIN_14.degrade())).unwrap();
    spawner
        .spawn(event_handler(PIN_15.degrade(), leds))
        .unwrap();
}
