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
type InPin = hal::gpio::Pin<hal::gpio::DynPinId, hal::gpio::FunctionSio<hal::gpio::SioInput>, hal::gpio::PullUp>;

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

/// The two GPIOs a PS/2 keyboard's CLK/DATA lines are wired to.
struct Ps2Pins {
    clk: InPin,
    data: InPin,
}

/// In-progress 11-bit PS/2 frame (start + 8 data bits LSB-first + parity +
/// stop), plus the break/extended prefix flags that arrive as their own
/// separate frames and must persist until the frame after them.
/// `last_clk_high` is software edge-detection state — CLK is polled, not
/// interrupt-driven, so a falling edge is "was high last poll, low now".
struct Ps2State {
    bits: u16,
    count: u8,
    pending_break: bool,
    pending_extended: bool,
    last_clk_high: bool,
}

impl Ps2State {
    const fn new() -> Self {
        Ps2State { bits: 0, count: 0, pending_break: false, pending_extended: false, last_clk_high: true }
    }
}

static PS2_PINS: Mutex<RefCell<Option<Ps2Pins>>> = Mutex::new(RefCell::new(None));
static PS2_STATE: Mutex<RefCell<Ps2State>> = Mutex::new(RefCell::new(Ps2State::new()));
static KEY_QUEUE: Mutex<RefCell<VecDeque<char>>> = Mutex::new(RefCell::new(VecDeque::new()));
