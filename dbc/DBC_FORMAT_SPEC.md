# DBC CAN Database File Format Specifications

## Overview

The DBC (CAN Database) file format is a text-based format used to describe the structure and interpretation of messages and signals transmitted over a Controller Area Network (CAN) bus. It is widely used in automotive and industrial applications to define how raw CAN data should be decoded into human-readable values.

The DBC format was developed by **Vector Informatik GmbH** and has remained relatively stable over time. The primary reference document is **"DBC File Format Documentation Version 01/2007"** from Vector Informatik.

## Format Versions

**There is no formal "version 2" (v2) of the DBC specification.** The format has remained consistent since its introduction, with the main reference being the 2007 documentation from Vector Informatik.

### Version String vs. Format Version

The `VERSION` statement in a DBC file (e.g., `VERSION "1.0"`) refers to the **database version** (a user-defined version string for the specific database file), not the DBC file format specification version. Different DBC files may have different version strings, but they all follow the same format specification.

### Extensions and Variants

While there is no v2 specification, the DBC format has been extended and adapted for specific protocols:

- **J1939 DBC**: Extensions for heavy-duty vehicle protocols (SAE J1939)
- **NMEA 2000**: Adaptations for marine electronics
- **OBD2**: Extensions for On-Board Diagnostics
- **ARXML**: A more modern XML-based format (not DBC, but related)

These are protocol-specific extensions rather than a new version of the DBC format itself.

## Keyword Glossary

| Statement      | Purpose                                      | Typical Location                    |
|----------------|----------------------------------------------|-------------------------------------|
| `VERSION`      | Declares the logical revision of the file    | First non-comment line (optional)   |
| `NS_`          | Enumerates which optional keywords appear    | Immediately after `VERSION` (optional) |
| `BS_`          | Bit-timing defaults for CAN bus              | Optional, usually before `BU_`      |
| `BU_`          | Lists ECU/node names                         | Before messages that reference them |
| `BO_`          | Defines a CAN frame (message)                | Throughout body                     |
| `SG_`          | Defines a signal that belongs to a `BO_`     | Immediately after its parent `BO_`  |
| `VAL_TABLE_`   | Declares reusable enum tables                | Anywhere (commonly near top)        |
| `VAL_`         | Binds enum values to a signal                | Anywhere after the referenced signal |
| `CM_`          | Attaches free-form comments                  | Anywhere                            |
| `BA_DEF_`/`BA_`| Defines / assigns custom attributes          | Anywhere                            |
| `SIG_GROUP_`   | Groups signals for tooling/UI                | After associated message            |
| `SIG_VALTYPE_` | Sets integer/float encoding per signal       | After signals                       |
| `EV_`          | Declares environment variables               | Anywhere                            |

Only `BU_`, `BO_`, and `SG_` are strictly required for a minimal database. `VERSION` is optional (if omitted, an empty version is assumed). Everything else augments metadata or tooling hints.

## File Structure

A DBC file consists of multiple sections, each defined by specific keywords. The file is line-based, with each statement typically on a single line. Comments start with `//` and extend to the end of the line.

## Core Statements

### 1. VERSION

Defines the version string of the DBC file.

**Syntax:**
```
VERSION "version_string"
```

**Example:**
```
VERSION "1.0"
VERSION ""  # Empty version string (allowed, represents "no version specified")
```

**Notes:**
- The version string is enclosed in double quotes
- **VERSION is optional**: The `VERSION` statement may be omitted entirely. If omitted, parsers typically assume an empty version (represented as `VERSION ""`)
- This is typically the first line in a DBC file (when present)
- **Empty version strings are allowed**: `VERSION ""` is valid and represents "no version specified" (per DBC format specification)
- When empty, the version is typically represented internally as `0` but should be output as an empty string to preserve the original format

---

### 2. BU_ (Bus Nodes)

Defines the list of nodes (ECUs - Electronic Control Units) present on the CAN bus.

**Syntax:**
```
BU_: node1 node2 node3 ...
```

**Example:**
```
BU_: ECM TCM BCM ABS
```

**Notes:**
- Nodes are space-separated
- Each node name should be unique
- Nodes are referenced as transmitters and receivers in messages and signals

**Node Naming Requirements:**
- Uniqueness: Node names should be unique within a DBC file (recommended, not strictly enforced by all parsers)
- Case Sensitivity: Node names are case-sensitive (e.g., `ECM` and `ecm` are considered different nodes)
- Character Restrictions: No specific character restrictions are defined in the DBC format specification. Node names typically use:
  - Alphanumeric characters (A-Z, a-z, 0-9)
  - Underscores (`_`)
  - Common practice: Uppercase abbreviations (e.g., `ECM`, `TCM`, `BCM`)
- Length: No maximum length is specified in the DBC format specification
- Format: No specific format requirements (PascalCase, UPPERCASE, snake_case, etc. are all acceptable)
- **Examples of Valid Node Names**:
  - Short uppercase: `ECM`, `TCM`, `BCM`, `ABS`
  - With numbers: `ECU1`, `ECU2`, `NODE1`
  - With underscores: `ECU_A`, `ECU_B`
  - Mixed case: `Gateway`, `Sensor`, `Actuator`
  - All uppercase: `GATEWAY`, `SENSOR`, `ACTUATOR`

---

### 3. BO_ (Message Definition)

Defines a CAN message (also called a frame).

**Syntax:**
```
BO_ <CAN_ID> <Message_Name> : <DLC> <Transmitter>
```

**Parameters:**
- `<CAN_ID>`: The CAN identifier (decimal, typically 0-2047 for 11-bit or 0-536870911 for 29-bit)
- `<Message_Name>`: Name of the message (alphanumeric, may contain underscores)
- `<DLC>`: Data Length Code (1-64 bytes)
  - **Classic CAN Standard (CAN 2.0A)**: 1-8 bytes (64 bits maximum payload)
  - **Classic CAN Extended (CAN 2.0B)**: 1-8 bytes (64 bits maximum payload)
  - **CAN FD (Flexible Data Rate, ISO/Bosch)**: 1-64 bytes (512 bits maximum payload)  
- `<Transmitter>`: Node name that transmits this message (must be in BU_ list)

**Example:**
```
BO_ 256 EngineData : 8 ECM
```

**Notes:**
- Messages are followed by signal definitions (SG_)
- CAN IDs should be unique within a DBC file
- DLC must be between 1 and 64 bytes (supports CAN 2.0A, CAN 2.0B, and CAN FD)
- All signals must fit within the message boundary: `DLC * 8 bits`

---

### 4. SG_ (Signal Definition)

Defines a signal within a message. Signals represent individual data elements.

**Syntax:**
```
 SG_ <Signal_Name> : <StartBit>|<Length>@<ByteOrder><Sign> (<Factor>,<Offset>) [<Min>|<Max>] "<Unit>" <Receivers>
```

**Parameters:**
- `<Signal_Name>`: Name of the signal
- `<StartBit>`: Starting bit position (0-based, 0-63 for 8-byte messages)
- `<Length>`: Signal length in bits (1-64)
- `<ByteOrder>`: `0` = little-endian (Intel), `1` = big-endian (Motorola)
- `<Sign>`: `+` = unsigned, `-` = signed (two's complement)
- `<Factor>`: Scaling factor (multiplier for physical value conversion)
- `<Offset>`: Offset value (added after scaling)
- `<Min>`: Minimum physical value
- `<Max>`: Maximum physical value
- `<Unit>`: Physical unit string (e.g., "rpm", "°C", "V")
- `<Receivers>`: Space-separated list of receiving nodes (optional, can be empty or "*" for all)

**Physical Value Calculation:**
```
physical_value = (raw_value * factor) + offset
```

**Example:**
```
 SG_ RPM : 0|16@0+ (0.25,0) [0|8000] "rpm" ECM
 SG_ Temperature : 16|8@0- (1,-40) [-40|215] "°C" TCM
 SG_ Pressure : 0|16@1+ (0.1,0) [0|1000] "bar" *
```

**Notes:**
- Signals must be defined within a message (after BO_)
- Start bit and length determine signal position in the message
- Byte order affects how multi-byte signals are interpreted (Intel counts upward through bytes, Motorola counts downward)
- Signed signals use two's complement representation
- The receiver list can be omitted, empty, or "*" for broadcast
- Start-bit numbering follows the Vector convention: for little-endian signals, bit 0 is the LSB of byte 0; for big-endian signals, bit 0 is the MSB of byte 0 (which causes bit positions to "move left" across bytes).

#### Receiver semantics
- `*` (asterisk) = broadcast to all nodes.
- Explicit list (`ECM BCM`) = only those consumers.
- Empty / omitted = tooling dependent; often interpreted as unknown receivers.

---

## Extended Statements (Optional)

### 4.5 NS_ (New Symbols)

The `NS_` block simply enumerates which optional statements can appear in the remainder of the file. Many generators include the full canonical list regardless of actual usage.

```
NS_ :
    CM_
    BA_DEF_
    BA_
    VAL_
    ...
```

The block must end with a blank line. Parsers generally ignore unknown entries.

### 4.6 BS_ (Bit Timing)

`BS_` captures optional default bit-timing parameters (baud rate, sample point, etc.) for the CAN network. Most third-party tools leave the section empty:

```
BS_:
```

When populated it follows `BS_ <Baudrate> : <SamplePoint>` but support is tool-specific.

### 5. VAL_ (Value Descriptions)

Defines named values for signals (enum-like definitions).

**Syntax:**
```
VAL_ <Message_ID> <Signal_Name> <Value1> "<Description1>" <Value2> "<Description2>" ... ;
```

**Example:**
```
VAL_ 256 GearPosition 0 "Park" 1 "Reverse" 2 "Neutral" 3 "Drive" 4 "Sport" ;
```

---

### 6. CM_ (Comments)

Adds comments to messages, signals, or the entire database.

**Syntax:**
```
CM_ BO_ <Message_ID> "<Comment>";
CM_ SG_ <Message_ID> <Signal_Name> "<Comment>";
CM_ "<Comment>";
```

**Example:**
```
CM_ BO_ 256 "Engine control message";
CM_ SG_ 256 RPM "Engine revolutions per minute";
```

---

### 7. BA_ (Attribute Definitions)

Defines custom attributes for messages, signals, nodes, or the database.

**Syntax:**
```
BA_DEF_ <Object> "<AttributeName>" <Type> <Value>;
BA_ "<AttributeName>" <Value> <Object> <ID>;
```

**Example:**
```
BA_DEF_ BO_ "GenMsgCycleTime" INT 0 65535;
BA_ "GenMsgCycleTime" 100 BO_ 256;
```

---

### 8. EV_ (Environment Variables)

Defines environment variables (used in some CAN tools).

**Syntax:**
```
EV_ <Name> : <Type> [ <Min> | <Max> ] <Unit> <InitialValue> <ID> <AccessType> <AccessNodes> ;
```

---

---

### 9. VAL_TABLE_ (Reusable Enumerations)

Defines a named lookup table that can later be referenced by multiple signals via attributes or tooling. Format matches `VAL_` lines but ends with a semicolon:

```
VAL_TABLE_ StatusTable 0 "Inactive" 1 "Active" 2 "Error" ;
```

### 10. SIG_GROUP_ (Signal Grouping)

Groups one or more signals under a label so visualization tools can show related signals together:

```
SIG_GROUP_ 256 Powertrain 1 RPM Torque ThrottlePosition;
```

`256` is the parent message ID, `Powertrain` the group name, `1` the repetitions count (mostly unused), followed by signal names.

### 11. SIG_VALTYPE_ (Value Type Overrides)

Overrides the underlying data type used for transmitting a signal (e.g., 16-bit float):

```
SIG_VALTYPE_ 256 RPM : 1;
```

Common codes: `0 = unsigned`, `1 = IEEE float`, `2 = double`, etc. See Vector documentation for the exhaustive list.

### 12. SIG_TYPE_REF_ / SGTYPE_

These statements reference reusable signal templates (`SGTYPE_`) and bind them to concrete signals (`SIG_TYPE_REF_`). They are rarely seen outside auto-generated databases but allow enforcing consistent scaling/unit definitions across multiple messages.

### 13. Attribute Targets

`BA_DEF_`/`BA_` lines use object prefixes to scope attributes:

| Prefix | Applies To                  |
|--------|-----------------------------|
| `BU_`  | Nodes                       |
| `BO_`  | Messages                    |
| `SG_`  | Signals                     |
| `EV_`  | Environment variables       |
| `BA_` (no prefix) | Database-wide   |

Attributes can be of type `INT`, `HEX`, `FLOAT`, `STRING`, or enumeration. Defaults are provided via `BA_DEF_DEF_`.

## File Format Rules

1. **Line Format:**
   - Each statement typically occupies one line
   - Statements can span multiple lines (rare)
   - Whitespace is generally ignored except within quoted strings

2. **Comments:**
   - Single-line comments start with `//`
   - Comments extend to the end of the line
   - Empty lines are ignored

3. **Case Sensitivity:**
   - Keywords (BO_, SG_, etc.) are case-sensitive
   - Node names and signal names are case-sensitive
   - Unit strings are case-sensitive

4. **Ordering:**
   - VERSION should come first
   - BU_ should come early (before messages that reference nodes)
   - Messages (BO_) and their signals (SG_) should be grouped together
   - Extended statements (VAL_, CM_, BA_) can appear anywhere

5. **Identifiers:**
   - CAN IDs must be unique
   - Signal names must be unique within a message
   - Node names should be unique

## Complete Example

```
VERSION "1.0"

BU_: ECM TCM BCM

BO_ 256 EngineData : 8 ECM
 SG_ RPM : 0|16@0+ (0.25,0) [0|8000] "rpm" TCM
 SG_ Temperature : 16|8@0- (1,-40) [-40|215] "°C" TCM BCM
 SG_ ThrottlePosition : 24|8@0+ (0.392157,0) [0|100] "%" *

BO_ 512 BrakeData : 4 TCM
 SG_ BrakePressure : 0|16@1+ (0.1,0) [0|1000] "bar" ECM BCM
 SG_ ABSActive : 16|1@0+ (1,0) [0|1] "" *

CM_ BO_ 256 "Primary engine control message";
CM_ SG_ 256 RPM "Engine speed in revolutions per minute";

VAL_ 512 ABSActive 0 "Inactive" 1 "Active" ;
```

## Byte Order (Endianness)

### Little-Endian (Intel) - `@0`
- Least significant byte first
- Bits numbered from LSB to MSB within bytes
- Example: Signal at bits 0-15 spans bytes 0-1, with bit 0 in byte 0. Bit 15 corresponds to the MSB of byte 1.

### Big-Endian (Motorola) - `@1`
- Most significant byte first
- Bits numbered from MSB to LSB across bytes. Within the same byte, numbering counts down; crossing a byte boundary continues into the next lower address.
- Example: A 12-bit signal starting at bit 3 spans bytes 0-1; bit positions appear non-linear because of the descending count.

## Signal Encoding

### Unsigned Signals (`+`)
- Raw value is interpreted as unsigned integer
- Physical value = `raw * factor + offset`

### Signed Signals (`-`)
- Raw value is interpreted as signed integer (two's complement)
- Physical value = `raw * factor + offset`
- For a signal of length `n` bits, values range from `-2^(n-1)` to `2^(n-1)-1`
- For signed big-endian signals, the sign bit is still the MSB of the encoded field (e.g., start bit 3, length 12 ⇒ bit 3 is sign).

## Implementation Notes

The current `dbc-rs` implementation supports:
- ✅ VERSION parsing and writing
- ✅ BU_ (nodes) parsing and writing
- ✅ BO_ (messages) parsing and writing
- ✅ SG_ (signals) parsing and writing with:
  - Start bit and length
  - Factor and offset
  - Min and max values
  - Units
  - Byte order and sign (fully supported - parsed, stored, and written correctly)
  - Signal receivers (fully supported - parsed, stored, and written correctly)
  - 29-bit extended CAN IDs (fully supported and validated)

**Not yet implemented:**
- ❌ VAL_ (value descriptions)
- ❌ CM_ (structured comments)
- ❌ BA_ (attributes)
- ❌ EV_ (environment variables)
- ❌ Signal multiplexing

## References

- **Vector Informatik GmbH**: "DBC File Format Documentation Version 01/2007" (official reference)
- Vector CANdb++ Documentation
- CAN Specification (ISO 11898)
- Various DBC file format documentation from automotive tool vendors

**Note**: For the most accurate and up-to-date information, consult official documentation from Vector Informatik GmbH or authorized distributors. The format specification has remained stable since 2007, with no formal version 2 released.

