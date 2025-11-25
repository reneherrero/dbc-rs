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
pub const NODES_DUPLICATE_NAME: &str = "Nombre de nodo duplicado";

// ============================================================================
// Message-related error messages
// ============================================================================

pub const MESSAGE_NAME_EMPTY: &str = "El nombre del mensaje no puede estar vacío";
pub const MESSAGE_ID_REQUIRED: &str = "id es requerido";
pub const MESSAGE_DLC_REQUIRED: &str = "dlc es requerido";
pub const MESSAGE_SENDER_EMPTY: &str = "El remitente del mensaje no puede estar vacío";
pub const MESSAGE_DLC_TOO_SMALL: &str = "El DLC del mensaje debe ser de al menos 1 byte";
pub const MESSAGE_DLC_TOO_LARGE: &str = "El DLC del mensaje no puede exceder 8 bytes";
pub const MESSAGE_INVALID_FORMAT: &str = "Formato de mensaje inválido";
pub const MESSAGE_INVALID_ID: &str = "ID de mensaje inválido";
pub const MESSAGE_INVALID_DLC: &str = "DLC inválido";
pub const MESSAGE_ID_OUT_OF_RANGE: &str = "ID de mensaje fuera del rango válido";

// ============================================================================
// Signal-related error messages
// ============================================================================

pub const SIGNAL_NAME_EMPTY: &str = "El nombre de la señal no puede estar vacío";
pub const SIGNAL_START_BIT_REQUIRED: &str = "start_bit es requerido";
pub const SIGNAL_LENGTH_REQUIRED: &str = "length es requerido";
pub const SIGNAL_LENGTH_TOO_SMALL: &str = "La longitud de la señal debe ser de al menos 1 bit";
pub const SIGNAL_LENGTH_TOO_LARGE: &str = "La longitud de la señal no puede exceder 64 bits";
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

// ============================================================================
// Formatted error message templates
// ============================================================================

pub const FORMAT_DUPLICATE_MESSAGE_ID: &str = "ID de mensaje duplicado: {} (mensajes '{}' y '{}')";
pub const FORMAT_DUPLICATE_NODE_NAME: &str = "Nombre de nodo duplicado: '{}'";
pub const FORMAT_SENDER_NOT_IN_NODES: &str =
    "El mensaje '{}' tiene un remitente '{}' que no está en la lista de nodos";
pub const FORMAT_SIGNAL_EXTENDS_BEYOND_MESSAGE: &str = "La señal '{}' se extiende más allá del límite del mensaje: start_bit {} + length {} = {} > {} (DLC {} bytes)";
pub const FORMAT_SIGNAL_EXTENDS_BEYOND_CAN: &str =
    "La señal se extiende más allá del límite del mensaje CAN: start_bit {} + length {} = {} > 64";
pub const FORMAT_INVALID_RANGE: &str = "Rango inválido: min {} > max {}";
pub const FORMAT_UNKNOWN_BYTE_ORDER: &str = "Orden de bytes desconocido '{}'";
pub const FORMAT_UNKNOWN_SIGN: &str = "Signo desconocido '{}'";
pub const FORMAT_PARSE_NUMBER_FAILED: &str = "Error al analizar el número: {}";
pub const FORMAT_INVALID_UTF8: &str = "UTF-8 inválido: {}";
pub const FORMAT_READ_FAILED: &str = "Error al leer: {}";
pub const FORMAT_MESSAGE_ID_OUT_OF_RANGE: &str = "El ID de mensaje {} está fuera del rango válido (11 bits estándar: 0-2047, 29 bits extendido: 2048-536870911)";
pub const FORMAT_SIGNAL_OVERLAP: &str = "Las señales '{}' y '{}' se superponen en el mensaje '{}'";
