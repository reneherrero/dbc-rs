#![allow(dead_code)] // Constants are conditionally used based on language feature selection

// ============================================================================
// Category labels
// ============================================================================

pub const INVALID_DATA_CATEGORY: &str = "Datenfehler";
pub const SIGNAL_ERROR_CATEGORY: &str = "Signalfehler";

// ============================================================================
// Version-related error messages
// ============================================================================

pub const VERSION_EMPTY: &str = "Leere Versionszeichenkette";
pub const VERSION_INVALID: &str = "Ungültige Versionszeichenkette";
pub const VERSION_PATCH_REQUIRES_MINOR: &str = "Patch-Version erfordert Nebenversion";

// ============================================================================
// DBC file-related error messages
// ============================================================================

pub const DBC_EMPTY_FILE: &str = "Leere DBC-Datei";
pub const DBC_NODES_NOT_DEFINED: &str = "Knoten (BU_) sind nicht definiert";

// ============================================================================
// Message-related error messages
// ============================================================================

pub const MESSAGE_NAME_EMPTY: &str = "Nachrichtenname darf nicht leer sein";
pub const MESSAGE_SENDER_EMPTY: &str = "Nachrichtensender darf nicht leer sein";
pub const MESSAGE_DLC_TOO_SMALL: &str = "Nachrichten-DLC muss mindestens 1 Byte betragen";
pub const MESSAGE_DLC_TOO_LARGE: &str = "Nachrichten-DLC darf 8 Bytes nicht überschreiten";
pub const MESSAGE_INVALID_FORMAT: &str = "Ungültiges Nachrichtenformat";
pub const MESSAGE_INVALID_ID: &str = "Ungültige Nachrichten-ID";
pub const MESSAGE_INVALID_DLC: &str = "Ungültiger DLC";
pub const MESSAGE_ID_OUT_OF_RANGE: &str = "Nachrichten-ID außerhalb des gültigen Bereichs";

// ============================================================================
// Signal-related error messages
// ============================================================================

pub const SIGNAL_NAME_EMPTY: &str = "Signalname darf nicht leer sein";
pub const SIGNAL_LENGTH_TOO_SMALL: &str = "Signallänge muss mindestens 1 Bit betragen";
pub const SIGNAL_LENGTH_TOO_LARGE: &str = "Signallänge darf 64 Bits nicht überschreiten";
pub const SIGNAL_OVERLAP: &str = "Signale überlappen sich in der Nachricht";

// ============================================================================
// Signal parsing error messages
// ============================================================================

pub const SIGNAL_PARSE_EXPECTED_SG: &str = "Erwartet 'SG_' am Zeilenanfang";
pub const SIGNAL_PARSE_MISSING_COLON: &str = "Fehlendes ':' in der Signaldefinition";
pub const SIGNAL_PARSE_MISSING_POSITION: &str = "Fehlende Positionsangabe";
pub const SIGNAL_PARSE_MISSING_REST: &str = "Fehlender Rest nach Positionsangabe";
pub const SIGNAL_PARSE_EXPECTED_AT: &str = "Erwartet '@' in startbit|length@...";
pub const SIGNAL_PARSE_EXPECTED_PIPE: &str = "Erwartet '|' in startbit|length";
pub const SIGNAL_PARSE_INVALID_START_BIT: &str = "Ungültiger start_bit";
pub const SIGNAL_PARSE_INVALID_LENGTH: &str = "Ungültige Länge";
pub const SIGNAL_PARSE_MISSING_BYTE_ORDER: &str = "Fehlende Byte-Reihenfolge";
pub const SIGNAL_PARSE_MISSING_SIGN: &str = "Fehlendes Vorzeichen";
pub const SIGNAL_PARSE_MISSING_CLOSING_PAREN: &str = "Fehlende ')' für factor,offset";
pub const SIGNAL_PARSE_MISSING_OPENING_PAREN: &str = "Fehlende '(' für factor,offset";
pub const SIGNAL_PARSE_MISSING_COMMA: &str = "Fehlendes ',' in factor,offset";
pub const SIGNAL_PARSE_INVALID_FACTOR: &str = "Ungültiger Faktor";
pub const SIGNAL_PARSE_INVALID_OFFSET: &str = "Ungültiger Offset";
pub const SIGNAL_PARSE_MISSING_CLOSING_BRACKET: &str = "Fehlende ']' für min|max";
pub const SIGNAL_PARSE_MISSING_OPENING_BRACKET: &str = "Fehlende '[' für min|max";
pub const SIGNAL_PARSE_MISSING_PIPE_IN_RANGE: &str = "Fehlendes '|' in min|max";
pub const SIGNAL_PARSE_INVALID_MIN: &str = "Ungültiges Min";
pub const SIGNAL_PARSE_INVALID_MAX: &str = "Ungültiges Max";
pub const SIGNAL_PARSE_EXPECTED_UNIT_QUOTE: &str = "Erwartet Beginn der 'unit'-Zeichenkette '\"'";

// ============================================================================
// Formatted error message templates
// ============================================================================

pub const FORMAT_DUPLICATE_MESSAGE_ID: &str =
    "Doppelte Nachrichten-ID: {} (Nachrichten '{}' und '{}')";
pub const FORMAT_SENDER_NOT_IN_NODES: &str =
    "Nachricht '{}' hat einen Sender '{}', der nicht in der Knotenliste steht";
pub const FORMAT_SIGNAL_EXTENDS_BEYOND_MESSAGE: &str = "Signal '{}' erstreckt sich über die Nachrichtengrenze hinaus: start_bit {} + length {} = {} > {} (DLC {} Bytes)";
pub const FORMAT_SIGNAL_EXTENDS_BEYOND_CAN: &str = "Signal erstreckt sich über die CAN-Nachrichtengrenze hinaus: start_bit {} + length {} = {} > 64";
pub const FORMAT_INVALID_RANGE: &str = "Ungültiger Bereich: min {} > max {}";
pub const FORMAT_UNKNOWN_BYTE_ORDER: &str = "Unbekannte Byte-Reihenfolge '{}'";
pub const FORMAT_UNKNOWN_SIGN: &str = "Unbekanntes Vorzeichen '{}'";
pub const FORMAT_PARSE_NUMBER_FAILED: &str = "Fehler beim Parsen der Zahl: {}";
pub const FORMAT_INVALID_UTF8: &str = "Ungültiges UTF-8: {}";
pub const FORMAT_READ_FAILED: &str = "Fehler beim Lesen: {}";
pub const FORMAT_MESSAGE_ID_OUT_OF_RANGE: &str = "Nachrichten-ID {} liegt außerhalb des gültigen Bereichs (Standard 11-Bit: 0-2047, Erweitert 29-Bit: 2048-536870911)";
pub const FORMAT_SIGNAL_OVERLAP: &str =
    "Signale '{}' und '{}' überlappen sich in der Nachricht '{}'";
