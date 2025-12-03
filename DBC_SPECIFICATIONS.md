# DBC File Format Documentation

# Introduction

A DBC file describes the communication protocol for a single Controller Area Network (CAN). The information contained within enables network monitoring, traffic analysis, and simulation of electronic control units (ECUs) that are not physically present on the network. This simulation capability is known as "remaining bus simulation."

DBC files also serve as a foundation for developing communication software for ECUs that will participate in the CAN network. It is important to note that DBC files describe only the communication layer—they do not specify the internal functional behavior or application logic of the ECUs.

## General Definitions

The following general elements are used in this documentation:

| Element              | Description                                                                                                                                    |
|----------------------|------------------------------------------------------------------------------------------------------------------------------------------------|
| `unsigned_integer`   | an unsigned integer                                                                                                                            |
| `signed_integer`     | a signed integer                                                                                                                              |
| `double`             | a double precision float number                                                                                                                |
| `Printable character`| One of the characters 0x20 - 0x7E in the ASCII Code. I.e. the space character (0x20) is also considered as a printable character.              |
| `char_string`        | an arbitrary string consisting of any printable characters except double quotes (`"`) and backslashes (`\`). Control characters like Line Feed, Horizontal Tab, etc. are tolerated, but their interpretation depends on the application. |
| `C_identifier`      | a valid C_identifier. C_identifiers have to start with an alpha character or an underscore and may further consist of alpha-numeric characters and underscores. `C_identifier = (alpha_char \| '_') {alpha_num_char \| '_'}` |
| `DBC_identifier`    | a C_identifier which doesn't represent a DBC Keyword.                                                                                          |

```bnf
DBC-Keyword = 'VERSION' | 'NS_' | 'NS_DESC_' | 'CM_' | 'BA_DEF_' | 'BA_' | 'VAL_' | 'CAT_DEF_' | 'CAT_' | 'FILTER' | 'BA_DEF_DEF_' | 'EV_DATA_' | 'ENVVAR_DATA_' | 'SGTYPE_' | 'SGTYPE_VAL_' | 'BA_DEF_SGTYPE_' | 'BA_SGTYPE_' | 'SIG_TYPE_REF_' | 'VAL_TABLE_' | 'SIG_GROUP_' | 'SIG_VALTYPE_' | 'SIGTYPE_VALTYPE_' | 'BO_TX_BU_' | 'BA_DEF_REL_' | 'BA_REL_' | 'BA_DEF_DEF_REL_' | 'BU_SG_REL_' | 'BU_EV_REL_' | 'BU_BO_REL_' | 'SG_MUL_VAL_' | 'BS_' | 'BU_' | 'BO_' | 'SG_' | 'EV_' | 'VECTOR__INDEPENDENT_SIG_MSG' | 'VECTOR__XXX'
```

DBC-identifiers used in DBC files may have a length of up to 32 characters.

Other strings used in DBC files may be of an arbitrary length (do note that for security reasons, dbc-rs does limit the string length dependent on the objects).

The following table lists the primary DBC keywords that identify different object types within a DBC file:

| DBC-Keyword | Object Type |
| :-- | :-- |
| `BU_` | Network Node |
| `BO_` | Message |
| `SG_` | Signal |
| `EV_` | Environment Variable |
| `SIG_GROUP_` | Signal Group |
| `VAL_TABLE_` | Value Table |

All syntax definitions in this specification use extended BNF notation ([Extended Backus-Naur Form documentation](https://en.wikipedia.org/wiki/Extended_Backus%E2%80%93Naur_form)).

| Symbol      | Meaning                                                                              |
|-------------|--------------------------------------------------------------------------------------|
| `=`         | A name on the left of the `=` is defined using the syntax on the right (syntax rule).|
| `;`         | The semicolon terminates a definition.                                               |
| `\|`        | The vertical bar indicates an alternative.                                           |
| `[...]`     | The definitions within brackets are optional (zero or one occurrence).               |
| `{...}`     | The definitions within braces are repeated (zero or multiple occurrences).           |
| `(...)`     | Parentheses define grouped elements.                                                 |
| `"..."`     | Text in quotes has to appear as defined.                                             |
| `(* ... *)` | Comment.                                                                             |

# Structure of the DBC File

The DBC file format has the following overall structure:

```bnf
DBC_file =
    version
    new_symbols
    bit_timing (*obsolete but required*)
    nodes
    value_tables
    messages
    message_transmitters
    environment_variables
    environment_variables_data
    signal_types
    comments
    attribute_definitions
    sigtype_attr_list
    attribute_defaults
    attribute_values
    value_descriptions
    category_definitions (*obsolete*)
    categories (*obsolete*)
    filter (*obsolete*)
    signal_type_refs
    signal_groups
    signal_extended_value_type_list
    extended_multiplexing ;
```

DBC files describing the basic communication of a CAN network include the following sections:
- **Bit_timing**: This section is required but is normally empty.
- **nodes**: This section is required and defines the network nodes.
- **messages**: This section defines the messages and the signals.

The following sections aren't used in normal DBC files. They are defined here for the sake of completeness only:
- **signal_types**
- **sigtype_attr_list**
- **category_definitions**
- **categories**
- **filter**
- **signal_type_refs**
- **signal_extended_value_type_list**

DBC files that focus solely on CAN communication without additional data for system or remaining bus simulation typically omit environment variables.

--- 

# Version and New Symbol Specification

## Version

```bnf
version = ['VERSION' '"' { CANdb_version_string } '"' ] ;
```

**Notes:**
- The `VERSION` statement is optional. If omitted, parsers typically assume an empty version (represented as `VERSION ""`).
- The version string is enclosed in double quotes and may be empty.
- Empty version strings (`VERSION ""`) are valid and represent "no version specified".

## New Symbols

```bnf
new_symbols = [ '_NS' ':' ['CM_'] ['BA_DEF_'] ['BA_'] ['VAL_']
['CAT_DEF_'] ['CAT_'] ['FILTER'] ['BA_DEF_DEF_'] ['EV_DATA_']
['ENVVAR_DATA_'] ['SGTYPE_'] ['SGTYPE_VAL_'] ['BA_DEF_SGTYPE_']
['BA_SGTYPE_'] ['SIG_TYPE_REF_'] ['VAL_TABLE_'] ['SIG_GROUP_']
['SIG_VALTYPE_'] ['SIGTYPE_VALTYPE_'] ['BO_TX_BU_']
['BA_DEF_REL_'] ['BA_REL_'] ['BA_DEF_DEF_REL_'] ['BU_SG_REL_']
['BU_EV_REL_'] ['BU_BO_REL_'] ['SG_MUL_VAL_'] ];
```

# Bit Timing Definition

The bit timing section defines the baudrate and the settings of the BTR registers of the network. This section is obsolete and no longer used. Nevertheless, the keyword `BS_` must appear in the DBC file.

```bnf
bit_timing = 'BS_:' [baudrate ':' BTR1 ',' BTR2 ] ;
baudrate = unsigned_integer ;
BTR1 = unsigned_integer ;
BTR2 = unsigned_integer ;
```

**Notes:**
- The `BS_` keyword is **required** but the section is typically empty (`BS_:`).
- This section is obsolete and no longer used in modern DBC files.
- The baudrate and BTR values are ignored by most parsers.

# Node Definitions

The node section defines the names of all nodes that participate in the CAN network. All node names defined in this section must be unique.

```bnf
nodes = 'BU_:' {node_name} ;
node_name = DBC_identifier ;
```

**Notes:**
- The `BU_` section is **required** in a DBC file.
- Node names are separated by whitespace.
- All node names must be unique within the DBC file.
- Node names are case-sensitive.

# Value Table Definitions

The value table section defines global value tables. Value descriptions in these tables map signal raw values to human-readable text encodings. In commonly used DBC files, global value tables are typically not used; instead, value descriptions are defined independently for each signal.

```bnf
value_tables = {value_table} ;
value_table = 'VAL_TABLE_' value_table_name {value_description} ';' ;
value_table_name = DBC_identifier ;
```

## Value Descriptions (Value Encodings)

A value description maps a single numeric value to a human-readable textual description. The value may be either a signal raw value transmitted on the CAN bus or the value of an environment variable used in remaining bus simulation.

```bnf
value_description = unsigned_integer char_string ;
```

# Message Definitions

The message section defines all CAN frames (messages) in the network, including their properties and the signals contained within each frame.

```bnf
messages = {message} ;
message = BO_ message_id message_name ':' message_size transmitter {signal} ;
message_id = unsigned_integer ;
```
The message's CAN identifier (CAN-ID) must be unique within the DBC file. If the most significant bit of the CAN-ID is set, the ID represents an extended CAN ID. The extended CAN ID value can be determined by masking out the most significant bit using the mask `0x7FFFFFFF`.

```bnf
message_name = DBC_identifier ;
```

All message names defined in this section must be unique across all messages in the DBC file.

```bnf
message_size = unsigned_integer ;
```
The `message_size` field specifies the data length of the message in bytes (Data Length Code, or DLC).

```bnf
transmitter = node_name | 'Vector__XXX' ;
```

The transmitter name specifies the node that transmits the message. The transmitter name must be defined in the node section. If the message has no sender, the string `Vector__XXX` must be used.

## Pseudo-message

A pseudo-message named `VECTOR__INDEPENDENT_SIG_MSG` may exist in DBC files. This is an internal DBC construct used to store signals that are not associated with any actual CAN message.

## Signal Definitions

The message's signal section lists all signals contained within the message, including their bit positions in the message's data field and their properties.

```bnf
signal = 'SG_' signal_name multiplexer_indicator ':' start_bit '|' signal_size '@' byte_order value_type '(' factor ',' offset ')' '[' minimum '|' maximum ']' unit receiver {',' receiver} ;
signal_name = DBC_identifier ;
```

All signal names defined within a message must be unique within that message.

```bnf
multiplexer_indicator = ' ' | [m multiplexer_switch_value] [M] ;
```

The multiplexer indicator specifies whether the signal is a normal signal, a multiplexer switch, or a multiplexed signal. An uppercase `M` character identifies the signal as the multiplexer switch. A lowercase `m` character followed by an unsigned integer identifies the signal as multiplexed by the multiplexer switch. A multiplexed signal is transmitted in the message only when the multiplexer switch value equals the signal's `multiplexer_switch_value`.

**Note**: A signal may function as both a multiplexed signal and a multiplexer switch signal simultaneously. Additionally, more than one signal within a single message can serve as a multiplexer switch. In both of these cases, the extended multiplexing section (see below) must not be empty.

The `start_bit` value specifies the bit position of the signal within the message's data field. For signals with Motorola byte order (big-endian, `@0`), the position of the most significant bit is specified. For signals with Intel byte order (little-endian, `@1`), the position of the least significant bit is specified. Bits are numbered in a sawtooth pattern (byte 0: bits 0-7, byte 1: bits 8-15, etc.). The `start_bit` must be in the range of 0 to (8 × `message_size` - 1).

```bnf
start_bit = unsigned_integer ;
```

The `signal_size` specifies the length of the signal in bits.

```bnf
byte_order = '0' | '1' ; (* 0=big endian (Motorola), 1=little endian (Intel) *)
```

The `byte_order` is `0` for Motorola (big-endian) or `1` for Intel (little-endian).

**Note:** Per Vector DBC specification version 1.0.1 (2007-11-19), this was corrected: "Big endian is stored as '0', little endian is stored as '1'."

```bnf
value_type = '+' | '-' ; (* +=unsigned, -=signed *)
```

The `value_type` defines the signal as unsigned (`+`) or signed (`-`).

```bnf
factor = double ;
offset = double ;
```

The `factor` and `offset` define the linear conversion rule to convert the signal's raw value into its physical value and vice versa:

```bnf
physical_value = raw_value * factor + offset
raw_value = (physical_value – offset) / factor
```

As shown in the conversion formulas, the `factor` must not be zero.

```bnf
minimum = double ;
maximum = double ;
```

The `minimum` and `maximum` define the valid range of physical values for the signal.

```bnf
unit = char_string ;
receiver = node_name | 'Vector__XXX' | '*' ;
receivers = receiver | receiver { ( ' ' | ',' ) receiver } ;
```

The receiver specification defines which nodes receive the signal. Multiple receivers may be specified as either a space-separated or comma-separated list. The receiver names must be defined in the node section. 

**Receiver formats:**
- `*` - Signal is broadcast to all nodes
- `node_name` - Signal is sent to a single specific node
- `node1 node2 node3` - Signal is sent to multiple specific nodes (space-separated)
- `node1,node2,node3` - Signal is sent to multiple specific nodes (comma-separated, also common in practice)
- Empty (no receivers specified) - Signal has no receivers defined

**Note:** While the BNF grammar allows both space and comma separation, many DBC files in practice use comma-separated lists for multiple receivers. Parsers should accept both formats. If a signal has no receiver, the string `Vector__XXX` may be used, though this is less common than using `*` for broadcast signals.

Signals with value types `float` and `double` have additional entries in the `signal_extended_value_type_list` section.

```bnf
signal_extended_value_type_list = {signal_extended_value_type_entry} ;
signal_extended_value_type_entry = 'SIG_VALTYPE_' message_id signal_name signal_extended_value_type ';' ;
signal_extended_value_type = '0' | '1' | '2' | '3' ; (* 0=signed or unsigned integer, 1=32-bit IEEE-float, 2=64-bit IEEE-double, 3=reserved *)
```

**Notes:**
- This section is only used for signals that have `float` or `double` value types.
- For integer signals (signed or unsigned), this section is not needed.
- The value type `3` is reserved for future use.

---

## Bit Manipulation and Byte Ordering

Understanding how signals are encoded and decoded in CAN messages requires knowledge of bit numbering, byte ordering (endianness), and the sawtooth bit pattern used in DBC files.

### Bit Numbering: The Sawtooth Pattern

CAN messages are transmitted as sequences of bytes. Bits within these bytes are numbered using a "sawtooth" pattern that counts sequentially across bytes:

```
Byte 0:  bits  0-7   (LSB=0, MSB=7)
Byte 1:  bits  8-15  (LSB=8, MSB=15)
Byte 2:  bits 16-23  (LSB=16, MSB=23)
Byte 3:  bits 24-31  (LSB=24, MSB=31)
... and so on
```

This numbering is independent of byte order and provides a consistent way to reference any bit position in the message.

**Example:** In an 8-byte message, bit positions range from 0 to 63.

### Byte Order (Endianness)

The byte order determines how multi-byte signals are interpreted within the message. DBC files support two byte orders:

#### Big-Endian (Motorola) - `@0`

- **MSB first**: The most significant byte is stored at the lower memory address
- **Bit interpretation**: `start_bit` refers to the **most significant bit (MSB)** of the signal in big-endian numbering
- **Signal extension**: The signal extends **backward** (toward lower bit numbers) from the start bit
- **Bit numbering**: Big-endian uses a different bit numbering scheme within bytes

**Big-Endian Bit Numbering:**
Within each byte, bits are numbered from MSB to LSB:
```
Byte 0: bits 7, 6, 5, 4, 3, 2, 1, 0 (MSB to LSB)
Byte 1: bits 15, 14, 13, 12, 11, 10, 9, 8
Byte 2: bits 23, 22, 21, 20, 19, 18, 17, 16
... and so on
```

**Example - Big-Endian Signal:**
```
Signal: start_bit=0, length=16, byte_order=@0
Message bytes: [0x12, 0x34, ...]
                ^^^^  ^^^^
                MSB   LSB

Raw value = 0x1234 (bytes read as: (0x12 << 8) + 0x34)
```

**Visual representation:**
```
Byte 0          Byte 1
7 6 5 4 3 2 1 0  7 6 5 4 3 2 1 0
└─┘ └─────┘ └─┘  └─┘ └─────┘ └─┘
│   │     │   │   │   │     │   │
│   └─────┴───┴───┴───┴──────┘   │
│        16-bit signal           │
│        (start_bit=0, BE)       │
└────────────────────────────────┘
```

#### Little-Endian (Intel) - `@1`

- **LSB first**: The least significant byte is stored at the lower memory address
- **Bit interpretation**: `start_bit` refers to the **least significant bit (LSB)** of the signal
- **Signal extension**: The signal extends **forward** (toward higher bit numbers) from the start bit
- **Bit range**: For a signal with `start_bit = N` and `length = L`, the signal occupies bits `[N, N+L-1]`

**Example - Little-Endian Signal:**
```
Signal: start_bit=0, length=16, byte_order=@1
Message bytes: [0x34, 0x12, ...]
                ^^^^  ^^^^
                LSB   MSB

Raw value = 0x1234 (bytes read as: 0x34 + (0x12 << 8))
```

**Visual representation:**
```
Byte 0          Byte 1
7 6 5 4 3 2 1 0  7 6 5 4 3 2 1 0
└─┘ └─────┘ └─┘  └─┘ └─────┘ └─┘
│   │     │   │   │   │     │   │
│   └─────┴───┴───┴───┴──────┘   │
│        16-bit signal           │
│        (start_bit=0)           │
└────────────────────────────────┘
```

### Converting Between Physical and Raw Values

Once the raw integer value is extracted from the message bytes, it must be converted to a physical value using the signal's scaling:

**Formula:**
```
physical_value = raw_value × factor + offset
raw_value = (physical_value - offset) / factor
```

**Example:**
```
Signal definition: SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm"
- start_bit = 0
- length = 16 bits
- byte_order = @1 (little-endian)
- value_type = + (unsigned)
- factor = 0.25
- offset = 0

If message bytes are [0x40, 0x01, ...]:
Raw value = 0x0140 = 320 (decimal)
Physical value = 320 × 0.25 + 0 = 80 rpm
```

### Signal Extraction Algorithm

To extract a signal value from CAN message bytes:

1. **Determine bit range** based on byte order:
   - **Little-endian**: `[start_bit, start_bit + length - 1]`
   - **Big-endian**: Convert `start_bit` to physical bit position, then calculate range

2. **Extract bits** from the message bytes:
   - Identify which bytes contain the signal
   - Extract the relevant bits, handling byte boundaries
   - Combine bits into a single integer value

3. **Apply sign extension** (for signed signals):
   - If `value_type = '-'` and the MSB is set, extend the sign bit

4. **Convert to physical value**:
   - Apply scaling: `physical = raw × factor + offset`
   - Validate against min/max range

### Signal Overlap Detection

Signals within a message must not overlap in their physical bit ranges. Overlap detection requires:

1. Calculate the physical bit range for each signal (accounting for byte order)
2. Check if any two signals have overlapping bit ranges
3. Report an error if overlap is detected

**Note:** Signals can share the same physical bits only if they are multiplexed (see multiplexing section above).

### Practical Examples

#### Example 1: Big-Endian 16-bit Signal
```
Signal: SG_ Pressure : 0|16@0+ (0.01,0) [0|1000] "kPa"
Message: BO_ 200 PressureMsg : 2 ECU2

Message bytes: [0x03, 0xE8]
                ^^^^  ^^^^
                MSB   LSB

Raw value = 0x03E8 = 1000
Physical value = 1000 × 0.01 + 0 = 10.0 kPa
```

#### Example 2: Little-Endian 16-bit Signal
```
Signal: SG_ Speed : 0|16@1+ (0.1,0) [0|6553.5] "km/h"
Message: BO_ 100 SpeedMsg : 2 ECU1

Message bytes: [0x64, 0x00]
                ^^^^  ^^^^
                LSB   MSB

Raw value = 0x0064 = 100
Physical value = 100 × 0.1 + 0 = 10.0 km/h
```

#### Example 3: Multiple Signals in One Message
```
Message: BO_ 256 EngineData : 8 ECM
 SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm"
 SG_ Temp : 16|8@1- (1,-40) [-40|215] "°C"
 SG_ Status : 24|8@1+ (1,0) [0|255] ""

Message bytes: [0x40, 0x01, 0x82, 0x05, ...]
                ^^^^  ^^^^  ^^^^  ^^^^
                RPM   RPM   Temp  Status
                LSB   MSB

RPM:    Raw = 0x0140 = 320,    Physical = 80.0 rpm
Temp:   Raw = 0x82 = 130,       Physical = 130 - 40 = 90°C
Status: Raw = 0x05 = 5,          Physical = 5
```

### Common Pitfalls

1. **Confusing byte order with bit numbering**: Byte order affects how bytes are interpreted, but bit numbering always follows the sawtooth pattern.

2. **Sign extension**: Signed signals require sign extension when the MSB is set. A 16-bit signed signal with raw value `0x8000` represents `-32768`, not `32768`.

3. **Bit range calculation**: For big-endian signals, the physical bit range is not simply `[start_bit, start_bit + length - 1]`; it requires conversion based on the big-endian bit numbering scheme.

4. **Factor cannot be zero**: The conversion formula requires division by factor, so `factor = 0` is invalid and will cause errors.

5. **Overlapping signals**: Signals that physically overlap in the message (without multiplexing) indicate an error in the DBC file definition.

## Definition of Message Transmitters

The message transmitter section enables defining multiple transmitter nodes for a single message. This is used to describe communication data for higher-layer protocols, not for CAN layer-2 communication.

```bnf
message_transmitters = {message_transmitter} ;
Message_transmitter = 'BO_TX_BU_' message_id ':' {transmitter} ';' ;
```

### Signal Value Descriptions (Value Encodings)

Signal value descriptions define textual encodings for specific raw signal values.

```bnf
value_descriptions = { value_descriptions_for_signal | value_descriptions_for_env_var } ;
value_descriptions_for_signal = 'VAL_' message_id signal_name { value_description } ';' ;
```

# Environment Variable Definitions

The environment variables section defines environment variables used in system simulation and other bus simulation tools.

```bnf
environment_variables = {environment_variable}
environment_variable = 'EV_' env_var_name ':' env_var_type '[' minimum '|' maximum ']' unit initial_value ev_id access_type access_node {',' access_node } ';' ;
env_var_name = DBC_identifier ;
env_var_type = '0' | '1' | '2' ; (* 0=integer, 1=float, 2=string *)
minimum = double ;
maximum = double ;
initial_value = double ;
ev_id = unsigned_integer ; (* obsolete *)
access_type = 'DUMMY_NODE_VECTOR0' | 'DUMMY_NODE_VECTOR1' |
    'DUMMY_NODE_VECTOR2' | 'DUMMY_NODE_VECTOR3' |'DUMMY_NODE_VECTOR8000' | 'DUMMY_NODE_VECTOR8001' | 'DUMMY_NODE_VECTOR8002' | 'DUMMY_NODE_VECTOR8003'; (* 0=unrestricted, 1=read, 2=write, 3=readWrite, if the value behind 'DUMMY_NODE_VECTOR' is OR-ed with 0x8000, the value type is always string. *)
access_node = node_name | 'VECTOR__XXX' ;
```

The environment variables data section defines environment variables as having the "Data" type. These variables can store arbitrary binary data of a specified length, where the length is specified in bytes.

```bnf
environment_variables_data = {environment_variable_data} ;
environment_variable_data = 'ENVVAR_DATA_' env_var_name ':' data_size ';' ;
data_size = unsigned_integer ;
```

### Environment Variable Value Descriptions

Environment variable value descriptions provide textual representations for specific variable values.

```bnf
value_descriptions_for_env_var = 'VAL_' env_var_name { value_description } ';' ;
```

# Signal Type and Signal Group Definitions

Signal types define common properties shared by multiple signals. They are typically not used in DBC files.

```bnf
signal_types = {signal_type} ;
signal_type = 'SGTYPE_' signal_type_name ':' signal_size '@' byte_order value_type '(' factor ',' offset ')' '[' minimum '|' maximum ']' unit default_value ',' value_table ';' ;
signal_type_name = DBC_identifier ;
default_value = double ;
value_table = value_table_name ;
signal_type_refs = {signal_type_ref} ;
signal_type_ref = 'SGTYPE_' message_id signal_name ':' signal_type_name ';' ;
```

Signal groups define collections of signals within a message, for example, to specify that all signals in a group must be updated together.

```bnf
signal_groups = {signal_group} ;
signal_group = 'SIG_GROUP_' message_id signal_group_name repetitions ':' { signal_name } ';' ;
signal_group_name = DBC_identifier ;
repetitions = unsigned_integer ;
```

**Notes:**
- Signal groups are used to organize signals within a message for tooling and UI purposes.
- The `repetitions` field specifies how many times the signal group should be repeated (typically `1`).
- Signal groups do not affect the actual CAN message structure or signal encoding.

# Comment Definitions

The comment section contains comments for objects. Each object with a comment has an entry in this section, identified by the object's type.

```bnf
comments = {comment} ;
comment = 'CM_' (char_string | 'BU_' node_name char_string | 'BO_' message_id char_string | 'SG_' message_id signal_name char_string | 'EV_' env_var_name char_string) ';' ;
```

**Notes:**
- Comments can be attached to nodes, messages, signals, environment variables, or the DBC file itself.
- Each comment entry specifies the object type and identifier, followed by the comment text.
- Comment text is a `char_string` (may contain spaces and printable characters).

# User Defined Attribute Definitions

User-defined attributes extend the object properties in a DBC file. These attributes must be defined using an attribute definition with a default value. For each object with a value assigned to the attribute, an attribute value entry must be defined. If no attribute value entry is defined for an object, the object's attribute value defaults to the attribute's default value.

### Attribute Definitions

```bnf
attribute_definitions = { attribute_definition } ;
attribute_definition = 'BA_DEF_' object_type attribute_name attribute_value_type ';' ;
object_type = '' | 'BU_' | 'BO_' | 'SG_' | 'EV_' ;
attribute_name = '"' DBC_identifier '"' ;
attribute_value_type = 'INT' signed_integer signed_integer | 'HEX' signed_integer signed_integer | 'FLOAT' double double | 'STRING' | 'ENUM' {char_string (',' char_string)} ;
attribute_defaults = { attribute_default } ;
attribute_default = 'BA_DEF_DEF_' attribute_name attribute_value ';' ;
attribute_value = unsigned_integer | signed_integer | double | char_string ;
```

### Attribute Values

```bnf
attribute_values = { attribute_value_for_object } ;
attribute_value_for_object = 'BA_' attribute_name (attribute_value | 'BU_' node_name attribute_value | 'BO_' message_id attribute_value | 'SG_' message_id signal_name attribute_value | 'EV_' env_var_name attribute_value) ';' ;
```

# Extended Multiplexing

Extended multiplexing enables defining multiple multiplexer switches within a single message. It also allows using multiple multiplexer switch values for each multiplexed signal.

The extended multiplexing section contains multiplexed signals for which following conditions were fulfilled:

- The multiplexed signal is multiplexed by more than one multiplexer switch value
- The multiplexed signal belongs to a message which contains more than one multiplexor switch

```bnf
extended_multiplexing = {multiplexed_signal} ;
multiplexed_signal = 'SG_MUL_VAL_' message_id multiplexed_signal_name multiplexor_switch_name multiplexor_value_ranges ';' ;
message_id = unsigned_integer ;
multiplexed_signal_name = DBC_identifier ;
multiplexor_switch_name = DBC_identifier ;
multiplexor_value_ranges = {multiplexor_value_range} ;
multiplexor_value_range = unsigned_integer '-' unsigned_integer ;
```

# Examples

## Minimal DBC File

```
VERSION "1.0"

BS_:

BU_: Engine Gateway

BO_ 100 EngineData : 8 Engine
 SG_ PetrolLevel : 24|8@1+ (1,0) [0|255] "l" Gateway
 SG_ EngPower : 48|16@1+ (0.01,0) [0|150] "kW" Gateway
 SG_ EngForce : 32|16@1+ (1,0) [0|0] "N" Gateway
 SG_ IdleRunning : 23|1@1+ (1,0) [0|0] "" Gateway
 SG_ EngTemp : 16|7@1+ (2,-50) [-50|150] "degC" Gateway
 SG_ EngSpeed : 0|16@1+ (1,0) [0|8000] "rpm" Gateway

CM_ "CAN communication matrix for power train electronics" ;
```

## Complete DBC File with New Symbols Section

```
VERSION "1.0"

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
    SG_MUL_VAL_

BS_:

BU_: Engine Gateway

BO_ 100 EngineData : 8 Engine
 SG_ PetrolLevel : 24|8@1+ (1,0) [0|255] "l" Gateway
 SG_ EngPower : 48|16@1+ (0.01,0) [0|150] "kW" Gateway
 SG_ EngForce : 32|16@1+ (1,0) [0|0] "N" Gateway
 SG_ IdleRunning : 23|1@1+ (1,0) [0|0] "" Gateway
 SG_ EngTemp : 16|7@1+ (2,-50) [-50|150] "degC" Gateway
 SG_ EngSpeed : 0|16@1+ (1,0) [0|8000] "rpm" Gateway

CM_ "CAN communication matrix for power train electronics" ;
```
