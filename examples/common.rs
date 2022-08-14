#![allow(unused_attributes)]
#![no_std]
#![no_main]

//! Common features used in examples.
//!
//! Note that this module isn't an example by itself.

use stm32_eth::{
    hal::{gpio::GpioExt, rcc::Clocks},
    stm32::{ETHERNET_DMA, ETHERNET_MAC, ETHERNET_MMC},
};

pub use pins::{setup_pins, Gpio};

use fugit::RateExtU32;
use stm32_eth::hal::rcc::RccExt;

pub struct EthernetPeripherals {
    pub dma: ETHERNET_DMA,
    pub mac: ETHERNET_MAC,
    pub mmc: ETHERNET_MMC,
}

/// Setup the clocks and return clocks and a GPIO struct that
/// can be used to set up all of the pins.
///
/// This configures HCLK to be at least 25 MHz, which is the minimum required
/// for ethernet operation to be valid.
pub fn setup_clocks(p: stm32_eth::stm32::Peripherals) -> (Clocks, Gpio, EthernetPeripherals) {
    let ethernet = EthernetPeripherals {
        dma: p.ETHERNET_DMA,
        mac: p.ETHERNET_MAC,
        mmc: p.ETHERNET_MMC,
    };

    #[cfg(any(feature = "stm32f7xx-hal", feature = "stm32f4xx-hal"))]
    {
        let rcc = p.RCC.constrain();

        let clocks = rcc.cfgr.sysclk(100.MHz()).hclk(100.MHz()).freeze();

        let gpio = Gpio {
            gpioa: p.GPIOA.split(),
            gpiob: p.GPIOB.split(),
            gpioc: p.GPIOC.split(),
            gpiog: p.GPIOG.split(),
        };

        (clocks, gpio, ethernet)
    }

    #[cfg(feature = "stm32f1xx-hal")]
    {
        use stm32_eth::hal::flash::FlashExt;

        let rcc = p.RCC.constrain();
        let mut flash = p.FLASH.constrain();

        let clocks = rcc
            .cfgr
            .sysclk(72.MHz())
            .hclk(72.MHz())
            .freeze(&mut flash.acr);

        let gpio = Gpio {
            gpioa: p.GPIOA.split(),
            gpiob: p.GPIOB.split(),
            gpioc: p.GPIOC.split(),
        };

        (clocks, gpio, ethernet)
    }
}

pub use pins::*;

#[cfg(any(feature = "stm32f4xx-hal", feature = "stm32f7xx-hal",))]
mod pins {
    use stm32_eth::{hal::gpio::*, EthPins};

    pub struct Gpio {
        pub gpioa: gpioa::Parts,
        pub gpiob: gpiob::Parts,
        pub gpioc: gpioc::Parts,
        pub gpiog: gpiog::Parts,
    }

    pub type RefClk = PA1<Input>;
    pub type Crs = PA7<Input>;
    pub type TxD1 = PB13<Input>;
    pub type RxD0 = PC4<Input>;
    pub type RxD1 = PC5<Input>;

    #[cfg(not(feature = "example-nucleo-pins"))]
    pub type TxEn = PB11<Input>;
    #[cfg(not(feature = "example-nucleo-pins"))]
    pub type TxD0 = PB12<Input>;

    #[cfg(all(feature = "example-nucleo-pins"))]
    pub type TxEn = PG11<Input>;
    #[cfg(feature = "example-nucleo-pins")]
    pub type TxD0 = PG13<Input>;

    pub type Mdio = PA2<Alternate<11>>;
    pub type Mdc = PC1<Alternate<11>>;

    pub fn setup_pins(
        gpio: Gpio,
    ) -> (
        EthPins<RefClk, Crs, TxEn, TxD0, TxD1, RxD0, RxD1>,
        Mdio,
        Mdc,
    ) {
        #[allow(unused_variables)]
        let Gpio {
            gpioa,
            gpiob,
            gpioc,
            gpiog,
        } = gpio;

        let ref_clk = gpioa.pa1.into_floating_input();
        let crs = gpioa.pa7.into_floating_input();
        let tx_d1 = gpiob.pb13.into_floating_input();
        let rx_d0 = gpioc.pc4.into_floating_input();
        let rx_d1 = gpioc.pc5.into_floating_input();

        #[cfg(not(feature = "example-nucleo-pins"))]
        let (tx_en, tx_d0) = (
            gpiob.pb11.into_floating_input(),
            gpiob.pb12.into_floating_input(),
        );

        #[cfg(feature = "example-nucleo-pins")]
        let (tx_en, tx_d0) = {
            (
                gpiog.pg11.into_floating_input(),
                gpiog.pg13.into_floating_input(),
            )
        };

        #[cfg(feature = "stm32f4xx-hal")]
        let (mdio, mdc) = {
            let mut mdio = gpioa.pa2.into_alternate();
            mdio.set_speed(Speed::VeryHigh);
            let mut mdc = gpioc.pc1.into_alternate();
            mdc.set_speed(Speed::VeryHigh);
            (mdio, mdc)
        };

        #[cfg(any(feature = "stm32f7xx-hal"))]
        let (mdio, mdc) = (
            gpioa.pa2.into_alternate().set_speed(Speed::VeryHigh),
            gpioc.pc1.into_alternate().set_speed(Speed::VeryHigh),
        );

        (
            EthPins {
                ref_clk,
                crs,
                tx_en,
                tx_d0,
                tx_d1,
                rx_d0,
                rx_d1,
            },
            mdio,
            mdc,
        )
    }
}

#[cfg(any(feature = "stm32f1xx-hal"))]
mod pins {
    use stm32_eth::{
        hal::gpio::{Alternate, Input, PushPull, *},
        EthPins,
    };

    pub struct Gpio {
        pub gpioa: gpioa::Parts,
        pub gpiob: gpiob::Parts,
        pub gpioc: gpioc::Parts,
    }

    pub type RefClk = PA1<Input<Floating>>;
    pub type Crs = PA7<Input<Floating>>;
    pub type TxEn = PB11<Alternate<PushPull>>;
    pub type TxD0 = PB12<Alternate<PushPull>>;
    pub type TxD1 = PB13<Alternate<PushPull>>;
    pub type RxD0 = PC4<Input<Floating>>;
    pub type RxD1 = PC5<Input<Floating>>;

    pub type Mdio = PA2<Alternate<PushPull>>;
    pub type Mdc = PC1<Alternate<PushPull>>;

    pub fn setup_pins(
        gpio: Gpio,
    ) -> (
        EthPins<RefClk, Crs, TxEn, TxD0, TxD1, RxD0, RxD1>,
        Mdio,
        Mdc,
    ) {
        let Gpio {
            mut gpioa,
            mut gpiob,
            mut gpioc,
        } = gpio;

        let ref_clk = gpioa.pa1.into_floating_input(&mut gpioa.crl);
        let mdio = gpioa.pa2.into_alternate_push_pull(&mut gpioa.crl);
        let crs = gpioa.pa7.into_floating_input(&mut gpioa.crl);

        let mdc = gpioc.pc1.into_alternate_push_pull(&mut gpioc.crl);
        let rx_d0 = gpioc.pc4.into_floating_input(&mut gpioc.crl);
        let rx_d1 = gpioc.pc5.into_floating_input(&mut gpioc.crl);

        let tx_en = gpiob.pb11.into_alternate_push_pull(&mut gpiob.crh);
        let tx_d0 = gpiob.pb12.into_alternate_push_pull(&mut gpiob.crh);
        let tx_d1 = gpiob.pb13.into_alternate_push_pull(&mut gpiob.crh);

        let pins = EthPins {
            ref_clk,
            crs,
            tx_en,
            tx_d0,
            tx_d1,
            rx_d0,
            rx_d1,
        };

        (pins, mdio, mdc)
    }
}

use ieee802_3_miim::{
    phy::{
        lan87xxa::{LAN8720A, LAN8742A},
        BarePhy, KSZ8081R,
    },
    Miim, Pause, Phy,
};

/// An ethernet PHY
pub enum EthernetPhy<M: Miim> {
    /// LAN8720A
    LAN8720A(LAN8720A<M>),
    /// LAN8742A
    LAN8742A(LAN8742A<M>),
    /// KSZ8081R
    KSZ8081R(KSZ8081R<M>),
}

impl<M: Miim> Phy<M> for EthernetPhy<M> {
    fn best_supported_advertisement(&self) -> ieee802_3_miim::AutoNegotiationAdvertisement {
        unimplemented!()
    }

    fn get_miim(&mut self) -> &mut M {
        match self {
            EthernetPhy::LAN8720A(phy) => phy.get_miim(),
            EthernetPhy::LAN8742A(phy) => phy.get_miim(),
            EthernetPhy::KSZ8081R(phy) => phy.get_miim(),
        }
    }

    fn get_phy_addr(&self) -> u8 {
        match self {
            EthernetPhy::LAN8720A(phy) => phy.get_phy_addr(),
            EthernetPhy::LAN8742A(phy) => phy.get_phy_addr(),
            EthernetPhy::KSZ8081R(phy) => phy.get_phy_addr(),
        }
    }
}

impl<M: Miim> EthernetPhy<M> {
    /// Attempt to create one of the known PHYs from the given
    /// MIIM.
    ///
    /// Returns an error if the PHY does not support the extended register
    /// set, or if the PHY's identifier does not correspond to a known PHY.
    pub fn from_miim(miim: M, phy_addr: u8) -> Result<Self, ()> {
        let mut bare = BarePhy::new(miim, phy_addr, Pause::NoPause);
        let phy_ident = bare.phy_ident().ok_or(())?;
        let miim = bare.release();
        match phy_ident & 0xFFFFFFF0 {
            0x0007C0F0 => Ok(Self::LAN8720A(LAN8720A::new(miim, phy_addr))),
            0x0007C130 => Ok(Self::LAN8742A(LAN8742A::new(miim, phy_addr))),
            0x00221560 => Ok(Self::KSZ8081R(KSZ8081R::new(miim, phy_addr))),
            _ => Err(()),
        }
    }

    /// Get a string describing the type of PHY
    pub const fn ident_string(&self) -> &'static str {
        match self {
            EthernetPhy::LAN8720A(_) => "LAN8720A",
            EthernetPhy::LAN8742A(_) => "LAN8742A",
            EthernetPhy::KSZ8081R(_) => "KSZ8081R",
        }
    }

    /// Initialize the PHY
    pub fn phy_init(&mut self) {
        match self {
            EthernetPhy::LAN8720A(phy) => phy.phy_init(),
            EthernetPhy::LAN8742A(phy) => phy.phy_init(),
            EthernetPhy::KSZ8081R(phy) => {
                phy.set_autonegotiation_advertisement(phy.best_supported_advertisement());
            }
        }
    }
}
