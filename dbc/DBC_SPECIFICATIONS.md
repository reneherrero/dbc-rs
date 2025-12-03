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
| `char_string`        | an arbitrary string consisting of any printable characters except double hyphens (`"`) and backslashes (`\`). Control characters like Line Feed, Horizontal Tab, etc. are tolerated, but their interpretation depends on the application. |
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
version = ['VERSION' '"' { CANdb_version_string } '"' ];
```

## New Symbols

```bnf
new_symbols = [ '_NS' ':' ['CM_'] ['BA_DEF_'] ['BA_'] ['VAL_']
['CAT_DEF_'] ['CAT_'] ['FILTER'] ['BA_DEF_DEF_'] ['EV_DATA_']
['ENVVAR_DATA_'] ['SGTYPE_'] ['SGTYPE_VAL_'] ['BA_DEF_SGTYPE_']
['BA_SGTYPE_'] ['SIG_TYPE_REF_'] ['VAL_TABLE_'] ['SIG_GROUP_']
['SIG_VALTYPE_'] ['SIGTYPE_VALTYPE_'] ['BO_TX_BU_']
['BA_DEF_REL_'] ['BA_REL_'] ['BA_DEF_DEF_REL_'] ['BU_SG_REL_']
['BU_EV_REL_'] ['BU_BO_REL_'] [SG_MUL_VAL_'] ];
```

# Bit Timing Definition

The bit timing section defines the baudrate and the settings of the BTR registers of the network. This section is obsolete and no longer used. Nevertheless, the keyword `BS_` must appear in the DBC file.

```bnf
bit_timing = 'BS_:' [baudrate ':' BTR1 ',' BTR2 ] ;
baudrate = unsigned_integer ;
BTR1 = unsigned_integer ;
BTR2 = unsigned_integer ;
```

# Node Definitions

The node section defines the names of all nodes that participate in the CAN network. All node names defined in this section must be unique.

```bnf
nodes = 'BU_:' {node_name} ;
node_name = DBC_identifier ;
```

# Value Table Definitions

The value table section defines global value tables. Value descriptions in these tables map signal raw values to human-readable text encodings. In commonly used DBC files, global value tables are typically not used; instead, value descriptions are defined independently for each signal.

```
value_tables = {value_table} ;
value_table = 'VAL_TABLE_' value_table_name {value_description} ';' ;
value_table_name = DBC_identifier ;
```

## Value Descriptions (Value Encodings)

A value description maps a single numeric value to a human-readable textual description. The value may be either a signal raw value transmitted on the CAN bus or the value of an environment variable used in remaining bus simulation.

```bnf
value_description = unsigned_integer char_string ;
```

The message section defines all CAN frames (messages) in the network, including their properties and the signals contained within each frame.

# Message Definitions

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

The `start_bit` value specifies the bit position of the signal within the message's data field. For signals with Intel byte order (little-endian), the position of the least significant bit is specified. For signals with Motorola byte order (big-endian), the position of the most significant bit is specified. Bits are numbered in a sawtooth pattern. The `start_bit` must be in the range of 0 to (8 × `message_size` - 1).

```bnf
start_bit = unsigned_integer ;
```

The `signal_size` specifies the length of the signal in bits.

```bnf
byte_order = '0' | '1' ; (* 0=big endian, 1=little endian *)
```

The `byte_order` is `0` for Motorola (big-endian) or `1` for Intel (little-endian).

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
receiver = node_name | 'Vector__XXX' ;
```

The receiver name specifies the node that receives the signal. The receiver name must be defined in the node section. If the signal has no receiver, the string `Vector__XXX` must be used.

Signals with value types `float` and `double` have additional entries in the `signal_extended_value_type_list` section.

```bnf
signal_extended_value_type_list = 'SIG_VALTYPE_' message_id signal_name signal_extended_value_type ';' ;
signal_extended_value_type = '0' | '1' | '2' | '3' ; (* 0=signed or unsigned integer, 1=32-bit IEEE-float, 2=64-bit IEEE-double *)
```

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
environment_variables_data = environment_variable_data ;
environment_variable_data = 'ENVVAR_DATA_' env_var_name ':'' data_size ';' ;
data_size = unsigned_integer ;
```

### Environment Variable Value Descriptions

Environment variable value descriptions provide textual representations for specific variable values.

```bnf
value_descriptions_for_env_var = 'VAL_' env_var_aname { value_description } ';' ;
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
signal_groups = 'SIG_GROUP_' message_id signal_group_name repetitions ':' { signal_name } ';' ;
signal_group_name = DBC_identifier ;
repetitions = unsigned_integer ;
```

# Comment Definitions

The comment section contains comments for objects. Each object with a comment has an entry in this section, identified by the object's type.

```bnf
comments = {comment} ;
```

```bnf
comment = 'CM_' (char_string 'BU_' node_name char_string 'BO_' message_id char_string 'SG_' message_id signal_name char_string 'EV_' env_var_name char_string) ';' ;
```

# User Defined Attribute Definitions

User-defined attributes extend the object properties in a DBC file. These attributes must be defined using an attribute definition with a default value. For each object with a value assigned to the attribute, an attribute value entry must be defined. If no attribute value entry is defined for an object, the object's attribute value defaults to the attribute's default value.

### Attribute Definitions

```bnf
attribute_definitions = { attribute_definition } ;
attribute_definition = 'BA_DEF_' object_type attribute_name attribute_value_type ';' ;
object_type = '' | 'BU_' | 'BO_' | 'SG_' | 'EV_' ;
attribute_name = '"' DBC_identifier '"' ;
attribute_value_type = 'INT' signed_integer signed_integer | 'HEX' signed_integer signed_integer | 'FLOAT' double double | 'STRING' | 'ENUM' [char_string (',' char_string)]
attribute_defaults = { attribute_default } ;
attribute_default = 'BA_DEF_DEF_' attribute_name attribute_value ';' ;
attribute_value = unsigned_integer | signed_integer | double | char_string ;
```

### Attribute Values

```
attribute_values = { attribute_value_for_object } ;
attribute_value_for_object = 'BA_' attribute_name (attribute_value | 'BU_' node_name attribute_value | 'BO_' message_id attribute_value | 'SG_' message_id signal_name attribute_value | 'EV_' env_var_name attribute_value) ';' ;
```

# Extended Multiplexing

Extended multiplexing enables defining multiple multiplexer switches within a single message. It also allows using multiple multiplexer switch values for each multiplexed signal.

The extended multiplexing section contains multiplexed signals for which following conditions were fulfilled:

- The multiplexed signal is multiplexed by more than one multiplexer switch value
- The multiplexed signal belongs to a message which contains more than one multiplexor switch

```bnf
extended multiplexing = {multiplexed signal} ;
multiplexed signal = SG_MUL_VAL_ message_id multiplexed_signal_name multiplexor_switch_name multiplexor_value_ranges ';' ;
message_id = unsigned_integer ;
multiplexed_signal_name = DBC_identifier ;
multiplexor_switch_name = DBC_identifier ;
multiplexor_value_ranges = {multiplexor_value_range} ;
multiplexor_value_range = unsigned_integer '-' unsigned_integer ;
```

# Examples

```bnf
VERSION ""

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

BS_:

BU_: Engine Gateway

BO_ 100 EngineData: 8 Engine
 SG_ PetrolLevel : 24|8@1+ (1,0) [0|255] "l" Gateway
 SG_ EngPower : 48|16@1+ (0.01,0) [0|150] "kW" Gateway
 SG_ EngForce : 32|16@1+ (1,0) [0|0] "N" Gateway
 SG_ IdleRunning : 23|1@1+ (1,0) [0|0] "" Gateway
 SG_ EngTemp : 16|7@1+ (2,-50) [-50|150] "degC" Gateway
 SG_ EngSpeed : 0|16@1+ (1,0) [0|8000] "rpm" Gateway

CM_ "CAN communication matrix for power train electronics
```
