# DMX Outputs

The `output_dmx` plugin serves as the DMX universe renderer and transport manager.
It's primary job is to render lighting values to DMX and make sure the final universe
values get output to their appropriate transport plugins.

## What is rendering?

Rendering is the process of taking abstract lighting values along with patch settings,
and converting it into the final values that should be sent to the lights. These values
take the form of an array of 512 8-bit unsigned integers (represented in rust as `u8`).

In doing so, SimplyDMX might see that a light has a 16-bit RGB color value of
`R:65535, G:12850, B:0` (orange), with an intensity of 50%. SimplyDMX can see that this
hypothetical light does not understand the concept of intensity, and simulate it. The
light also allows for 16-bit values, so the final output will be in 16-bit DMX.
`65535 * 0.5 = 32767.5`, `12850 * 0.5 = 6425`, and `0 * 0.5 = 0`. SimplyDMX also knows
that the light takes the values in RGB format, in that order, and starts at ID 73 (1-indexed).
Therefore, the output would be `[ 72 zero values, 127, 255, 25, 25, 0, 0, 435 zero values ]`.
This is because 32767 split into 2 8-bit slots becomes `[127, 255]`, 6425 becomes `[25, 25]`,
and 0 becomes `[0, 0]`. The operation for this in code is as follows:

```
let int1: u8 = (value16 & 0b1111111100000000 >> 8) as u8;
let int2: u8 = (value16 & 0b0000000011111111     ) as u8;
let output = [int1, int2]; // [MSB, LSB]
```
