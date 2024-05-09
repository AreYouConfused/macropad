# QMK macropad assistant

this software is paired with raw HID on a qmk keyboard (see [here](https://github.com/AreYouConfused/qmk_firmware/tree/ayc_doio_16/keyboards/doio/kb16/rev2/keymaps/areyouconfused) for an example on the qmk firmware side)

basic concept is using RawHID to bypass keyboard input to allow arbitrary command to be run on the host PC without sending input or take up keyboard keys,
