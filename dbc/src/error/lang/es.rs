#![allow(dead_code)] // Constants are conditionally used based on language feature selection

// ============================================================================
// Category labels
// ============================================================================

pub const INVALID_DATA_CATEGORY: &str = "Error de datos";
pub const SIGNAL_ERROR_CATEGORY: &str = "Error de señal";
pub const MESSAGE_ERROR_CATEGORY: &str = "Error de mensaje";
pub const DBC_ERROR_CATEGORY: &str = "Error DBC";
pub const VERSION_ERROR_CATEGORY: &str = "Error de versión";
pub const NODES_ERROR_CATEGORY: &str = "Error de nodos";

// ============================================================================
// Version-related error messages
// ============================================================================

pub const VERSION_EMPTY: &str = "Cadena de versión vacía";
pub const VERSION_INVALID: &str = "Cadena de versión inválida";
pub const VERSION_MAJOR_REQUIRED: &str = "versión mayor es requerida";
pub const VERSION_PATCH_REQUIRES_MINOR: &str = "La versión de parche requiere una versión menor";

// ============================================================================
// DBC file-related error messages
// ============================================================================

pub const DBC_EMPTY_FILE: &str = "Archivo DBC vacío";
pub const DBC_VERSION_REQUIRED: &str = "versión es requerida";
pub const DBC_NODES_REQUIRED: &str = "nodos son requeridos";
pub const DBC_NODES_NOT_DEFINED: &str = "Los nodos (BU_) no están definidos";
pub const DBC_TOO_MANY_MESSAGES: &str = "Demasiados mensajes: el máximo permitido es 10000";
pub const NODES_DUPLICATE_NAME: &str = "Nombre de nodo duplicado";
pub const NODES_TOO_MANY: &str = "Demasiados nodos: el máximo permitido es 256";
pub const MESSAGE_TOO_MANY_SIGNALS: &str =
    "Demasiadas señales: el máximo permitido es 64 por mensaje";
pub const SIGNAL_RECEIVERS_TOO_MANY: &str =
    "Demasiados nodos receptores: el máximo permitido es 64 por señal";

// ============================================================================
// Message-related error messages
// ============================================================================

pub const MESSAGE_NAME_EMPTY: &str = "El nombre del mensaje no puede estar vacío";
pub const MESSAGE_ID_REQUIRED: &str = "id es requerido";
pub const MESSAGE_DLC_REQUIRED: &str = "dlc es requerido";
pub const MESSAGE_SENDER_EMPTY: &str = "El remitente del mensaje no puede estar vacío";
pub const MESSAGE_DLC_TOO_SMALL: &str = "El DLC del mensaje debe ser de al menos 1 byte";
pub const MESSAGE_DLC_TOO_LARGE: &str =
    "El DLC del mensaje no puede exceder 64 bytes (máximo CAN FD)";
pub const FORMAT_MESSAGE_DLC_TOO_SMALL: &str =
    "El mensaje '{}' (ID {}) tiene DLC {}, debe ser de al menos 1 byte. Use DLC entre 1 y 64 bytes";
pub const FORMAT_MESSAGE_DLC_TOO_LARGE: &str = "El mensaje '{}' (ID {}) tiene DLC {}, no puede exceder 64 bytes (máximo CAN FD). Use DLC entre 1 y 64 bytes";
pub const MESSAGE_INVALID_FORMAT: &str = "Formato de mensaje inválido";
pub const MESSAGE_INVALID_ID: &str = "ID de mensaje inválido";
pub const MESSAGE_INVALID_DLC: &str = "DLC inválido";
pub const FORMAT_MESSAGE_INVALID_ID: &str =
    "ID de mensaje inválido '{}'. Se esperaba un número válido (0-536,870,911 para IDs extendidos)";
pub const FORMAT_MESSAGE_INVALID_DLC: &str =
    "DLC inválido '{}' para el mensaje '{}' (ID {}). Se esperaba un número entre 1 y 64";
pub const MESSAGE_ID_OUT_OF_RANGE: &str = "ID de mensaje fuera del rango válido";

// ============================================================================
// Signal-related error messages
// ============================================================================

pub const SIGNAL_NAME_EMPTY: &str = "El nombre de la señal no puede estar vacío";
pub const SIGNAL_START_BIT_REQUIRED: &str = "start_bit es requerido";
pub const SIGNAL_LENGTH_REQUIRED: &str = "length es requerido";
pub const SIGNAL_LENGTH_TOO_SMALL: &str = "La longitud de la señal debe ser de al menos 1 bit";
pub const SIGNAL_LENGTH_TOO_LARGE: &str =
    "La longitud de la señal no puede exceder 512 bits (máximo CAN FD)";
pub const FORMAT_SIGNAL_LENGTH_TOO_SMALL: &str = "La señal '{}' tiene longitud {} bits, debe ser de al menos 1 bit. Use longitud entre 1 y 512 bits";
pub const FORMAT_SIGNAL_LENGTH_TOO_LARGE: &str = "La señal '{}' tiene longitud {} bits, no puede exceder 512 bits (máximo CAN FD). Use longitud entre 1 y 512 bits";
pub const SIGNAL_OVERLAP: &str = "Las señales se superponen en el mensaje";

// ============================================================================
// Signal parsing error messages
// ============================================================================

pub const SIGNAL_PARSE_EXPECTED_SG: &str = "Se esperaba 'SG_' al inicio de la línea";
pub const SIGNAL_PARSE_MISSING_COLON: &str = "Falta ':' en la definición de señal";
pub const SIGNAL_PARSE_MISSING_POSITION: &str = "Especificación de posición faltante";
pub const SIGNAL_PARSE_MISSING_REST: &str =
    "Falta el resto después de la especificación de posición";
pub const SIGNAL_PARSE_EXPECTED_AT: &str = "Se esperaba '@' en startbit|length@...";
pub const SIGNAL_PARSE_EXPECTED_PIPE: &str = "Se esperaba '|' en startbit|length";
pub const SIGNAL_PARSE_INVALID_START_BIT: &str = "start_bit inválido";
pub const SIGNAL_PARSE_INVALID_LENGTH: &str = "Longitud inválida";
pub const FORMAT_SIGNAL_PARSE_INVALID_START_BIT: &str =
    "La señal '{}' tiene start_bit {}, debe estar entre 0 y 511. Use start_bit entre 0 y 511";
pub const FORMAT_SIGNAL_PARSE_INVALID_LENGTH: &str =
    "Longitud inválida '{}' para la señal '{}'. Se esperaba un número entre 1 y 512";
pub const SIGNAL_PARSE_MISSING_BYTE_ORDER: &str = "Orden de bytes faltante";
pub const SIGNAL_PARSE_MISSING_SIGN: &str = "Signo faltante";
pub const SIGNAL_PARSE_MISSING_CLOSING_PAREN: &str = "Falta ')' para factor,offset";
pub const SIGNAL_PARSE_MISSING_OPENING_PAREN: &str = "Falta '(' para factor,offset";
pub const SIGNAL_PARSE_MISSING_COMMA: &str = "Falta ',' en factor,offset";
pub const SIGNAL_PARSE_INVALID_FACTOR: &str = "Factor inválido";
pub const SIGNAL_PARSE_INVALID_OFFSET: &str = "Desplazamiento inválido";
pub const SIGNAL_PARSE_MISSING_CLOSING_BRACKET: &str = "Falta ']' para min|max";
pub const SIGNAL_PARSE_MISSING_OPENING_BRACKET: &str = "Falta '[' para min|max";
pub const SIGNAL_PARSE_MISSING_PIPE_IN_RANGE: &str = "Falta '|' en min|max";
pub const SIGNAL_PARSE_INVALID_MIN: &str = "Min inválido";
pub const SIGNAL_PARSE_INVALID_MAX: &str = "Max inválido";
pub const SIGNAL_PARSE_EXPECTED_UNIT_QUOTE: &str = "Se esperaba el inicio de la cadena 'unit' '\"'";
pub const SIGNAL_PARSE_UNIT_TOO_LONG: &str =
    "La cadena de unidad excede la longitud máxima de 256 caracteres";

// ============================================================================
// Formatted error message templates
// ============================================================================

pub const FORMAT_DUPLICATE_MESSAGE_ID: &str = "ID de mensaje duplicado: {} (mensajes '{}' y '{}')";
pub const FORMAT_DUPLICATE_NODE_NAME: &str = "Nombre de nodo duplicado: '{}'";
pub const FORMAT_SENDER_NOT_IN_NODES: &str =
    "El mensaje '{}' tiene un remitente '{}' que no está en la lista de nodos";
pub const FORMAT_SIGNAL_EXTENDS_BEYOND_MESSAGE: &str = "La señal '{}' se extiende más allá del límite del mensaje: start_bit {} + length {} = {} > {} (DLC {} bytes). {}";
pub const SUGGEST_CAN_FD: &str =
    "Considere usar CAN FD con DLC {} o superior (requiere DLC > 8 bytes)";
pub const SUGGEST_INCREASE_DLC: &str = "Considere aumentar DLC a {} bytes";
pub const FORMAT_INVALID_RANGE: &str = "Rango inválido: min {} > max {}";
pub const FORMAT_UNKNOWN_BYTE_ORDER: &str = "Orden de bytes desconocido '{}'";
pub const FORMAT_UNKNOWN_SIGN: &str = "Signo desconocido '{}'";
pub const FORMAT_PARSE_NUMBER_FAILED: &str = "Error al analizar el número: {}";
pub const FORMAT_INVALID_UTF8: &str = "UTF-8 inválido: {}";
pub const FORMAT_READ_FAILED: &str = "Error al leer: {}";
pub const FORMAT_MESSAGE_ID_OUT_OF_RANGE: &str = "El ID de mensaje {} ({} decimal) está fuera del rango válido (11 bits estándar: 0x000-0x7FF (0-2,047 decimal), 29 bits extendido: 0x0000_0000-0x1FFF_FFFF (0-536,870,911 decimal))";
pub const FORMAT_SIGNAL_OVERLAP: &str =
    "Las señales '{}' y '{}' se superponen en el mensaje '{}'. {}";
pub const SUGGEST_MULTIPLEXING: &str = "Nota: Si estas señales están multiplexadas (activas en diferentes momentos), se requiere soporte de multiplexación de señales (aún no implementado)";
pub const FORMAT_LINE_NUMBER: &str = "{} (línea {})";
