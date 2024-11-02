#![no_std]
#![allow(non_snake_case)]

mod platform;
mod state_hp_idle;
mod state_ranging;
mod results_data;
mod uld_raw;
pub mod units;

#[cfg(feature = "defmt")]
use defmt::{debug, warn, trace, error};

use core::{
    ffi::CStr,
    fmt::{Display, Formatter},
    mem::MaybeUninit,
    ptr::addr_of_mut,
    result::Result as CoreResult,
};

use state_hp_idle::State_HP_Idle;

pub use state_ranging::{
    RangingConfig,
    State_Ranging,
    TargetOrder
};

pub use results_data::ResultsData;

use crate::uld_raw::{
    VL53L5CX_Configuration,
    vl53l5cx_init,
    API_REVISION as API_REVISION_r,   // &[u8] with terminating '\0'
    ST_OK, ST_ERROR,

    /*** tbd. if needed, bring under features
    *vl53l5cx_disable_internal_cp,
    *vl53l5cx_enable_internal_cp,
    *    //
    *vl53l5cx_dci_read_data,
    *vl53l5cx_dci_write_data,
    *    //
    *vl53l5cx_get_VHV_repeat_count,
    *vl53l5cx_set_VHV_repeat_count,
    */
};

pub type Result<T> = core::result::Result<T,Error>;

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(core::fmt::Debug)]
pub struct Error(pub u8);

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "ULD driver or hardware error ({})", self.0)
    }
}

/**
* @brief App provides, to talk to the I2C and do blocking delays; provides a mechanism to inform
*       the platform about an I2C address change.
*/
pub trait Platform {
    // provided by the app
    //
    // Note: We're using a slightly unconventional 'Result<(),()>' (no type for the errors).
    //      This is because of adaptation difficulties between the application-level I2C stack
    //      error values and the vendor ULD C API (that deals with 'u8's, but basically is a
    //      binary between ST_OK/ST_ERR). We didn't want to expose the library to I2C, nor the
    //      application to the ST_OK/ST_ERR.
    //
    //      This essentially eradicates the error type. We could (and tried):
    //          - using 'impl Any'  | not supported under 'stable'
    //          - using 'E: Error'  | works, but makes prototypes look complex (not good for training..)
    //          - bool              | just feels... wrong in Rust
    //
    //      A boolean would do, but using 'Result' is kinda customary.
    //
    fn rd_bytes(&mut self, index: u16, buf: &mut [u8]) -> CoreResult<(),()>;
    fn wr_bytes(&mut self, index: u16, vs: &[u8]) -> CoreResult<(),()>;
    fn delay_ms(&mut self, ms: u32);

    // This is our addition (vendor API struggles with the concept). Once we have changed the I2C
    // address the device identifies with, inform the 'Platform' struct about it.
    //
    fn addr_changed(&mut self, new_addr_8bit: u8);
}

pub const DEFAULT_I2C_ADDR_8BIT: u8 = 0x52;    // vendor default

// After LOTS of variations, here's a way to expose a 'CStr' string as a '&str' const (as long as
// we avoid '.unwrap*()' of any kind, it's const). Why it matters (does it tho?):
//  - follows the ULD C API closely
//  - works out-of-the-box for "defmt" ('&CStr' doesn't)
//
pub const API_REVISION: &str = {
    match unsafe{ CStr::from_bytes_with_nul_unchecked(API_REVISION_r) }.to_str() {
        Ok(s) => s,
        _ => unreachable!()     // use "" if this doesn't pass the compiler tbd.
    }
};

/*
* Adds a method to the ULD C API struct.
*
* Note: Since the C-side struct has quite a lot of internal "bookkeeping" fields, we don't expose
*       this directly to Rust customers, but wrap it. We *could* also consider making
*       those fields non-pub in the 'bindgen' phase, and be able to pass this struct, directly. #design
*/
impl VL53L5CX_Configuration {
    /** @brief Returns a default 'VL53L5CX_Configuration' struct, spiced with the application
       * provided 'Platform'-derived state (opaque to us, except for its size).
       *
       * Initialized state is (as per ULD C code):
       *   <<
       *       .platform: dyn Platform     = anything the app keeps there
       *       .streamcount: u8            = 0 (undefined by ULD C code)
       *       .data_read_size: u32        = 0 (undefined by ULD C code)
       *       .default_configuration: *mut u8 = VL53L5CX_DEFAULT_CONFIGURATION (a const table)
       *       .default_xtalk: *mut u8     = VL53L5CX_DEFAULT_XTALK (a const table)
       *       .offset_data: [u8; 488]     = data read from the sensor
       *       .xtalk_data: [u8; 776]      = copy of 'VL53L5CX_DEFAULT_XTALK'
       *       .temp_buffer: [u8; 1452]    = { being used for multiple things }
       *       .is_auto_stop_enabled: u8   = 0
       *   <<
       *
       * Side effects:
       *   - the sensor is reset, and firmware uploaded to it
       *   - NVM (non-volatile?) data is read from the sensor to the driver
       *   - default Xtalk data programmed to the sensor
       *   - default configuration ('.default_configuration') written to the sensor
       *   - four bytes written to sensor's DCI memory at '0xDB80U' ('VL53L5CX_DCI_PIPE_CONTROL'):
       *       {VL53L5CX_NB_TARGET_PER_ZONE, 0x00, 0x01, 0x00}
       *   - if 'NB_TARGET_PER_ZONE' != 1, 1 byte updated at '0x5478+0xc0' ('VL53L5CX_DCI_FW_NB_TARGET'+0xc0)  // if I got that right!?!
       *       {VL53L5CX_NB_TARGET_PER_ZONE}
       *   - one byte written to sensor's DCI memory at '0xD964' ('VL53L5CX_DCI_SINGLE_RANGE'):
       *       {0x01}
       *   - two bytes updated at sensor's DCI memory at '0x0e108' ('VL53L5CX_GLARE_FILTER'):
       *       {0x01, 0x01}
    */
    fn init_with<P : Platform + 'static>(mut p: P) -> Result<Self> {

        #[allow(unused_unsafe)]
        let ret: Result<VL53L5CX_Configuration> = unsafe {
            let mut uninit = MaybeUninit::<VL53L5CX_Configuration>::uninit();
                // note: use '::zeroed()' in place of '::uninit()' to get more predictability

            let up = uninit.as_mut_ptr();

            // Get a '&mut dyn Platform' reference to 'p'. Converting '*c_void' to a templated 'P'
            // type is beyond the author's imagination (perhaps impossible?) whereas '&mut dyn Platform'
            // *may be* just within doable!
            //
            // Nice part of using '&mut dyn Platform' is also that the size and alignment requirements
            // (16 and 8 bytes), remain constant for the C side.
            //
            let dd: &mut dyn Platform = &mut p;

            // Make a bitwise copy of 'dd' in 'uninit.platform'; ULD C 'vl.._init()' will need it,
            // and pass back to us (Rust) once platform calls (I2C/delays) are needed.
            //
            // This is what allows multiple VL boards to be utilized, at once; they each will get
            // their own, opaque 'Platform'.
            {
                let pp = addr_of_mut!((*up).platform);

                /*** disabled; gives _zero_ for the 'size_of_val(dd)'; WHY????
                // Check size and alignment
                let (sz_c,sz_rust) = (field_size!(VL53L5CX_Configuration::platform), size_of_val(dd));
                assert_eq!(sz_rust,sz_c, "Tunnel entry and exit sizes don't match");   // edit 'platform.h' to adjust
                ***/
                let al_rust = align_of_val(dd);
                assert!( (pp as usize)%al_rust == 0, "bad alignment on C side (needs {})", al_rust );

                *(pp as *mut &mut dyn Platform) = dd;
            }

            // Initialize those fields we know C API won't touch (just in case)
            addr_of_mut!((*up).streamcount).write(u8::MAX);
            addr_of_mut!((*up).data_read_size).write(u32::MAX);

            // Call ULD C API to arrange the rest
            //
            // Note: Already this will call the platform methods (via the tunnel).
            //
            match vl53l5cx_init(up) {
                0 => Ok(uninit.assume_init()),  // we guarantee it's now initialized
                e => Err(Error(e))
            }
        };
        ret
    }
}

/**
* @brief Beginning of preparing access to a single VL53L5CX sensor.
*/
pub struct VL53L5CX<P: Platform + 'static> {
    p: P
}

impl<P: Platform + 'static> VL53L5CX<P> {
    /*
    * Instead of just creating this structure, this already pings the bus to see, whether there's
    * a suitable sensor out there.
    */
    pub fn ping_new(/*move*/ mut p: P) -> Result<Self> {    // old name was: 'new_maybe()'
        match Self::ping(&mut p) {
            Err(_) => Err(Error(ST_ERROR)),
            Ok(()) => Ok(Self{ p })
        }
    }

    pub fn init(self) -> Result<State_HP_Idle> {
        let uld = VL53L5CX_Configuration::init_with(/*move*/ self.p)?;

        Ok( State_HP_Idle::new(uld) )
    }

    fn ping(p: &mut P) -> CoreResult<(),()> {
        #[cfg_attr(not(feature="defmt"), allow(unused_variables))]
        match vl53l5cx_ping(p)? {
            (a@ 0xf0, b@ 0x02) => {     // vendor driver ONLY proceeds with this
                #[cfg(feature="defmt")]
                debug!("Ping succeeded: {=u8:#04x},{=u8:#04x}", a,b)
            },
            t => {
                #[cfg(feature="defmt")]
                error!("Unexpected '(device id, rev id)': {:#04x}", t);
                return Err(());
            }
        }
        Ok(())
    }
}

/**
* Function, modeled akin to the vendor ULD 'vl53l5cx_is_alive()', but:
*   - made in Rust
*   - returns the device and revision id's
*
* This is the only code that the ULD C driver calls on the device, prior to '.init()', i.e. it
* is supposed to be functioning also before the firmware and parameters initialization.
*
* Vendor note:
*   - ULD C driver code expects '(0xf0, 0x02)'.
*/
fn vl53l5cx_ping<P : Platform>(pl: &mut P) -> CoreResult<(u8,u8),()> {
    let mut buf: [u8;2] = [u8::MAX;2];

    pl.wr_bytes(0x7fff, &[0x00])?;
    pl.rd_bytes(0, &mut buf)?;   // [dev_id, rev_id]
    pl.wr_bytes(0x7fff, &[0x02])?;

    Ok( (buf[0], buf[1]) )
}
