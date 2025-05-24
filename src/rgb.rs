use crate::PWM3_CHANNELS;
use core::sync::atomic::Ordering;
use defmt::info;
use portable_atomic::AtomicBool;
use stm32f0xx_hal::prelude::*;

/// 开关灯
pub fn switch_rgb_led() {
    static STATE: AtomicBool = AtomicBool::new(false);

    let state = !STATE.fetch_not(Ordering::SeqCst);

    let channels = unsafe { PWM3_CHANNELS.channels_mut() };

    if state {
        info!("开灯！");
        channels.0.enable();
        channels.1.enable();
        channels.2.enable();
    } else {
        info!("关灯！");
        channels.0.disable();
        channels.1.disable();
        channels.2.disable();
    }
}
