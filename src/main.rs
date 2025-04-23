#![no_std]
#![no_main]


use esp_backtrace as _;
use esp_hal::{
    delay::Delay, gpio::Level::{High, Low}, main, rmt::{ PulseCode, Rmt, TxChannel, TxChannelConfig, TxChannelCreator}, rng::Rng, time::{Instant, Rate}
};
const PERIOD_RMT: u32 = 1250;
const LED_T0H: u32 = 320;
const LED_T0L: u32 = PERIOD_RMT - LED_T0H;
const LED_T1H: u32 = 640;
const LED_T1L: u32 = PERIOD_RMT - LED_T1H;
const APB_FREQUENCY_MHZ: u32 = 80;
#[derive(PartialEq, Clone, Copy, Debug)]
struct RGB {
    green: u8,
    red: u8,
    blue: u8,
}

impl RGB {
    fn new(rng: &mut Rng) -> Self {
        Self {
            green: (rng.random() % 255) as u8,
            red: (rng.random() % 255) as u8,
            blue: (rng.random() % 255) as u8,
        }
    }

    fn to_u32(self) -> u32 {
        return (self.blue as u32) + ((self.red as u32) << 8) + ((self.green as u32) << 16);
    }
}
trait LedSerializable {
    fn to_color(self) -> [u32;25];
    fn get_blue(self) -> u32;
    fn get_red(self) -> u32;
    fn get_green(self) -> u32;
}

impl LedSerializable for u32 {

    fn to_color(self) -> [u32; 25] {
    
        let bit_1: u32 = PulseCode::new(
            High,
            (LED_T1H / (1000 / APB_FREQUENCY_MHZ) ) as u16,
            Low,
            (LED_T1L / (1000 / APB_FREQUENCY_MHZ) ) as u16);

        let bit_0: u32 = PulseCode::new(
            High,
            (LED_T0H / (1000 / APB_FREQUENCY_MHZ) ) as u16,
            Low,
            (LED_T0L / (1000 / APB_FREQUENCY_MHZ) ) as u16);

        let data = self;
        let r: [u32; 25] = core::array::from_fn(|i| if i != 24 {if data & (1 << (23 - i)) != 0 {bit_1} else {bit_0}} else {PulseCode::empty()} );
        r
        
    }
    fn get_green(self) -> u32 {
        return self & 0xff
    }
    fn get_red(self) -> u32 {
        return self & 0xff00
    }
    fn get_blue(self) -> u32 {
        return self & 0xff0000
    }

}



#[main]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let freq = Rate::from_mhz(80);
    let mut rng = Rng::new(peripherals.RNG);
    let rmt = Rmt::new(peripherals.RMT, freq).unwrap();
    let mut rmt_channel = rmt.channel0.configure(peripherals.GPIO8, 
        TxChannelConfig::default().with_clk_divider(1)
            .with_carrier_high(0)
            .with_carrier_modulation(false)
            .with_carrier_level(Low)
            .with_idle_output(false)
            .with_idle_output_level(Low)).unwrap();

    let mut last_time = Instant::now();
    let delay = Delay::new();
    let mut ep: [RGB; 2] = core::array::from_fn(|_| RGB::new(&mut rng));
    let mut color = ep[0].to_u32();
    
    loop {
        let delta = (last_time.elapsed().as_millis() + 1) as u32;
        if ep[0] == ep[1] {
            ep[1] = RGB::new(&mut rng);
        }

        

        if delta > 20 {
            
            if ep[0].green < ep[1].green {
                ep[0].green += 1;
                
            } else if ep[0].green > ep[1].green {
                ep[0].green -= 1;
                
            }
            if ep[0].red < ep[1].red {
                ep[0].red += 1;
                
            } else if ep[0].red > ep[1].red {
                ep[0].red -= 1;
                
            }
            if ep[0].blue < ep[1].blue {
                ep[0].blue += 1;
            } else if ep[0].blue > ep[1].blue {
                ep[0].blue -= 1
            }

            color = ep[0].to_u32();
            
            last_time = Instant::now();
            
            
        }
        
        let data = color.to_color();
        let transaction = rmt_channel.transmit(&data).unwrap();
        
        rmt_channel = transaction.wait().unwrap();

        delay.delay_micros(100);
    }
}
