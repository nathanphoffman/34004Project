/* RP2040 (Pico 1) memory layout — used only for the Wokwi simulation
 * build, since Wokwi has no RP2350 core yet. See memory.x for the real
 * (RP2350 / Pico 2) hardware layout.
 */
MEMORY {
    BOOT2 : ORIGIN = 0x10000000, LENGTH = 0x100
    FLASH : ORIGIN = 0x10000100, LENGTH = 2048K - 0x100
    RAM   : ORIGIN = 0x20000000, LENGTH = 256K
}

EXTERN(BOOT2_FIRMWARE)

SECTIONS {
    .boot2 ORIGIN(BOOT2) :
    {
        KEEP(*(.boot2));
    } > BOOT2
} INSERT BEFORE .text;
