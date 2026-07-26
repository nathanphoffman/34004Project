// Boot image metadata, the global allocator, and the shared hardware
// types (Lcd, DataBus, Hardware) used by lcd.rs and hardware.rs.

/// Tell the RP2350 Boot ROM about this application.
#[cfg(feature = "rp2350")]
#[link_section = ".start_block"]
#[used]
pub static IMAGE_DEF: hal::block::ImageDef = hal::block::ImageDef::secure_exe();

/// RP2040 needs an explicit second-stage bootloader blob instead — required
/// only for the Wokwi (Pico 1) simulation build.
#[cfg(feature = "rp2040")]
#[link_section = ".boot2"]
#[used]
pub static BOOT2_FIRMWARE: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

/// External crystal on the Raspberry Pi Pico 2 board.
const XTAL_FREQ_HZ: u32 = 12_000_000u32;

/// Deor strings transpile to heap-allocated `String`, so `alloc` needs a
/// global allocator even for this tiny amount of text.
const HEAP_SIZE: usize = 4096;
static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];

#[global_allocator]
static HEAP: embedded_alloc::Heap = embedded_alloc::Heap::empty();

type OutPin = hal::gpio::Pin<hal::gpio::DynPinId, hal::gpio::FunctionSioOutput, hal::gpio::PullDown>;

struct Lcd {
    rs: OutPin,
    en: OutPin,
}

struct DataBus {
    d4: OutPin,
    d5: OutPin,
    d6: OutPin,
    d7: OutPin,
}

struct Hardware {
    bus: DataBus,
    lcd1: Lcd,
    lcd2: Lcd,
    lcd3: Lcd,
    delay: cortex_m::delay::Delay,
}

static HARDWARE: Mutex<RefCell<Option<Hardware>>> = Mutex::new(RefCell::new(None));
