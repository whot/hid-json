# JSON Format

Note that "data" fields can be skipped with the `--skip-data` argument.

- All HID-specific names use a CamelCase naming convention.
- The JSON file format version follows semver conventions

```json
{
  "version": "1.0",       // JSON File format semver string
  "descriptor": {         // Information about the report descriptor itself
    "length": 93,         // Report Descriptor length in bytes
    "data": [1, 2, ...]   // Report Descriptor data bytes
  },
  "items": [              // Item details
    {
      "offset": 0,        // Item offset in the report descriptor
      "data": [1, 2],     // Item data bytes (including prefix byte)
      "type": "Global",   // Item type (Global/Local/Main)
      "name": "UsagePage",// Item name
      "value": 13         // *optional*: Item data bytes converted to i32
    },
    ...
    // Some items have extra fields for easier parsing
    {
      "offset": 10,
      "data": [ 161, 1 ],
      "type": "Main",
      "name": "Collection",
      "value": 1,
      "collection": "Application"
    },
}
```

An item `value` is provided as i32 for convenience but is susceptible to i32/u32
conversion issues - for some items it is not possible to distinguish between -1
and u32-max without looking at other items.
Where this is a concern a caller should parse the item data bytes.
For zero-length items (e.g. `Pop` or `EndCollection`) the value is not provided.

The item `data` always includes the item designator bytes, i.e. it is always
of size 1. The HID Specification requires item lengths of 0, 1, 2, or 4, the
item `data` may thus be of length 1, 2, 3, or 5.

## Extra fields

As shown above, some items include extra fields for convenience.

**These fields are advisory only and should not be relied upon for correctness**

- a `Collection` application item may include the named type of the collection:
  `Physical`, `Logical`, `Application`.
- a `UsagePage` item may include the named Usage Page, if any.
- a `Usage` item may include the named Usage, if any.
