# DBC CAN Database File Format Specifications

## Overview

The DBC file describes the communication of a **single CAN network**. It contains all information required for monitoring, analysis, and remaining-bus simulation. It does **not** describe ECU functional behavior.

# DBC Reserved Keywords Glossary  

| Keyword                  | Purpose                                                                                             | Typical Location / Frequency                         |
|--------------------------|-----------------------------------------------------------------------------------------------------|-------------------------------------------------------|
| `VERSION`                | Human-readable version/revision of the DBC file                                                     | First non-empty line                                  |
| `NS_`                    | Lists which optional/new keywords appear in this file                                               | Right after `VERSION`                                 |
| `BS_`                    | Obsolete bit-timing section (must be present, usually empty)                                        | After `NS_`, before `BU_`                             |
| `BU_`                    | Lists all ECU/node names                                                                            | Before any `BO_` that references them                 |
| `BO_`                    | Defines a CAN message (frame)                                                                       | Main body                                             |
| `SG_`                    | Defines a signal belonging to the preceding `BO_`                                                   | Immediately after its `BO_`                           |
| `VAL_TABLE_`             | Declares a reusable enumeration table                                                               | Usually near top                                      |
| `VAL_`                   | Assigns value descriptions (enums) to a signal or `VAL_TABLE_`                                      | Anywhere after signal/table                           |
| `CM_`                    | Free-form comment for network, node, message, signal, or environment variable                       | Anywhere                                              |
| `BA_DEF_`                | Defines a custom attribute (name, type, range)                                                      | Usually grouped together                              |
| `BA_DEF_DEF_`            | Sets default value for a custom attribute                                                           | After `BA_DEF_`                                       |
| `BA_`                    | Assigns an attribute value to an object (network, message, signal, node)                            | Usually near end                                      |
| `BO_TX_BU_`              | Lists additional transmitters for a message                                                         | After messages                                        |
| `EV_`                    | Declares an environment variable (for remaining-bus simulation)                                    | Usually near end                                      |
| `ENVVAR_DATA_`           | Defines data-block size for an environment variable                                                 | After corresponding `EV_`                             |
| `SIG_GROUP_`             | Logical group of signals within one message (UI / CAPL scripting)                                   | After the relevant message                            |
| `SG_MUL_VAL_`            | Extended multiplexing – defines valid mux value ranges for a multiplexed signal                    | When complex multiplexing is needed                   |
| `VECTOR__XXX`            | Placeholder meaning "no node" / "don't care"                                                        | In `BU_`, transmitter, or receiver lists             |
| `VECTOR__INDEPENDENT_SIG_MSG` | Internal Vector construct for signals not attached to real messages                           | Rare, only in exported files                          |

> Rarely Seen / Effectively Obsolete Keywords (Still Valid but Avoid)

| Keyword                  | Purpose                                           | Status / Recommendation                              |
|--------------------------|---------------------------------------------------|------------------------------------------------------|
| `SGTYPE_` / `SIG_TYPE_REF_` | Reusable signal templates                      | Obsolete – ignored or rejected by most parsers       |
| `BA_DEF_REL_` / `BA_REL_`   | Relational attributes (node↔signal links)      | Extremely rare                                       |
| `CAT_DEF_` / `CAT_`         | Categories for messages/signals                | Very rare                                            |
| `SIG_VALTYPE_`              | Old way to declare integer vs float signals   | Obsolete since ~2008                                 |
| `BU_SG_REL_` / `BU_EV_REL_` | Legacy node-signal / node-env-var relations    | Legacy only                                          |

## File Structure

A DBC file consists of multiple sections, each defined by specific keywords. The file is line-based, with each statement typically on a single line. Comments start with `//` and extend to the end of the line.

Only `BU_`, `BO_`,`BS_`, `NS_`, and `SG_` are strictly required for a minimal database. `VERSION` is optional (if omitted, an empty version is assumed). Everything else augments metadata or tooling hints.

## Core Statements

### `VERSION`: Version

Human-readable version/revision of the DBC.

**Syntax:**
```text
VERSION "version_string";
```

**Examples:**
```text
VERSION "1.0";
VERSION "2023-01-15";
VERSION "v1.0.1";
VERSION "Land Rover Defender 2011";
VERSION "";
```

**Notes:**
- The version string is enclosed in double quotes
- VERSION is optional and the statement may be omitted entirely.
- This is typically the first line in a DBC file (when present)
- Empty version strings are allowed and represent "no version specified"

---

### `NS_`: New Symbols Statement

Declares which optional ("new") keywords are actually used in this DBC file.

**Example:**
```text
NS_ :
    NS_DESC_
    CM_
    BA_DEF_
    BA_
    VAL_
    CAT_DEF_
    CAT_
    FILTER
    BA_DEF_DEF_
    EV_DATA_
    ENVVAR_DATA_
    SGTYPE_
    SGTYPE_VAL_
    BA_DEF_SGTYPE_
    BA_SGTYPE_
    SIG_TYPE_REF_
    VAL_TABLE_
    SIG_GROUP_
    SIG_VALTYPE_
    SIGTYPE_VALTYPE_
    BO_TX_BU_
    BA_DEF_REL_
    BA_REL_
    BA_DEF_DEF_REL_
    BU_SG_REL_
    BU_EV_REL_
    BU_BO_REL_
    SG_MUL_VAL_;
```

**Requirements:**
| Rule                                      | Specification & Real-World Enforcement                                                                               |
|-------------------------------------------|----------------------------------------------------------------------------------------------------------------------|
| Order of keywords | No order required by the spec, but Vector tools always output them in exactly the order shown in the example above |
| Duplicates | Not allowed – each keyword may appear only once |
| Whitespace | One space after NS_, one space before and after the colon, each keyword indented with spaces or tabs (Vector uses spaces) |
| Mandatory? | Optional in the spec, but present in >99.9 % of all real DBC files since ~2005 because Vector CANdb++ always writes it |

### `BU_`: Bus Nodes

Defines the list of nodes (ECUs - Electronic Control Units) present on the CAN bus. Nodes are referenced as transmitters and receivers in messages and signals.

**Syntax:**
```text
BU_: <node1> <node2> <node3>;
```

**Example:**
```text
BU_: Engine Gearbox ABS Dashboard Vector__XXX
```

**Requirements:**
| Rule                                      | Specification & Real-World Enforcement                                                                               |
|-------------------------------------------|----------------------------------------------------------------------------------------------------------------------|
| Must be a valid C identifier              | Must start with a letter (A–Z, a–z) or underscore (_), followed only by letters, digits, and underscores             |
| Allowed characters                        | Only `A–Z`, `a–z`, `0–9`, `_` — **no spaces, hyphens, dots, or special characters**                                  |
| Maximum length                            | **32 characters** (hard limit in spec)                                                                               |
| Case-sensitive                            | Yes — `Engine` ≠ `ENGINE` ≠ `engine`                                                                                 |
| Uniqueness                                | **All node names in `BU_` must be unique**                                                                           |
| Reserved / special names                  | `Vector__XXX` = universal dummy node ("no node / don't care")<br>`VECTOR__INDEPENDENT_SIG_MSG` = internal pseudo-node|
| Recommended dummy node                    | Always use `Vector__XXX` as the last node in `BU_`                                                                   |
| Spaces, hyphens, other characters         | **Not allowed** in the official spec and rejected by all modern parsers                                              |
| Semicolon on `BU_` line                   | **Mandatory**                                                                                                        |

---

### `BO_`: Message Definition

Defines a CAN message (also called a frame).

**Syntax:**
```text
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
```text
BO_ 256 EngineData : 8 ECM
```

**Requirements:**
| Rule                                      | Specification & Real-World Enforcement                                                                                                |
|-------------------------------------------|---------------------------------------------------------------------------------------------------------------------------------------|
| Keyword                                   | Must start with exactly `BO_` (case-sensitive, space after)                                                                           |
| Message ID (`<message_id>`)               | Unsigned integer (0 – 2³²−1)  <br>• 11-bit IDs: 0 – 2047  <br>• 29-bit IDs: raw 29-bit value + 2³¹ (i.e. +2147483648) to set bit 31   |
| Message ID uniqueness                     | **Must be unique** across the entire file (11-bit and 29-bit IDs are in separate namespaces because of the +2³¹ rule)                 |
| Message Name (`<message_name>`)           | Valid DBC identifier: starts with letter or `_`, then only letters, digits, `_`  <br>Maximum 32 characters                            |
| Message Name uniqueness                   | **Must be unique** across the entire file                                                                                             |
| Colon separator                           | Exactly one colon `:` after the message name, **no spaces before or after**                                                           |
| Message Size / DLC                        | Unsigned integer representing number of data bytes  <br>• Classic and Extended CAN (CAN 2.0A, CAN 2.0B): 0 – 8  <br>• CAN FD: 0 – 64                        |
| Transmitter node                          | Must be a node name previously declared in `BU_` or the dummy node `Vector__XXX`                                                      |
| Semicolon                                 | **No semicolon** allowed at the end of the `BO_` line                                                                                 |
| Following lines                           | Zero or more `SG_` signal definitions **must immediately follow** the `BO_` line (no other keywords allowed in between)               |
| Whitespace                                | Exactly one space after `BO_`, after the ID, after the name, before and after the colon, and before the transmitter                   |
| Empty messages (DLC = 0)                  | Explicitly allowed                                                                                                                    |
| J1939 / extended multiplexing             | Handled via attributes and `SG_MUL_VAL_` (the `BO_` line itself follows the same rules)                                               |

---

### `SG_`: Signal Definition

Defines a signal within a message and must be defined within a message (after BO_). Signals represent individual data elements.

**Syntax:**
```text
 SG_ <Signal_Name> [multiplex]: <StartBit>|<Length>@<ByteOrder><Sign> (<Factor>,<Offset>) [<Min>|<Max>] "<Unit>" <Receivers>
```

**Parameters:**
- `<Signal_Name>`: Name of the signal
- `[Multiplex]`: Multiplex Indicator (optional)
- `<StartBit>`: Starting bit position (0-based, 0-63 for 8-byte messages)
- `<Length>`: Signal length in bits (1-64)
- `<ByteOrder>`: `0` = big-endian (Motorola), `1` = little-endian (Intel)
- `<Sign>`: `+` = unsigned, `-` = signed (two's complement)
- `<Factor>`: Scaling factor (multiplier for physical value conversion)
- `<Offset>`: Offset value (added after scaling)
- `<Min>`: Minimum physical value
- `<Max>`: Maximum physical value
- `<Unit>`: Physical unit string (e.g., "rpm", "°C", "V")
- `<Receivers>`: Space-separated list of receiving nodes (optional, can be empty or "*" for all)

**Physical Value Calculation:**
```text
physical_value = (raw_value * factor) + offset
```

**Example:**
```text
 SG_ RPM : 0|16@0+ (0.25,0) [0|8000] "rpm" ECM
 SG_ Temperature : 16|8@0- (1,-40) [-40|215] "°C" TCM
 SG_ Pressure : 0|16@1+ (0.1,0) [0|1000] "bar" *
```

**Requirements:**
| Rule                                      | Official Vector DBC Spec v1.0.5 (2010-04-12) – Exact Requirement                                                                                           |
|-------------------------------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Keyword                                   | Must start with exactly `SG_` followed by exactly one space                                                                                               |
| Signal Name                               | Valid DBC identifier: starts with letter or `_`, then only letters, digits, `_`  <br>Maximum 32 characters  <br>**Must be unique within the message**   |
| Multiplex Indicator (optional)            | After name, either nothing, or ` M` (multiplexer switch), or ` m<value>` (multiplexed signal, value = 0–2³²−2)                                           |
| Colon and space                           | Exactly ` : ` (space-colon-space) after name (or after multiplex indicator)                                                                               |
| Start Bit                                 | Unsigned integer 0–2047 (for classic CAN 8-byte frame)  <br>For CAN FD up to 64 bytes: 0–2047 still used (bit numbering stops at 64×8−1 = 511)             |
| Bit Length                                | `<Length>` – unsigned integer 1–64 (1–32 in classic CAN is typical, 64 allowed in CAN FD extensions)                                                    |
| Byte Order / Endianness                   | `@0` = Motorola (big-endian, MSB first)  <br>`@1` = Intel (little-endian, LSB first)                                                                     |
| Value Type (signed/unsigned)              | `+` = unsigned  <br>`-` = signed                                                                                                                         |
| Factor & Offset                           | `(<Factor>,<Offset>)` – both are `double` values (floating-point allowed)  <br>Factor ≠ 0 required                                                                |
| Minimum | Maximum (optional)                     | `[min|max]` – both `double`, `min` ≤ `max`  <br>If omitted, defaults to physical min/max of the raw type                                              |
| Unit                                      | `"unit"` – string in double quotes, max 32 chars, printable ASCII except `"` and `\`                                                                      |
| Receiver List                             | One or more node names from `BU_` or `Vector__XXX`, space-separated  <br>Use `*` for broadcast to all nodes                                                  |
| No semicolon                              | **No semicolon** allowed at the end of the `SG_` line                                                                                                     |
| Must immediately follow its `BO_`        | `SG_` lines **must** directly follow their parent `BO_` line with no other keywords in between                                                          |
| Signal bit layout rules                   | • Start bit + length − 1 ≤ 64×8−1 (511 for 64-byte CAN FD)  <br>• No overlapping bits allowed within the same message (except multiplexed signals)       |
| Multiplexed signals rules                 | • Exactly one signal with ` M` (the multiplexer switch) per message  <br>• Multiplexed signals (` m<number>`) must have unique multiplex values in their range |
| Extended multiplexing (SG_MUL_VAL_)       | Optional additional keyword to define valid ranges for multiplexed signals (does not affect `SG_` line syntax itself)                                     |
| Whitespace                                | Exactly one space after `SG_`, after name/multiplex, before and after `:`, before and after `|`, `@`, `(`, `)`, `[`, `]`, and between receiver node names |

---

### `BS_`: Bit Timing

`BS_` captures optional default bit-timing parameters (baud rate, sample point, etc.) for the CAN network. Most third-party tools leave the section empty:

**Syntax:**
```text
BS_:
```

When populated it follows `BS_ <Baudrate> : <SamplePoint>` but support is tool-specific.

### `VAL_` and `VAL_TABLE`: Value Tables

Defines named values for signals (enum-like definitions).

**Syntax:**
```text
VAL_ <Message_ID> <Signal_Name> <Value1> "<Description1>" <Value2> "<Description2>";
```

**Example:**
```text
VAL_TABLE_ Gear 0 "P" 1 "R" 2 "N" 3 "D" 4 "1" 5 "2";

VAL_ 256 GearSelector Gear;   // references table
```

---

### `CM_`: Comments

Adds comments to messages, signals, or the entire database.

**Syntax:**
```text
CM_ BO_ <Message_ID> "<Comment>";
CM_ SG_ <Message_ID> <Signal_Name> "<Comment>";
CM_ "<Comment>";
```

**Example:**
```text
CM_ "Vehicle XYZ database";
CM_ BU_ Engine "Main engine controller";
CM_ BO_ 256 "Engine status";
CM_ SG_ 256 RPM "Engine speed";
```

---

### `BA_`: Attribute Definitions

Defines custom attributes for messages, signals, nodes, or the database.

**Syntax:**
```text
BA_DEF_ <Object> "<AttributeName>" <Type> <Value>;
BA_ "<AttributeName>" <Value> <Object> <ID>;
```

**Example:**
```text
BA_DEF_ "GenMsgCycleTime" INT 0 2000;
BA_DEF_ BO_ "GenMsgSendType" ENUM "Cyclic","OnChange","None";
BA_DEF_ "BC_" STRING;

BA_DEF_DEF_ "GenMsgCycleTime" 100;
BA_DEF_DEF_ "BC_" "500 kbit/s";

BA_ "GenMsgCycleTime" BO_ 256 50;
BA_ "BC_" "500 kbit/s";
```

---

### `EV_`: Environment Variables

Defines environment variables (used in some CAN tools).

**Syntax:**
```text
EV_ <Name> : <Type> [ <Min> | <Max> ] <Unit> <InitialValue> <ID> <AccessType> <AccessNodes> ;
```

---

### `VAL_TABLE_`: Reusable Enumerations

Defines a named lookup table that can later be referenced by multiple signals via attributes or tooling. Format matches `VAL_` lines but ends with a semicolon:

```text
VAL_TABLE_ StatusTable 0 "Inactive" 1 "Active" 2 "Error" ;
```

###  `SIG_GROUP_`: Signal Grouping

Groups one or more signals under a label so visualization tools can show related signals together:

```text
SIG_GROUP_ 256 Powertrain 1 RPM Torque ThrottlePosition;
```

`256` is the parent message ID, `Powertrain` the group name, `1` the repetitions count (mostly unused), followed by signal names.

### `SIG_VALTYPE_`: Value Type Overrides

Overrides the underlying data type used for transmitting a signal (e.g., 16-bit float):

```text
SIG_VALTYPE_ 256 RPM : 1;
```

Common codes: `0 = unsigned`, `1 = IEEE float`, `2 = double`, etc. See Vector documentation for the exhaustive list.

### `BA_DEF_` and `BA_`: Attribute Targets

`BA_DEF_`/`BA_` lines use object prefixes to scope attributes:

| Prefix | Applies To                  |
|--------|-----------------------------|
| `BU_`  | Nodes                       |
| `BO_`  | Messages                    |
| `SG_`  | Signals                     |
| `EV_`  | Environment variables       |
| `BA_` (no prefix) | Database-wide   |

Attributes can be of type `INT`, `HEX`, `FLOAT`, `STRING`, or enumeration. Defaults are provided via `BA_DEF_DEF_`.


## Complete Example

```text
VERSION "1.0";

NS_ :
    NS_DESC_ CM_ BA_DEF_ BA_ VAL_ CAT_DEF_ CAT_ FILTER
    BA_DEF_DEF_ EV_DATA_ ENVVAR_DATA_ SGTYPE_ SGTYPE_VAL_
    BA_DEF_SGTYPE_ BA_SGTYPE_ SIG_TYPE_REF_ VAL_TABLE_
    SIG_GROUP_ SIG_VALTYPE_ SIGTYPE_VALTYPE_ BO_TX_BU_
    BA_DEF_REL_ BA_REL_ BA_DEF_DEF_REL_ BU_SG_REL_
    BU_EV_REL_ BU_BO_REL_ SG_MUL_VAL_;

BS_:

BU_: Engine Dashboard Gearbox

BA_DEF_ "BC_" STRING;
BA_DEF_DEF_ "BC_" "500 kbit/s";
BA_ "BC_" "500 kbit/s";

BO_ 256 EngineStatus: 8 Engine
 SG_ RPM : 0|16@1+ (0.125,0) [0|8031.875] "rpm" Dashboard
 SG_ Temp : 16|8@1- (1,-40) [-40|215] "°C" Dashboard

VAL_ 256 Gear 0 "P" 1 "R" 2 "N" 3 "D";

CM_ BO_ 256 "Engine status message";
CM_ SG_ 256 RPM "Current engine speed";
```

