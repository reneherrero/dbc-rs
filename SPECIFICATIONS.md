# DBC File Format Specification

---

## Table of Contents

1. [Introduction](#introduction)
2. [General Definitions](#general-definitions)
3. [File Structure](#file-structure)
4. [Version and New Symbols](#version-and-new-symbols)
5. [Bit Timing Definition](#bit-timing-definition)
6. [Node Definitions](#node-definitions)
7. [Value Table Definitions](#value-table-definitions)
8. [Message Definitions](#message-definitions)
9. [Signal Definitions](#signal-definitions)
10. [Bit Manipulation and Byte Ordering](#bit-manipulation-and-byte-ordering)
11. [Message Transmitters](#message-transmitters)
12. [Environment Variables](#environment-variables)
13. [Signal Types and Groups](#signal-types-and-groups)
14. [Comments](#comments)
15. [User-Defined Attributes](#user-defined-attributes)
16. [Extended Multiplexing](#extended-multiplexing)
17. [Common Attributes](#common-attributes)
18. [Examples](#examples)

---

## 1. Introduction

A DBC (Database Container) file describes the communication protocol for a single Controller Area Network (CAN). The information contained within enables:

- **Network monitoring** - Real-time observation of CAN traffic
- **Traffic analysis** - Decoding raw CAN data to human-readable values
- **ECU simulation** - "Remaining bus simulation" for ECUs not physically present
- **Communication software development** - Foundation for ECU software that participates in the network

**Important Limitations:**
- DBC files describe **only the communication layer**
- They do **not** specify internal functional behavior or application logic of ECUs
- They represent the **passive/reading** part of CAN communication, not transmission logic

---

## 2. General Definitions

The following general elements are used throughout this specification:

| Element | Description | Example |
|---------|-------------|---------|
| `unsigned_integer` | An unsigned integer | `100`, `255`, `0` |
| `signed_integer` | A signed integer | `-50`, `0`, `100` |
| `double` | A double precision floating point number | `0.25`, `-40.5`, `1.0` |
| `Printable character` | ASCII characters 0x20 - 0x7E (including space) | `A`, `9`, `_`, ` ` |
| `char_string` | String of printable characters (no `"` or `\`) | `"Engine Speed"`, `"km/h"` |
| `C_identifier` | Valid C identifier: starts with letter/underscore, contains alphanumeric/underscores | `EngSpeed`, `_temp`, `signal_1` |
| `DBC_identifier` | A `C_identifier` that is not a DBC keyword | `EngineData`, `RPM` |

### DBC Keywords

The following are reserved keywords and **cannot** be used as identifiers:

```
VERSION, NS_, NS_DESC_, CM_, BA_DEF_, BA_, VAL_, CAT_DEF_, CAT_, FILTER, 
BA_DEF_DEF_, EV_DATA_, ENVVAR_DATA_, SGTYPE_, SGTYPE_VAL_, BA_DEF_SGTYPE_, 
BA_SGTYPE_, SIG_TYPE_REF_, VAL_TABLE_, SIG_GROUP_, SIG_VALTYPE_, 
SIGTYPE_VALTYPE_, BO_TX_BU_, BA_DEF_REL_, BA_REL_, BA_DEF_DEF_REL_, 
BU_SG_REL_, BU_EV_REL_, BU_BO_REL_, SG_MUL_VAL_, BS_, BU_, BO_, SG_, EV_, 
VECTOR__INDEPENDENT_SIG_MSG, VECTOR__XXX
```

### Object Type Keywords

| Keyword | Object Type |
|---------|-------------|
| `BU_` | Network Node (Bus Unit) |
| `BO_` | Message (Bus Object) |
| `SG_` | Signal |
| `EV_` | Environment Variable |
| `SIG_GROUP_` | Signal Group |
| `VAL_TABLE_` | Value Table |

### String and Identifier Limits

- **DBC identifiers:** Maximum 32 characters
- **Other strings:** Arbitrary length (parser-dependent limits may apply)
- **Security:** Modern parsers limit string lengths to prevent buffer overflow attacks
- **dbc-rs note:** For security reasons, dbc-rs limits string lengths depending on the object type

---

## 3. File Structure

The DBC file format has the following overall structure:

```bnf
DBC_file =
    version
    new_symbols
    bit_timing            (* Required but typically empty *)
    nodes                 (* Required *)
    value_tables
    messages              (* Core section *)
    message_transmitters
    environment_variables
    environment_variables_data
    signal_types          (* Rarely used *)
    comments
    attribute_definitions
    sigtype_attr_list     (* Rarely used *)
    attribute_defaults
    attribute_values
    value_descriptions
    category_definitions  (* Obsolete *)
    categories            (* Obsolete *)
    filter                (* Obsolete *)
    signal_type_refs      (* Rarely used *)
    signal_groups
    signal_extended_value_type_list
    extended_multiplexing ;
```

### Core Sections (Required for Basic DBC)

1. **bit_timing** - Required keyword but typically empty
2. **nodes** - Defines network participants
3. **messages** - Defines CAN messages and their signals

### Optional Sections (Common)

- **comments** - Human-readable descriptions
- **attribute_definitions** - Custom attributes
- **attribute_values** - Attribute assignments
- **value_descriptions** - Text encodings for signal values
- **signal_extended_value_type_list** - Float/double signal types

### Rarely Used Sections

- **signal_types** - Shared signal properties
- **sigtype_attr_list** - Signal type attributes
- **signal_type_refs** - Signal type references
- **signal_groups** - Signal groupings

### Rarely Used Sections

- **category_definitions** - Defined for completeness but rarely used in practice
- **categories** - Defined for completeness but rarely used in practice
- **filter** - Defined for completeness but rarely used in practice

---

## 4. Version and New Symbols

### 4.1 Version

```bnf
version = ['VERSION' '"' char_string '"' ] ;
```

**Examples:**
```
VERSION ""
VERSION "1.0"
VERSION "MyDatabase_v2.5"
```

**Notes:**
- Optional section
- Empty version (`VERSION ""`) is valid and common
- If omitted, parsers assume empty version

### 4.2 New Symbols

```bnf
new_symbols = [ 'NS_' ':' 
    ['CM_'] ['BA_DEF_'] ['BA_'] ['VAL_']
    ['CAT_DEF_'] ['CAT_'] ['FILTER'] ['BA_DEF_DEF_'] 
    ['EV_DATA_'] ['ENVVAR_DATA_'] ['SGTYPE_'] ['SGTYPE_VAL_'] 
    ['BA_DEF_SGTYPE_'] ['BA_SGTYPE_'] ['SIG_TYPE_REF_'] 
    ['VAL_TABLE_'] ['SIG_GROUP_'] ['SIG_VALTYPE_'] 
    ['SIGTYPE_VALTYPE_'] ['BO_TX_BU_'] ['BA_DEF_REL_'] 
    ['BA_REL_'] ['BA_DEF_DEF_REL_'] ['BU_SG_REL_']
    ['BU_EV_REL_'] ['BU_BO_REL_'] ['SG_MUL_VAL_'] 
] ;
```

**Purpose:** Declares which optional sections are present in the file

**Example:**
```
NS_ : 
    CM_
    BA_DEF_
    BA_
    VAL_
```

---

## 5. Bit Timing Definition

```bnf
bit_timing = 'BS_:' [baudrate ':' BTR1 ',' BTR2 ] ;
baudrate = unsigned_integer ;
BTR1 = unsigned_integer ;
BTR2 = unsigned_integer ;
```

**Examples:**
```
BS_:
BS_: 500
BS_: 500 : 12,34
```

**Important Notes:**
- **REQUIRED KEYWORD** but section is typically empty
- **OBSOLETE** - No longer used in modern CAN systems
- Baudrate and BTR values are not processed by modern parsers (legacy feature)
- Always include `BS_:` in your DBC files

---

## 6. Node Definitions

```bnf
nodes = 'BU_:' {node_name} ;
node_name = DBC_identifier ;
```

**Examples:**
```
BU_:
BU_: Engine Gateway Dashboard
BU_: ECM TCM BCM ADAS_Controller
```

**Rules:**
- **REQUIRED SECTION**
- Node names separated by whitespace
- All node names must be **unique** (case-sensitive)
- Empty node list is valid (`BU_:`)
- Maximum 32 characters per node name

**Best Practices:**
- Use descriptive names: `EngineControlModule` not `ECU1`
- Follow naming convention: CamelCase or underscore_separated
- Avoid special characters

---

## 7. Value Table Definitions

```bnf
value_tables = {value_table} ;
value_table = 'VAL_TABLE_' value_table_name {value_description} ';' ;
value_table_name = DBC_identifier ;
value_description = unsigned_integer char_string ;
```

**Example:**
```
VAL_TABLE_ GearPosition 0 "Park" 1 "Reverse" 2 "Neutral" 3 "Drive" 4 "Sport" ;
VAL_TABLE_ OnOff 0 "Off" 1 "On" ;
```

**Notes:**
- Global value tables (rarely used in practice)
- More common: signal-specific value descriptions (see Value Descriptions section)
- Maps numeric values to human-readable text

---

## 8. Message Definitions

```bnf
messages = {message} ;
message = 'BO_' message_id message_name ':' message_size transmitter {signal} ;
message_id = unsigned_integer ;
message_name = DBC_identifier ;
message_size = unsigned_integer ;
transmitter = node_name | 'Vector__XXX' ;
```

### 8.1 Message ID

**Format:** `unsigned_integer`

**Rules:**
- Must be **unique** within the DBC file
- **Standard CAN ID:** 0 to 2047 (0x7FF) - 11-bit identifier
- **Extended CAN ID:** Set bit 31 (0x80000000) and use bits 0-28

**Extended ID Calculation:**
```
Extended ID in DBC = 0x80000000 | actual_extended_id
Example: 0x80001234 represents extended ID 0x1234
```

**Examples:**
```
BO_ 100 SpeedData : 8 ECM           (Standard ID: 100)
BO_ 2147484148 DiagData : 8 TCM     (Extended ID: 0x494, with bit 31 set)
```

### 8.2 Message Name

**Rules:**
- Must be unique across all messages
- Maximum 32 characters
- Must be valid `DBC_identifier`

### 8.3 Message Size

**Format:** Data Length Code (DLC) in bytes

**Valid Values:**
- **CAN 2.0:** 0 to 8 bytes
- **CAN FD:** 0 to 64 bytes (tool-dependent support)

### 8.4 Transmitter

**Options:**
- **Node name** - Must be defined in `BU_:` section
- **`Vector__XXX`** - No sender / unknown sender

**Example:**
```
BO_ 200 EngineData : 8 Engine
BO_ 300 UnknownMsg : 4 Vector__XXX
```

### 8.5 Message Components

| Component | Description | Example |
|-----------|-------------|---------|
| `message_id` | Unique CAN identifier | `100`, `2147484148` |
| `message_name` | Unique message name | `EngineData`, `SpeedData` |
| `message_size` | Data Length Code (DLC) in bytes | `0`, `4`, `8` |
| `transmitter` | Sending node | `ECM`, `Vector__XXX` |

### 8.6 Pseudo-Message

Special message for signals not associated with any CAN message:

```
BO_ 3221225472 VECTOR__INDEPENDENT_SIG_MSG : 0 Vector__XXX
```

**Notes:**
- Message ID: `3221225472` (0xC0000000)
- Used internally by tools
- Contains "orphan" signals

---

## 9. Signal Definitions

### 9.1 Signal Syntax

```bnf
signal = 'SG_' signal_name multiplexer_indicator ':' 
         start_bit '|' signal_size '@' byte_order value_type 
         '(' factor ',' offset ')' 
         '[' minimum '|' maximum ']' 
         unit receivers ;
```

### 9.2 Signal Components

| Component | Description | Example |
|-----------|-------------|---------|
| `signal_name` | Unique within message | `EngineSpeed` |
| `start_bit` | Bit position (0 to 8×DLC-1) | `0`, `16`, `48` |
| `signal_size` | Length in bits | `8`, `16`, `32` |
| `byte_order` | `0` = Big-Endian, `1` = Little-Endian | `@1` |
| `value_type` | `+` = unsigned, `-` = signed | `+`, `-` |
| `factor` | Scaling factor (cannot be 0) | `0.25`, `1.0` |
| `offset` | Offset value | `0`, `-40` |
| `minimum` | Minimum physical value | `0`, `-273.15` |
| `maximum` | Maximum physical value | `8000`, `100` |
| `unit` | Physical unit (string) | `"rpm"`, `"°C"` |
| `receivers` | Receiving nodes | `Gateway`, `Vector__XXX` |

### 9.3 Complete Example

```
SG_ EngineSpeed : 0|16@1+ (0.25,0) [0|8000] "rpm" Gateway,Dashboard
```

**Breakdown:**
- Signal name: `EngineSpeed`
- Start bit: 0 (LSB for little-endian)
- Length: 16 bits
- Byte order: Little-endian (`@1`)
- Value type: Unsigned (`+`)
- Factor: 0.25 (raw × 0.25 = physical)
- Offset: 0
- Range: 0 to 8000 rpm
- Unit: "rpm"
- Receivers: Gateway and Dashboard nodes

### 9.4 Multiplexer Indicator

```bnf
multiplexer_indicator = ' ' | 'M' | 'm' multiplexer_switch_value ;
multiplexer_switch_value = unsigned_integer ;
```

**Types:**
- **Normal signal:** (space character or nothing)
- **Multiplexer switch:** `M`
- **Multiplexed signal:** `m0`, `m1`, `m2`, etc.

**Example (Basic Multiplexing):**
```
BO_ 400 MultiplexedMsg : 8 ECM
 SG_ MuxSwitch M : 0|8@1+ (1,0) [0|255] "" Gateway
 SG_ Signal_0 m0 : 8|16@1+ (0.1,0) [0|1000] "kPa" Gateway
 SG_ Signal_1 m1 : 8|16@1+ (0.01,0) [0|100] "°C" Gateway
```

**Rules:**
- When `MuxSwitch` = 0, `Signal_0` is valid
- When `MuxSwitch` = 1, `Signal_1` is valid
- Multiplexed signals share the same bit positions
- For extended multiplexing (multiple multiplexer switches), see Section 16

### 9.5 Receivers

```bnf
receiver = node_name | 'Vector__XXX' ;
receivers = receiver {',' receiver} ;
```

**Formats:**
```
Gateway                    (Single receiver)
Gateway,Dashboard          (Multiple receivers - comma-separated)
Vector__XXX                (No specific receiver)
```

**Note:** The specification BNF defines only comma-separated receivers. Some tools may accept space-separated receivers as an extension, but this is not part of the specification.

---

## 10. Bit Manipulation and Byte Ordering

### 10.1 Sawtooth Bit Numbering

CAN message bytes use sequential bit numbering:

```
Byte 0:  bits  0-7   (bit 0 = LSB, bit 7 = MSB of byte)
Byte 1:  bits  8-15
Byte 2:  bits 16-23
Byte 3:  bits 24-31
Byte 4:  bits 32-39
Byte 5:  bits 40-47
Byte 6:  bits 48-55
Byte 7:  bits 56-63
```

**Example:** 8-byte message has bit positions 0-63

### 10.2 Byte Order (CORRECTED)

**⚠️ CRITICAL CORRECTION:**

The original Vector documentation had an error regarding byte order encoding. The correct interpretation is:

| Value | Byte Order | Description | Alternative Name |
|-------|------------|-------------|------------------|
| `@0` | **Big-Endian** | Most significant byte first | Motorola |
| `@1` | **Little-Endian** | Least significant byte first | Intel |

**Source:** Vector DBC specification version 1.0.1 (2007-11-19) correction states "Big endian is stored as '0', little endian is stored as '1'"

### 10.3 Little-Endian Signals (`@1`)

**Start bit interpretation:** **Least Significant Bit (LSB)**

**Signal bit range:** `[start_bit, start_bit + length - 1]`

**Example:**
```
Signal: SG_ Speed : 0|16@1+ (0.1,0) [0|6553.5] "km/h"
Message bytes: [0x64, 0x00, ...]

Bit layout:
Byte 0          Byte 1
7 6 5 4 3 2 1 0  15 14 13 12 11 10 9 8
[   0x64      ]  [    0x00         ]
└───────┬───────┴────────┬──────────┘
     16-bit signal (bits 0-15)

Raw value = 0x0064 = 100 decimal
Physical = 100 × 0.1 = 10.0 km/h
```

### 10.4 Big-Endian Signals (`@0`)

**Start bit interpretation:** **Most Significant Bit (MSB)**

**More complex:** Signal extends "backward" toward lower bit numbers

**Big-Endian Bit Numbering Within Bytes:**
```
Byte 0: bits 7, 6, 5, 4, 3, 2, 1, 0 (MSB to LSB)
Byte 1: bits 15, 14, 13, 12, 11, 10, 9, 8
Byte 2: bits 23, 22, 21, 20, 19, 18, 17, 16
```

**Example:**
```
Signal: SG_ Pressure : 7|16@0+ (0.01,0) [0|655.35] "kPa"
Message bytes: [0x03, 0xE8, ...]

Bit layout (Big-Endian perspective):
Byte 0          Byte 1
7 6 5 4 3 2 1 0  15 14 13 12 11 10 9 8
[   0x03      ]  [    0xE8         ]
└───────┬───────┴────────┬──────────┘
     16-bit signal (bits 7 down to -8... wraps to next byte)

Raw value = 0x03E8 = 1000 decimal
Physical = 1000 × 0.01 = 10.0 kPa
```

### 10.5 Value Conversion

**Formula:**
```
physical_value = raw_value × factor + offset
raw_value = (physical_value - offset) / factor
```

**Rules:**
- Factor **cannot be zero** (division required)
- Signed signals use two's complement representation
- Range check: `minimum ≤ physical_value ≤ maximum`

**Example with Offset:**
```
Signal: SG_ Temperature : 16|8@1- (1,-40) [-40|87] "°C"

Raw value = 127 (0x7F)
Physical = 127 × 1 + (-40) = 87°C

Raw value = 0 (0x00)
Physical = 0 × 1 + (-40) = -40°C

Physical = 20°C
Raw = (20 - (-40)) / 1 = 60
```

**Note:** For a signed 8-bit signal, the raw value range is -128 to 127. With factor=1 and offset=-40, the physical range is -168 to 87. The example uses a valid range of [-40|87] for clarity.

### 10.6 Signal Extended Value Types

For floating-point signals:

```bnf
signal_extended_value_type_list = {signal_extended_value_type_entry} ;
signal_extended_value_type_entry = 'SIG_VALTYPE_' message_id signal_name signal_extended_value_type ';' ;
signal_extended_value_type = '0' | '1' | '2' ;
```

**Value Types:**
- `0` = Signed or unsigned integer (default)
- `1` = 32-bit IEEE float
- `2` = 64-bit IEEE double

**Example:**
```
SIG_VALTYPE_ 100 FloatSignal 1 ;
SIG_VALTYPE_ 100 DoubleSignal 2 ;
```

---

## 11. Message Transmitters

Defines multiple transmitters for a message (higher-layer protocols):

```bnf
message_transmitters = {message_transmitter} ;
message_transmitter = 'BO_TX_BU_' message_id ':' {transmitter} ';' ;
```

**Example:**
```
BO_TX_BU_ 100 : Engine Gateway ;
```

**Note:** This is **not** for CAN layer-2 communication, but for higher-layer protocol descriptions.

---

## 12. Environment Variables

### 12.1 Environment Variable Definition

```bnf
environment_variable = 'EV_' env_var_name ':' env_var_type 
    '[' minimum '|' maximum ']' unit initial_value ev_id 
    access_type access_node {',' access_node } ';' ;
env_var_type = '0' | '1' | '2' ; (* 0=integer, 1=float, 2=string *)
access_type = 'DUMMY_NODE_VECTOR' ('0'|'1'|'2'|'3'|'8000'|'8001'|'8002'|'8003') ;
```

**Access Types:**
- `DUMMY_NODE_VECTOR0` = Unrestricted
- `DUMMY_NODE_VECTOR1` = Read only
- `DUMMY_NODE_VECTOR2` = Write only
- `DUMMY_NODE_VECTOR3` = Read/Write
- `DUMMY_NODE_VECTOR800X` = String type (OR-ed with 0x8000)

**Example:**
```
EV_ TestVar : 0 [0|100] "" 0 0 DUMMY_NODE_VECTOR0 ECM,TCM ;
```

### 12.2 Environment Variable Data

```bnf
environment_variable_data = 'ENVVAR_DATA_' env_var_name ':' data_size ';' ;
data_size = unsigned_integer ;
```

**Purpose:** Defines binary data storage with specified byte length

### 12.3 Value Descriptions for Environment Variables

```bnf
value_descriptions_for_env_var = 'VAL_' env_var_name { value_description } ';' ;
```

---

## 13. Signal Types and Groups

### 13.1 Signal Types (Rarely Used)

```bnf
signal_types = {signal_type} ;
signal_type = 'SGTYPE_' signal_type_name ':' signal_size '@' byte_order value_type '(' factor ',' offset ')' '[' minimum '|' maximum ']' unit default_value ',' value_table ';' ;
signal_type_name = DBC_identifier ;
default_value = double ;
value_table = value_table_name ;
signal_type_refs = {signal_type_ref} ;
signal_type_ref = 'SIG_TYPE_REF_' message_id signal_name ':' signal_type_name ';' ;
```

**Purpose:** Define reusable signal properties

### 13.2 Signal Type Attributes

Signal type attributes allow defining custom attributes for signal types, similar to regular attributes but specific to signal types.

```bnf
sigtype_attr_list = {sigtype_attribute_definition} {sigtype_attribute_value} ;
sigtype_attribute_definition = 'BA_DEF_SGTYPE_' attribute_name attribute_value_type ';' ;
sigtype_attribute_value = 'BA_SGTYPE_' attribute_name signal_type_name attribute_value ';' ;
attribute_name = '"' DBC_identifier '"' ;
attribute_value_type = 
    'INT' signed_integer signed_integer |
    'HEX' signed_integer signed_integer |
    'FLOAT' double double |
    'STRING' |
    'ENUM' char_string {',' char_string} ;
attribute_value = unsigned_integer | signed_integer | double | char_string ;
signal_type_name = DBC_identifier ;
```

**Purpose:** Extend signal type objects with custom attributes

**Examples:**
```
BA_DEF_SGTYPE_ "SignalTypeCategory" ENUM "Temperature","Pressure","Speed" ;
BA_SGTYPE_ "SignalTypeCategory" TempType "Temperature" ;
```

**Notes:**
- Similar to `BA_DEF_` and `BA_` but for signal types
- Rarely used in practice
- Signal types must be defined with `SGTYPE_` before attributes can be assigned

### 13.3 Signal Groups

```bnf
signal_groups = 'SIG_GROUP_' message_id signal_group_name repetitions { signal_name } ';' ;
signal_group_name = DBC_identifier ;
repetitions = unsigned_integer ;
```

**Example:**
```
SIG_GROUP_ 256 EngineGroup 1 RPM Temperature ThrottlePosition;
```

**Purpose:**
- Organize related signals
- UI/tool grouping
- Atomic update requirements
- Does **not** affect CAN message structure

---

## 14. Comments

```bnf
comments = {comment} ;
comment = 'CM_' (
    char_string |                             (* General comment *)
    'BU_' node_name char_string |            (* Node comment *)
    'BO_' message_id char_string |           (* Message comment *)
    'SG_' message_id signal_name char_string | (* Signal comment *)
    'EV_' env_var_name char_string           (* Env var comment *)
) ';' ;
```

**Examples:**
```
CM_ "CAN communication matrix for powertrain electronics" ;
CM_ BU_ Engine "Engine Control Module - Main powertrain controller" ;
CM_ BO_ 100 "Engine status and diagnostic data" ;
CM_ SG_ 100 EngineSpeed "Actual engine speed calculated over 720° crankshaft angle" ;
```

**Best Practices:**
- Describe purpose and context
- Document special conditions or constraints
- Reference related specifications
- Keep concise but informative

---

## 15. User-Defined Attributes

### 15.1 Attribute Definitions

```bnf
attribute_definition = 'BA_DEF_' object_type attribute_name attribute_value_type ';' ;
object_type = '' | 'BU_' | 'BO_' | 'SG_' | 'EV_' ;
attribute_name = '"' DBC_identifier '"' ;
attribute_value_type = 
    'INT' signed_integer signed_integer |
    'HEX' signed_integer signed_integer |
    'FLOAT' double double |
    'STRING' |
    'ENUM' char_string {',' char_string} ;
```

**Object Types:**
- Empty `''` = Network/global attribute
- `BU_` = Node attribute
- `BO_` = Message attribute
- `SG_` = Signal attribute
- `EV_` = Environment variable attribute

**Examples:**
```
BA_DEF_ BO_ "GenMsgCycleTime" INT 0 10000 ;
BA_DEF_ SG_ "GenSigStartValue" INT 0 255 ;
BA_DEF_ BO_ "VFrameFormat" ENUM "StandardCAN","ExtendedCAN","J1939PG" ;
BA_DEF_ "BusType" STRING ;
```

### 15.2 Attribute Defaults

```bnf
attribute_default = 'BA_DEF_DEF_' attribute_name attribute_value ';' ;
```

**Examples:**
```
BA_DEF_DEF_ "GenMsgCycleTime" 0 ;
BA_DEF_DEF_ "VFrameFormat" "StandardCAN" ;
BA_DEF_DEF_ "BusType" "" ;
```

### 15.3 Attribute Values

```bnf
attribute_value_for_object = 'BA_' attribute_name (
    attribute_value |                                    (* Network/global *)
    'BU_' node_name attribute_value |                   (* Node *)
    'BO_' message_id attribute_value |                  (* Message *)
    'SG_' message_id signal_name attribute_value |      (* Signal *)
    'EV_' env_var_name attribute_value                  (* Env var *)
) ';' ;
```

**Examples:**
```
BA_ "GenMsgCycleTime" BO_ 100 10 ;
BA_ "GenSigStartValue" SG_ 100 EngineSpeed 0 ;
BA_ "VFrameFormat" BO_ 2364540158 "J1939PG" ;
```

---

## 16. Extended Multiplexing

### 16.1 Purpose

Enables advanced multiplexing scenarios:
- Multiple multiplexer switches per message
- Multiple multiplexer values per signal

### 16.2 Syntax

```bnf
extended_multiplexing = {multiplexed_signal} ;
multiplexed_signal = 'SG_MUL_VAL_' message_id multiplexed_signal_name 
    multiplexor_switch_name multiplexor_value_ranges ';' ;
multiplexor_value_ranges = multiplexor_value_range {',' multiplexor_value_range} ;
multiplexor_value_range = unsigned_integer '-' unsigned_integer ;
```

### 16.3 Example

```
BO_ 500 ComplexMux : 8 ECM
 SG_ Mux1 M : 0|8@1+ (1,0) [0|255] "" Gateway
 SG_ Mux2 M : 8|8@1+ (1,0) [0|255] "" Gateway
 SG_ Signal_A m0 : 16|16@1+ (0.1,0) [0|100] "" Gateway
 SG_ Signal_B m1 : 16|16@1+ (0.1,0) [0|100] "" Gateway

SG_MUL_VAL_ 500 Signal_A Mux1 0-5,10-15 ;
SG_MUL_VAL_ 500 Signal_B Mux2 20-25 ;
```

**Explanation:**
- `Signal_A` is valid when `Mux1` is 0-5 or 10-15
- `Signal_B` is valid when `Mux2` is 20-25

---

## 17. Common Attributes

**Note:** The DBC specification does not define any standard attributes. All attributes must be defined using `BA_DEF_` before use. The attributes listed below are commonly used **tool-specific extensions** (primarily from Vector CANdb++) and are not part of the specification itself.

### 17.1 Common Tool-Specific Message Attributes

These attributes are commonly used in practice but are tool-specific:

| Attribute | Type | Description | Example |
|-----------|------|-------------|---------|
| `GenMsgCycleTime` | INT | Message cycle time (ms) | `10`, `100`, `1000` |
| `GenMsgSendType` | ENUM | Send behavior | `"cyclic"`, `"triggered"` |
| `GenMsgStartDelayTime` | INT | Startup delay (ms) | `0`, `100` |
| `VFrameFormat` | ENUM | Frame format | `"StandardCAN"`, `"ExtendedCAN"`, `"J1939PG"` |

### 17.2 Common Tool-Specific Signal Attributes

| Attribute | Type | Description | Example |
|-----------|------|-------------|---------|
| `GenSigStartValue` | INT/FLOAT | Initial value | `0`, `100.5` |
| `GenSigSendType` | ENUM | Send behavior | `"cyclic"`, `"onWrite"` |

### 17.3 J1939-Specific Attributes

For J1939 networks, tools commonly define:

```
BA_DEF_ BO_ "VFrameFormat" ENUM "StandardCAN","ExtendedCAN","J1939PG" ;
BA_DEF_ SG_ "SPN" INT 0 524287 ;       (* Suspect Parameter Number *)
BA_DEF_ BO_ "PGN" INT 0 262143 ;        (* Parameter Group Number *)

BA_ "VFrameFormat" BO_ 2364540158 "J1939PG" ;
BA_ "SPN" SG_ 2364540158 EngineSpeed 190 ;
BA_ "PGN" BO_ 2364540158 61444 ;
```

---

## 18. Examples

### 18.1 Minimal DBC File

```
VERSION ""

BS_:

BU_: Engine Gateway Dashboard

BO_ 100 EngineData : 8 Engine
 SG_ EngineSpeed : 0|16@1+ (1,0) [0|8000] "rpm" Gateway,Dashboard
 SG_ EngineTemp : 16|8@1- (1,-40) [-40|87] "°C" Gateway
 SG_ ThrottlePos : 24|8@1+ (0.4,0) [0|100] "%" Gateway

```

### 18.2 Complete DBC File with All Sections

```
VERSION "1.0"

NS_ : 
    NS_DESC_
    CM_
    BA_DEF_
    BA_
    VAL_
    BA_DEF_DEF_
    SIG_VALTYPE_
    BO_TX_BU_

BS_:

BU_: Engine Gateway Dashboard

VAL_TABLE_ GearPosition 0 "Park" 1 "Reverse" 2 "Neutral" 3 "Drive" 4 "Sport" ;

BO_ 100 EngineData : 8 Engine
 SG_ EngineSpeed : 0|16@1+ (0.25,0) [0|8000] "rpm" Gateway,Dashboard
 SG_ EngineTemp : 16|7@1+ (2,-50) [-50|150] "°C" Gateway
 SG_ IdleRunning : 23|1@1+ (1,0) [0|1] "" Gateway
 SG_ PetrolLevel : 24|8@1+ (1,0) [0|255] "l" Gateway
 SG_ EngForce : 32|16@1+ (1,0) [0|65535] "N" Gateway
 SG_ EngPower : 48|16@1+ (0.01,0) [0|150] "kW" Gateway

BO_ 200 GearboxData : 4 Gateway
 SG_ GearActual : 0|8@1+ (1,0) [0|5] "" Dashboard

CM_ "CAN communication matrix for powertrain electronics" ;
CM_ BU_ Engine "Engine Control Module" ;
CM_ BO_ 100 "Cyclic message with engine parameters" ;
CM_ SG_ 100 EngineSpeed "Actual engine speed calculated over 720° crankshaft angle" ;

BA_DEF_ BO_ "GenMsgCycleTime" INT 0 10000 ;
BA_DEF_ SG_ "GenSigStartValue" FLOAT 0 100000 ;
BA_DEF_ "BusType" STRING ;

BA_DEF_DEF_ "GenMsgCycleTime" 0 ;
BA_DEF_DEF_ "GenSigStartValue" 0 ;
BA_DEF_DEF_ "BusType" "CAN" ;

BA_ "GenMsgCycleTime" BO_ 100 10 ;
BA_ "GenSigStartValue" SG_ 100 EngineSpeed 0 ;
BA_ "BusType" "CAN" ;

VAL_ 100 IdleRunning 0 "Running" 1 "Idle" ;
VAL_ 200 GearActual 0 "Park" 1 "Reverse" 2 "Neutral" 3 "Drive" 4 "Sport" 5 "Manual" ;

```

### 18.3 Multiplexed Message Example

```
VERSION ""

BS_:

BU_: Sensor Gateway

BO_ 300 MultiplexedSensors : 8 Sensor
 SG_ SensorID M : 0|8@1+ (1,0) [0|255] "" Gateway
 SG_ Temperature m0 : 8|16@1- (0.1,-40) [-40|125] "°C" Gateway
 SG_ Pressure m1 : 8|16@1+ (0.01,0) [0|655.35] "kPa" Gateway
 SG_ Humidity m2 : 8|8@1+ (0.5,0) [0|100] "%" Gateway
 SG_ Voltage m3 : 8|16@1+ (0.001,0) [0|65.535] "V" Gateway

CM_ BO_ 300 "Multiplexed sensor data - SensorID selects active signal" ;

VAL_ 300 SensorID 0 "Temperature Sensor" 1 "Pressure Sensor" 2 "Humidity Sensor" 3 "Voltage Sensor" ;

```

### 18.4 Extended Multiplexing Example

```
VERSION ""

BS_:

BU_: ECU Tester

BO_ 400 DiagnosticData : 8 ECU
 SG_ DiagMode M : 0|4@1+ (1,0) [0|15] "" Tester
 SG_ SubMode M : 4|4@1+ (1,0) [0|15] "" Tester
 SG_ Data_A m0 : 8|32@1+ (1,0) [0|4294967295] "" Tester
 SG_ Data_B m1 : 8|32@1- (1,0) [-2147483648|2147483647] "" Tester
 SG_ Data_C m2 : 8|32@1+ (0.001,0) [0|4294967.295] "" Tester

SG_MUL_VAL_ 400 Data_A DiagMode 0-2 ;
SG_MUL_VAL_ 400 Data_A SubMode 0-5 ;
SG_MUL_VAL_ 400 Data_B DiagMode 3-5 ;
SG_MUL_VAL_ 400 Data_C DiagMode 6-8 ;
SG_MUL_VAL_ 400 Data_C SubMode 10-15 ;

```

---

## 19. Validation and Error Checking

### 19.1 Required Checks

**File Structure:**
- ✅ `BS_:` keyword must be present
- ✅ `BU_:` section must be present (can be empty)
- ✅ All sections must follow the defined order

**Uniqueness:**
- ✅ All node names must be unique
- ✅ All message IDs must be unique
- ✅ Signal names within a message must be unique

**References:**
- ✅ Message transmitter must exist in `BU_:` or be `Vector__XXX`
- ✅ Signal receivers must exist in `BU_:` or be `Vector__XXX`
- ✅ Multiplexer switch must exist in same message
- ✅ Attribute names must be defined before use

**Value Ranges:**
- ✅ `start_bit` must be < (8 × message_size)
- ✅ Signal must fit within message boundaries
- ✅ `factor` cannot be zero
- ✅ `minimum` ≤ `maximum`
- ✅ Extended CAN IDs must have bit 31 set

**Signal Overlap:**
- ✅ Non-multiplexed signals must not overlap in bit positions
- ✅ Multiplexed signals can share bits only with different mux values

### 19.2 Common Errors

| Error | Description | Solution |
|-------|-------------|----------|
| Missing `BS_:` | Required keyword missing | Add `BS_:` after `NS_` section |
| Duplicate message ID | Two messages with same ID | Use unique IDs for each message |
| Invalid start bit | Start bit ≥ (8 × DLC) | Reduce start bit or increase DLC |
| Factor is zero | Division by zero in conversion | Use non-zero factor |
| Unknown transmitter | Node not in `BU_:` list | Add node to `BU_:` or use `Vector__XXX` |
| Signal overflow | Signal extends beyond message | Reduce size or adjust start bit |
| Overlapping signals | Signals share bits without mux | Adjust bit positions or use multiplexing |

---

## 20. Best Practices

### 20.1 Naming Conventions

**Nodes:**
```
✅ GOOD: EngineControlModule, Gateway_1, ADAS_ECU
❌ AVOID: ECU1, node, temp
```

**Messages:**
```
✅ GOOD: EngineStatus, WheelSpeedData, BrakePressure
❌ AVOID: Msg1, data, x
```

**Signals:**
```
✅ GOOD: EngineSpeed_rpm, BrakePress_Front_Left, VehicleSpeed_kph
❌ AVOID: sig1, s, temp
```

### 20.2 Message Design

- **Keep DLC minimal** - Only use bytes needed for signals
- **Align signals to byte boundaries** when possible for efficiency
- **Group related signals** in same message
- **Use consistent byte order** throughout the network (prefer little-endian)
- **Document multiplexing** thoroughly with comments

### 20.3 Signal Scaling

- **Choose appropriate factor** for required precision
- **Avoid extreme factors** (very large or very small)
- **Use integer raw values** when possible for efficiency
- **Set meaningful ranges** for `minimum` and `maximum`
- **Include units** in signal definition

### 20.4 Documentation

- **Add comments** for all messages and critical signals
- **Define attributes** for cycle times and send types
- **Use value descriptions** for enumerated signals
- **Document special conditions** (e.g., "valid only when vehicle speed > 0")

### 20.5 Version Control

- **Include VERSION** statement with meaningful version string
- **Use timestamps** in version string: `"v2.3_2024-12-04"`
- **Document changes** in file header comment
- **Track compatibility** with ECU software versions

---

## 21. Common Pitfalls

### 21.1 Byte Order Confusion

**Problem:** Misunderstanding `@0` and `@1`

**Solution:** Remember:
- `@0` = Big-endian (Motorola) - MSB first
- `@1` = Little-endian (Intel) - LSB first

**Test:** Always verify with sample data

### 21.2 Sign Extension

**Problem:** Signed signals not properly decoded

**Example:**
```
Signal: 8-bit signed, raw value = 0xFF
Incorrect: 255 (unsigned interpretation)
Correct: -1 (signed interpretation)
```

**Solution:** Check `value_type` (`-` for signed) and apply two's complement

### 21.3 Start Bit for Big-Endian

**Problem:** Start bit for big-endian refers to MSB, not LSB

**Example:**
```
Signal: SG_ Test : 7|16@0+ ...
Starts at bit 7 (MSB) and extends backward through bytes
```

**Solution:** Carefully calculate physical bit ranges for big-endian

### 21.4 Factor Cannot Be Zero

**Problem:** Setting `factor = 0` causes division by zero

**Solution:** Always use non-zero factor (use `1` if no scaling needed)

### 21.5 Receiver Format Variations

**Problem:** Tools may interpret receiver lists differently

**Formats seen in practice:**
```
SG_ Signal : ... "unit" Node1,Node2      (comma-separated - per spec)
SG_ Signal : ... "unit" Node1 Node2      (space-separated - tool extension)
SG_ Signal : ... "unit" Node1, Node2     (comma+space - tool extension)
```

**Solution:** The specification defines only comma-separated receivers. Some tools accept space-separated as an extension, but for maximum compatibility, use comma-separated format as defined in the specification.

---

## 22. Tools and Parsers

### 22.1 Common DBC Tools

- **Vector CANdb++** - Original DBC editor
- **CANalyzer / CANoe** - Analysis and simulation
- **PCAN-View** - Free CAN bus monitor
- **Kvaser Database Editor** - DBC creation/editing
- **Intrepid Control Systems Vehicle Spy** - Analysis tool

### 22.2 Open Source Parsers

- **cantools** (Python) - Full DBC parser and encoder/decoder
- **python-can** (Python) - CAN library with DBC support
- **CANdb** (C++) - Fast C++ parser
- **dbc-rs** (Rust) - Rust DBC parser library

### 22.3 Parser Recommendations

When implementing a DBC parser:

1. **Be lenient with whitespace** - Accept both spaces and tabs
2. **Handle line endings** - Support both Windows (CRLF) and Unix (LF)
3. **Validate carefully** - Check all references and ranges
4. **Support comments** - Allow C-style comments `/* */` in some tools
5. **Warn on violations** - Don't silently ignore errors
6. **Test with real files** - Use automotive industry DBC files for testing

---

## 23. Glossary

| Term | Definition |
|------|------------|
| **CAN** | Controller Area Network - automotive bus standard |
| **DBC** | Database Container - CAN database file format |
| **ECU** | Electronic Control Unit - network node/device |
| **DLC** | Data Length Code - message size in bytes |
| **LSB** | Least Significant Bit |
| **MSB** | Most Significant Bit |
| **Big-endian** | Byte order with MSB first (Motorola) |
| **Little-endian** | Byte order with LSB first (Intel) |
| **Multiplexing** | Multiple signals sharing bit positions |
| **Raw value** | Integer value transmitted on bus |
| **Physical value** | Scaled/offset value in engineering units |
| **J1939** | SAE standard for heavy-duty vehicle networks |
| **CAN FD** | CAN with Flexible Data rate - up to 64 bytes |

---

## 24. References

### 24.1 Specifications

- Vector Informatik GmbH DBC File Format Specification (versions 1.0.0 - 1.0.1)
- ISO 11898: Road vehicles — Controller area network (CAN)
- SAE J1939: Recommended Practice for Serial Control and Communications Heavy Duty Vehicle Network

### 24.2 Related Standards

- **ARXML** - AUTOSAR XML format (successor to DBC for modern vehicles)
- **KCD** - Kayak CAN Database (XML-based alternative)
- **LDF** - LIN Description File (for LIN networks)

---

## License and Disclaimer

**Important:** The DBC file format is proprietary to Vector Informatik GmbH. This documentation is provided for educational and interoperability purposes based on publicly available information and reverse engineering.

**No Warranty:** This specification is provided "as is" without warranty of any kind. Always verify against actual tool behavior and industry standards.

**Contributions:** Community feedback and corrections are welcome to improve accuracy.
