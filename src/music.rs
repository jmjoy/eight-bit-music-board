pub mod list;
pub mod note;

use crate::PWM3_CHANNELS;
use core::sync::atomic::Ordering;
use defmt::info;
use note::RST;
use portable_atomic::AtomicBool;
use stm32f0xx_hal::{delay::Delay, prelude::*, rcc::Rcc};

static PREVIOUS_STATE: AtomicBool = AtomicBool::new(false);
static NEXT_STATE: AtomicBool = AtomicBool::new(false);
static TURN_STATE: AtomicBool = AtomicBool::new(false);

pub enum MusicInterrupt {
    Previous,
    Next,
}

pub struct Music<'a> {
    pub name: &'a str,
    pub melody: &'a [(u16, u16)],
}

impl<'a> Music<'a> {
    pub fn play(&self, rcc: &mut Rcc, delay: &mut Delay) -> Result<(), MusicInterrupt> {
        info!("开始播放《{}》……", self.name);
        delay.delay_ms(300u16);

        let channels = unsafe { PWM3_CHANNELS.channels_mut() };

        for &(freq, duration) in self.melody {
            self.check_should_stop()?;

            // 灯光
            {
                channels
                    .0
                    .set_duty(channels.0.get_max_duty() / (freq % 7 + 1));
                channels
                    .1
                    .set_duty(channels.1.get_max_duty() / (freq % 8 + 1));
                channels
                    .2
                    .set_duty(channels.2.get_max_duty() / (freq % 9 + 1));
            }

            if freq == RST {
                // 休止符：禁用PWM
                channels.3.disable();
                delay.delay_ms(duration);
            } else {
                // 播放音符：设置频率并启用PWM
                unsafe {
                    PWM3_CHANNELS.set_freq(rcc, (freq as u32).hz());
                }
                channels.3.set_duty(channels.3.get_max_duty() / 2);
                channels.3.enable();
                delay.delay_ms(duration);
            }

            // 间隔
            {
                channels.3.disable();
                delay.delay_ms(10u8);
            }
        }

        {
            let channels = unsafe { PWM3_CHANNELS.channels_mut() };
            channels.3.disable();
        }

        // 音乐播放完毕后暂停
        delay.delay_ms(1500u16);

        self.check_should_stop()?;

        Ok(())
    }

    fn check_should_stop(&self) -> Result<(), MusicInterrupt> {
        if TURN_STATE.swap(false, Ordering::SeqCst) {
            info!("暂停！");
            loop {
                cortex_m::asm::wfi();
                if TURN_STATE.swap(false, Ordering::SeqCst) {
                    info!("继续！");
                    break Ok(());
                }
                self.check_should_interrupt()?;
            }
        } else {
            self.check_should_interrupt()?;
            Ok(())
        }
    }

    fn check_should_interrupt(&self) -> Result<(), MusicInterrupt> {
        if PREVIOUS_STATE.swap(false, Ordering::SeqCst) {
            info!("上一曲！");
            NEXT_STATE.store(false, Ordering::SeqCst);
            return Err(MusicInterrupt::Previous);
        }

        if NEXT_STATE.swap(false, Ordering::SeqCst) {
            info!("下一曲！");
            PREVIOUS_STATE.store(false, Ordering::SeqCst);
            return Err(MusicInterrupt::Next);
        }

        Ok(())
    }
}

pub fn previous_music() {
    PREVIOUS_STATE.store(true, Ordering::SeqCst);
}

pub fn next_music() {
    NEXT_STATE.store(true, Ordering::SeqCst);
}

pub fn turn_music() {
    TURN_STATE.store(true, Ordering::SeqCst);
}
