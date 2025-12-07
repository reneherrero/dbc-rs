#![allow(dead_code)] // Constants are conditionally used based on language feature selection

// ============================================================================
// Category labels
// ============================================================================

pub const INVALID_DATA_CATEGORY: &str = "Erreur de données";
pub const SIGNAL_ERROR_CATEGORY: &str = "Erreur de signal";
pub const MESSAGE_ERROR_CATEGORY: &str = "Erreur de message";
pub const DBC_ERROR_CATEGORY: &str = "Erreur DBC";
pub const VERSION_ERROR_CATEGORY: &str = "Erreur de version";
pub const NODES_ERROR_CATEGORY: &str = "Erreur de nœuds";

// ============================================================================
// Version-related error messages
// ============================================================================

pub const VERSION_EMPTY: &str = "Chaîne de version vide";
pub const VERSION_INVALID: &str = "Chaîne de version invalide";
pub const VERSION_MAJOR_REQUIRED: &str = "version majeure est requise";
pub const VERSION_PATCH_REQUIRES_MINOR: &str =
    "La version de correctif nécessite une version mineure";

// ============================================================================
// DBC file-related error messages
// ============================================================================

pub const DBC_EMPTY_FILE: &str = "Fichier DBC vide";
pub const DBC_VERSION_REQUIRED: &str = "version est requise";
pub const DBC_NODES_REQUIRED: &str = "nœuds sont requis";
pub const DBC_NODES_NOT_DEFINED: &str = "Les nœuds (BU_) ne sont pas définis";
pub const DBC_TOO_MANY_MESSAGES: &str = "Trop de messages : le maximum autorisé est de 10000";
pub const NODES_DUPLICATE_NAME: &str = "Nom de nœud en double";
pub const NODES_TOO_MANY: &str = "Trop de nœuds : le maximum autorisé est de 256";
pub const MESSAGE_TOO_MANY_SIGNALS: &str =
    "Trop de signaux : le maximum autorisé est de 64 par message";
pub const SIGNAL_RECEIVERS_TOO_MANY: &str =
    "Trop de nœuds récepteurs : le maximum autorisé est de 64 par signal";

// ============================================================================
// Message-related error messages
// ============================================================================

pub const MESSAGE_NAME_EMPTY: &str = "Le nom du message ne peut pas être vide";
pub const MESSAGE_ID_REQUIRED: &str = "id est requis";
pub const MESSAGE_DLC_REQUIRED: &str = "dlc est requis";
pub const MESSAGE_SENDER_EMPTY: &str = "L'expéditeur du message ne peut pas être vide";
pub const MESSAGE_DLC_TOO_SMALL: &str = "Le DLC du message doit être d'au moins 1 octet";
pub const MESSAGE_DLC_TOO_LARGE: &str =
    "Le DLC du message ne peut pas dépasser 64 octets (maximum CAN FD)";
pub const FORMAT_MESSAGE_DLC_TOO_SMALL: &str = "Le message '{}' (ID {}) a un DLC {}, doit être d'au moins 1 octet. Utilisez un DLC entre 1 et 64 octets";
pub const FORMAT_MESSAGE_DLC_TOO_LARGE: &str = "Le message '{}' (ID {}) a un DLC {}, ne peut pas dépasser 64 octets (maximum CAN FD). Utilisez un DLC entre 1 et 64 octets";
pub const MESSAGE_INVALID_FORMAT: &str = "Format de message invalide";
pub const MESSAGE_INVALID_ID: &str = "ID de message invalide";
pub const MESSAGE_INVALID_DLC: &str = "DLC invalide";
pub const FORMAT_MESSAGE_INVALID_ID: &str = "ID de message invalide '{}'. Un nombre valide était attendu (0-536,870,911 pour les IDs étendus)";
pub const FORMAT_MESSAGE_INVALID_DLC: &str =
    "DLC invalide '{}' pour le message '{}' (ID {}). Un nombre entre 1 et 64 était attendu";
pub const MESSAGE_ID_OUT_OF_RANGE: &str = "ID de message hors de la plage valide";

// ============================================================================
// Signal-related error messages
// ============================================================================

pub const SIGNAL_NAME_EMPTY: &str = "Le nom du signal ne peut pas être vide";
pub const SIGNAL_START_BIT_REQUIRED: &str = "start_bit est requis";
pub const SIGNAL_LENGTH_REQUIRED: &str = "length est requis";
pub const SIGNAL_LENGTH_TOO_SMALL: &str = "La longueur du signal doit être d'au moins 1 bit";
pub const SIGNAL_LENGTH_TOO_LARGE: &str =
    "La longueur du signal ne peut pas dépasser 512 bits (maximum CAN FD)";
pub const FORMAT_SIGNAL_LENGTH_TOO_SMALL: &str = "Le signal '{}' a une longueur de {} bits, doit être d'au moins 1 bit. Utilisez une longueur entre 1 et 512 bits";
pub const FORMAT_SIGNAL_LENGTH_TOO_LARGE: &str = "Le signal '{}' a une longueur de {} bits, ne peut pas dépasser 512 bits (maximum CAN FD). Utilisez une longueur entre 1 et 512 bits";
pub const SIGNAL_OVERLAP: &str = "Les signaux se chevauchent dans le message";

// ============================================================================
// Signal parsing error messages
// ============================================================================

pub const SIGNAL_PARSE_EXPECTED_SG: &str = "Attendu 'SG_' au début de la ligne";
pub const SIGNAL_PARSE_MISSING_COLON: &str = "Manquant ':' dans la définition du signal";
pub const SIGNAL_PARSE_MISSING_POSITION: &str = "Spécification de position manquante";
pub const SIGNAL_PARSE_MISSING_REST: &str = "Reste manquant après la spécification de position";
pub const SIGNAL_PARSE_EXPECTED_AT: &str = "Attendu '@' dans startbit|length@...";
pub const SIGNAL_PARSE_EXPECTED_PIPE: &str = "Attendu '|' dans startbit|length";
pub const SIGNAL_PARSE_INVALID_START_BIT: &str = "start_bit invalide";
pub const SIGNAL_PARSE_INVALID_LENGTH: &str = "Longueur invalide";
pub const FORMAT_SIGNAL_PARSE_INVALID_START_BIT: &str = "Le signal '{}' a un start_bit {}, doit être entre 0 et 511. Utilisez un start_bit entre 0 et 511";
pub const FORMAT_SIGNAL_PARSE_INVALID_LENGTH: &str =
    "Longueur invalide '{}' pour le signal '{}'. Un nombre entre 1 et 512 était attendu";
pub const SIGNAL_PARSE_MISSING_BYTE_ORDER: &str = "Ordre des octets manquant";
pub const SIGNAL_PARSE_MISSING_SIGN: &str = "Signe manquant";
pub const SIGNAL_PARSE_MISSING_CLOSING_PAREN: &str = "Manquant ')' pour factor,offset";
pub const SIGNAL_PARSE_MISSING_OPENING_PAREN: &str = "Manquant '(' pour factor,offset";
pub const SIGNAL_PARSE_MISSING_COMMA: &str = "Manquant ',' dans factor,offset";
pub const SIGNAL_PARSE_INVALID_FACTOR: &str = "Facteur invalide";
pub const SIGNAL_PARSE_INVALID_OFFSET: &str = "Décalage invalide";
pub const SIGNAL_PARSE_MISSING_CLOSING_BRACKET: &str = "Manquant ']' pour min|max";
pub const SIGNAL_PARSE_MISSING_OPENING_BRACKET: &str = "Manquant '[' pour min|max";
pub const SIGNAL_PARSE_MISSING_PIPE_IN_RANGE: &str = "Manquant '|' dans min|max";
pub const SIGNAL_PARSE_INVALID_MIN: &str = "Min invalide";
pub const SIGNAL_PARSE_INVALID_MAX: &str = "Max invalide";
pub const SIGNAL_PARSE_EXPECTED_UNIT_QUOTE: &str = "Attendu le début de la chaîne 'unit' '\"'";
pub const SIGNAL_PARSE_UNIT_TOO_LONG: &str =
    "La chaîne d'unité dépasse la longueur maximale de 256 caractères";

// ============================================================================
// Formatted error message templates
// ============================================================================

pub const FORMAT_DUPLICATE_MESSAGE_ID: &str =
    "ID de message en double : {} (messages '{}' et '{}')";
pub const FORMAT_DUPLICATE_NODE_NAME: &str = "Nom de nœud en double : '{}'";
pub const FORMAT_SENDER_NOT_IN_NODES: &str =
    "Le message '{}' a un expéditeur '{}' qui n'est pas dans la liste des nœuds";
pub const FORMAT_SIGNAL_EXTENDS_BEYOND_MESSAGE: &str = "Le signal '{}' dépasse la limite du message : start_bit {} + length {} = {} > {} (DLC {} octets). {}";
pub const SUGGEST_CAN_FD: &str =
    "Envisagez d'utiliser CAN FD avec un DLC {} ou supérieur (nécessite DLC > 8 octets)";
pub const SUGGEST_INCREASE_DLC: &str = "Envisagez d'augmenter le DLC à {} octets";
pub const FORMAT_INVALID_RANGE: &str = "Plage invalide : min {} > max {}";
pub const FORMAT_UNKNOWN_BYTE_ORDER: &str = "Ordre des octets inconnu '{}'";
pub const FORMAT_UNKNOWN_SIGN: &str = "Signe inconnu '{}'";
pub const FORMAT_PARSE_NUMBER_FAILED: &str = "Échec de l'analyse du nombre : {}";
pub const FORMAT_INVALID_UTF8: &str = "UTF-8 invalide : {}";
pub const FORMAT_READ_FAILED: &str = "Échec de la lecture : {}";
pub const FORMAT_MESSAGE_ID_OUT_OF_RANGE: &str = "L'ID de message {} ({} décimal) est hors de la plage valide (11 bits standard : 0x000-0x7FF (0-2,047 décimal), 29 bits étendu : 0x0000_0000-0x1FFF_FFFF (0-536,870,911 décimal))";
pub const FORMAT_SIGNAL_OVERLAP: &str =
    "Les signaux '{}' et '{}' se chevauchent dans le message '{}'. {}";
pub const SUGGEST_MULTIPLEXING: &str = "Note : Si ces signaux sont multiplexés (actifs à des moments différents), le support de multiplexage de signaux est requis (pas encore implémenté)";
pub const FORMAT_LINE_NUMBER: &str = "{} (ligne {})";
