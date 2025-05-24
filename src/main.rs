#![no_main]
#![no_std]
#![allow(static_mut_refs)]

pub mod music;
pub mod rgb;

use crate::music::list::MUSIC_LIST;
use core::{cell::RefCell, mem::zeroed, sync::atomic::Ordering};
use cortex_m::interrupt::Mutex;
use cortex_m_rt::entry;
use defmt::{debug, info};
use defmt_rtt as _;
use music::{MusicInterrupt, next_music, previous_music, turn_music};
use panic_probe as _;
use portable_atomic::AtomicBool;
use rgb::switch_rgb_led;
use stm32f0xx_hal::{
    self as hal,
    gpio::{
        Input, PullDown,
        gpioa::{PA11, PA12},
        gpiob::{PB6, PB7},
    },
    pac::{Interrupt, NVIC, TIM3, TIM16, interrupt},
    prelude::*,
    pwm::{self, PwmChannels},
    rcc::HSEBypassMode,
    timers::{Event, Timer},
};

pub type PWM3Channels = pwm::PWM3<(
    PwmChannels<TIM3, pwm::C1>,
    PwmChannels<TIM3, pwm::C2>,
    PwmChannels<TIM3, pwm::C3>,
    PwmChannels<TIM3, pwm::C4>,
)>;

static mut PWM3_CHANNELS: PWM3Channels = unsafe { zeroed() };

static TIMER16: Mutex<RefCell<Timer<TIM16>>> = Mutex::new(RefCell::new(unsafe { zeroed() }));

static PREVIOUS_BTN: Mutex<RefCell<PB6<Input<PullDown>>>> =
    Mutex::new(RefCell::new(unsafe { zeroed() }));
static LAST_PREVIOUS_STATE: AtomicBool = AtomicBool::new(false);

static NEXT_BTN: Mutex<RefCell<PB7<Input<PullDown>>>> =
    Mutex::new(RefCell::new(unsafe { zeroed() }));
static LAST_NEXT_STATE: AtomicBool = AtomicBool::new(false);

static TURN_BTN: Mutex<RefCell<PA11<Input<PullDown>>>> =
    Mutex::new(RefCell::new(unsafe { zeroed() }));
static LAST_TURN_STATE: AtomicBool = AtomicBool::new(false);

static RGB_BTN: Mutex<RefCell<PA12<Input<PullDown>>>> =
    Mutex::new(RefCell::new(unsafe { zeroed() }));
static LAST_RGB_STATE: AtomicBool = AtomicBool::new(false);

#[entry]
fn main() -> ! {
    let (Some(mut dp), Some(mut cp)) = (
        hal::pac::Peripherals::take(),
        cortex_m::peripheral::Peripherals::take(),
    ) else {
        panic!("failed to get peripherals");
    };

    info!("startup Birthday Music Board!");

    let mut rcc = dp
        .RCC
        .configure()
        .hse(8.mhz(), HSEBypassMode::NotBypassed)
        .sysclk(48.mhz())
        .pclk(48.mhz())
        .freeze(&mut dp.FLASH);

    let gpioa = dp.GPIOA.split(&mut rcc);
    let gpiob = dp.GPIOB.split(&mut rcc);

    cortex_m::interrupt::free(|cs| {
        TIMER16.borrow(cs).replace({
            let mut timer = Timer::tim16(dp.TIM16, 50.hz(), &mut rcc);
            timer.listen(Event::TimeOut);
            timer
        });
        PREVIOUS_BTN
            .borrow(cs)
            .replace(gpiob.pb6.into_pull_down_input(cs));
        NEXT_BTN
            .borrow(cs)
            .replace(gpiob.pb7.into_pull_down_input(cs));
        TURN_BTN
            .borrow(cs)
            .replace(gpioa.pa11.into_pull_down_input(cs));
        RGB_BTN
            .borrow(cs)
            .replace(gpioa.pa12.into_pull_down_input(cs));
    });

    // 配置延时器
    let mut delay = hal::delay::Delay::new(cp.SYST, &rcc);

    let pins = cortex_m::interrupt::free(move |cs| {
        (
            gpiob.pb4.into_alternate_af1(cs),
            gpiob.pb5.into_alternate_af1(cs),
            gpiob.pb0.into_alternate_af1(cs),
            gpiob.pb1.into_alternate_af1(cs),
        )
    });

    let pwm3 = hal::pwm::tim3(dp.TIM3, pins, &mut rcc, 20.khz());
    unsafe {
        PWM3_CHANNELS = pwm3;
    }

    unsafe {
        cp.NVIC.set_priority(Interrupt::TIM16, 1);
        NVIC::unmask(Interrupt::TIM16);
    }

    let mut index = 0;

    loop {
        match MUSIC_LIST[index].play(&mut rcc, &mut delay) {
            Ok(()) | Err(MusicInterrupt::Next) => {
                index = (index + 1) % MUSIC_LIST.len();
            }
            Err(MusicInterrupt::Previous) => {
                index = (index + MUSIC_LIST.len() - 1) % MUSIC_LIST.len();
            }
        }
    }
}

/// 定时器实现软件防抖
#[interrupt]
fn TIM16() {
    cortex_m::interrupt::free(|cs| {
        TIMER16.borrow(cs).borrow_mut().clear_irq();

        {
            let current_state = RGB_BTN.borrow(cs).borrow().is_high().unwrap();
            let last_state = LAST_RGB_STATE.swap(current_state, Ordering::SeqCst);
            if !last_state && current_state {
                debug!("RGB灯按键被按下！");
                switch_rgb_led();
            }
        }

        {
            let current_state = PREVIOUS_BTN.borrow(cs).borrow().is_high().unwrap();
            let last_state = LAST_PREVIOUS_STATE.swap(current_state, Ordering::SeqCst);
            if !last_state && current_state {
                debug!("上一首按键被按下！");
                previous_music();
            }
        }

        {
            let current_state = NEXT_BTN.borrow(cs).borrow().is_high().unwrap();
            let last_state = LAST_NEXT_STATE.swap(current_state, Ordering::SeqCst);
            if !last_state && current_state {
                debug!("下一首按键被按下！");
                next_music();
            }
        }

        {
            let current_state = TURN_BTN.borrow(cs).borrow().is_high().unwrap();
            let last_state = LAST_TURN_STATE.swap(current_state, Ordering::SeqCst);
            if !last_state && current_state {
                debug!("暂停/继续按键被按下！");
                turn_music();
            }
        }
    });
}
