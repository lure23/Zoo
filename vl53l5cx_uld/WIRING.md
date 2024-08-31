# Wiring

This document applies to a version of the [VL53L5CX-SATEL](https://www.st.com/en/evaluation-tools/vl53l5cx-satel.html#documentation) development boards marked with "PCB4109A" on the bottom side.

![](.images/VL53L5CX%20bottom.png)

![](.images/Wiring-ESP32-C3-DevKit-C02.png)

*Figure 1. Wiring the SATEL board with ESP32-C3-DevKit-C02*

Below, we go through the SATEL pins from left to right.

|SATEL pin|ESP32 pin|comments|SATEL|What can I do with it?|
|---|---|---|---|---|
|INT|---|Active low (open drain)|Pulled via 47k to IOVDD.|Make the waiting for a new frame non-polling. <font color=orange>*(tbd. confirm by testing)*</font>|
|I2C_RST|---|Active high. Toggle `0->1->0` to reset the I2C target.|Pulled via 47k to GND.|Reset the I2C side by pulling the pin momentarily up while `PWR_EN` remains up. Not needed in practice.|
|SDA|GPIO4|same pin as in a `esp-hal` I2C example|Pulled via 2.2k to IOVDD.<br />If you chain multiple boards, remove extra pull-ups by soldering open `SB5`, `SB7` on the underside of boards.|Talk with the device.|
|SCL|GPIO5|-''-|-''-|-''-|
|LPn|---|Chip enable, active high.|Pulled via 47k to IOVDD.|Disable the chip momentarily, by connecting to GND. Suggested to be used for programming non-default I2C addresses, but this can be reached also by simply plugging such chips in, on their own.<sup>(1)</sup>|
|PWR_EN|(GPIO0)|Drive directly with a GPIO pin, or pull up with e.g. 47k to IOVDD.|Drives the `CE` (chip enable) of the larger board's regulator. The whole VL53L5CX chip is powered off, unless this signal is high.|Control the power cycle of the VC53L5CX chip.|
|AVDD|5v / 3v3|Both AVDD and IOVDD must be provided, for the SATEL board to function. *(don't exactly understand, why, based on the schematics)*|
|IOVDD|3v3|-''-|
|GND|Gnd|

`(1)`: Haven't tried. If the VL device doesn't have persistent memory of its I2C address, one needs to initialize them via this feature (disable all but one; program the I2C address).


>For detailed description of the pins, see [`[1]`](https://www.st.com/resource/en/datasheet/vl53l5cx.pdf) table 3 ("VL53L5CX pin description").
>
>`PWR_EN` is a pin specific to the larger part (5v -> 3v3 conversion) of the SATEL board.

**Quotes**

|*"[...] SATEL has already the pull ups while the breakout board doesn't."* <sub>[source](https://community.st.com/t5/imaging-sensors/vl53l5cx-satel-won-t-respond-to-i2c/td-p/597080)</sub>|
|---|
|<strike>i.e. if you use the *larger* board, not SDA or SCL pull-ups are required. If you use the *smaller* board, you shall add those.</strike> This is clear in the Figures 1 and 2 or `[2]`.|
|This is **NOT ACCURATE** on the "PCB4109A" revision. There all the pull-ups are on the side of the *smaller* board.|

|*"EDIT got it working by pulling up the PWREN line. Tho this was not written in the datasheet of the satel and [...]"* <sub>[source](https://community.st.com/t5/interface-and-connectivity-ics/i-cannot-see-the-vl53l5cx-device-on-the-i2c-bus-i-m-tried-it/td-p/231586)</sub>|
|---|
|the Other Person had used "22K pull up to AVDD (5V)". That likely works, but so does pulling up to IOVDD. The schematic for "PCB4109A" [`[3]`]() shows a dimmed (not installed) 47k pull-up to IOVDD. Thus, that seems recommendable.|

**Other users**

[This example](https://github.com/stm32duino/VL53L5CX/blob/main/README.md) (Arduino library) has connected the pins like mentioned in the Nucleo document. But that seems overkill - we'd rather connect as few to active GPIO as possible (leaving that to the application engineer).

Based on [this ESP-focused library](https://github.com/RJRP44/VL53L5CX-Library) we should be able to get going with:

- pull-ups (2.2K) in both `SDA` and `SCL` *(only needed if using the smaller board)*
- pull-up (47K) in `LPn` (he's tied it up, but vendor docs demand a resistor)

In particular, the following had been left unconnected: `INT`, `I2C_RST`, `PWR_EN`

## Wiring multiple boards

*tbd. Do it!*

>Haven't tried this yet, but once I do, I will:
>
>- change boards 2..n I2C addresses by connecting them *individually* to the MCU (no playing with the `LPn` lines), and programming the new address in.
>- disabling the SDA & SCL pull-ups from all but one board, by removing `SB5`, `SB7` solder jumps (*tbd. image; backside of the smaller board*)

<!-- tbd. once done, edit the above -->


## References

- `[1]`: [VL53L5CX Product overview](https://www.st.com/resource/en/datasheet/vl53l5cx.pdf) (ST.com DS13754, Rev 12; April 2024)
- `[2]`: [How to setup and run the VL53L5CX-SATEL using an STM32 Nucleo64 board]() (ST.com AN5717, Rev 2; Dec 2021)
- `[3]`: [PCB Schematic VL53L5CX-SATEL](https://www.st.com/en/evaluation-tools/vl53l5cx-satel.html#cad-resources) (ST.com; Rev A, ver 012, 2021)

	The schematics of the SATEL board (PCB4109A).
