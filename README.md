# hid-json

hid-json is a utility that can take a HID Report descriptor from various sources
and converts it into JSON format split into the HID items.

See the [JSON_FORMAT.md] document for a description of the output.


## Usage

```
$ hid-json /sys/class/hidraw/hidraw1/device/report_descriptor
```
See the `--help` output for more options.

