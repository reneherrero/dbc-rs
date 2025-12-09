#![allow(dead_code)] // Constants are conditionally used based on language feature selection

// ============================================================================
// Category labels
// ============================================================================

pub const INVALID_DATA_CATEGORY: &str = "データエラー";
pub const SIGNAL_ERROR_CATEGORY: &str = "信号エラー";
pub const MESSAGE_ERROR_CATEGORY: &str = "メッセージエラー";
pub const DBC_ERROR_CATEGORY: &str = "DBCエラー";
pub const VERSION_ERROR_CATEGORY: &str = "バージョンエラー";
pub const NODES_ERROR_CATEGORY: &str = "ノードエラー";

// ============================================================================
// Version-related error messages
// ============================================================================

pub const VERSION_EMPTY: &str = "空のバージョン文字列";
pub const VERSION_INVALID: &str = "無効なバージョン文字列";
pub const VERSION_MAJOR_REQUIRED: &str = "メジャーバージョンが必要です";
pub const VERSION_PATCH_REQUIRES_MINOR: &str = "パッチバージョンにはマイナーバージョンが必要です";

// ============================================================================
// DBC file-related error messages
// ============================================================================

pub const DBC_EMPTY_FILE: &str = "空のDBCファイル";
pub const DBC_VERSION_REQUIRED: &str = "バージョンが必要です";
pub const DBC_NODES_REQUIRED: &str = "ノードが必要です";
pub const DBC_NODES_NOT_DEFINED: &str = "ノード（BU_）が定義されていません";
pub const DBC_TOO_MANY_MESSAGES: &str = "メッセージが多すぎます：最大10000個まで許可されています";
pub const NODES_DUPLICATE_NAME: &str = "重複するノード名";
pub const NODES_TOO_MANY: &str = "ノードが多すぎます：最大256個まで許可されています";
pub const DUPLICATE_MESSAGE_ID: &str = "重複するメッセージID";
pub const SENDER_NOT_IN_NODES: &str = "メッセージ送信者がノードリストにありません";
pub const SIGNAL_EXTENDS_BEYOND_MESSAGE: &str = "信号がメッセージ境界を超えています";
pub const INVALID_RANGE: &str = "無効な範囲: min > max";
pub const PARSE_NUMBER_FAILED: &str = "数値の解析に失敗しました";
pub const INVALID_UTF8: &str = "無効なUTF-8";
pub const READ_FAILED: &str = "読み取りに失敗しました";
pub const VALUE_DESCRIPTION_MESSAGE_NOT_FOUND: &str =
    "値の説明が存在しないメッセージを参照しています";
pub const VALUE_DESCRIPTION_SIGNAL_NOT_FOUND: &str = "値の説明が存在しない信号を参照しています";
pub const EXPECTED_WHITESPACE: &str = "空白が期待されました";
pub const EXPECTED_PATTERN: &str = "パターンが期待されました";
pub const MESSAGE_TOO_MANY_SIGNALS: &str =
    "信号が多すぎます：メッセージあたり最大64個まで許可されています";
pub const SIGNAL_RECEIVERS_TOO_MANY: &str =
    "受信ノードが多すぎます：信号あたり最大64個まで許可されています";

// ============================================================================
// Message-related error messages
// ============================================================================

pub const MESSAGE_NAME_EMPTY: &str = "メッセージ名を空にすることはできません";
pub const MESSAGE_ID_REQUIRED: &str = "idが必要です";
pub const MESSAGE_DLC_REQUIRED: &str = "dlcが必要です";
pub const MESSAGE_SENDER_EMPTY: &str = "メッセージ送信者を空にすることはできません";
pub const MESSAGE_DLC_TOO_SMALL: &str = "メッセージDLCは少なくとも1バイトである必要があります";
pub const MESSAGE_DLC_TOO_LARGE: &str =
    "メッセージDLCは64バイト（CAN FD最大値）を超えることはできません";
pub const MESSAGE_INVALID_FORMAT: &str = "無効なメッセージ形式";
pub const MESSAGE_INVALID_ID: &str = "無効なメッセージID";
pub const MESSAGE_INVALID_DLC: &str = "無効なDLC";
pub const MESSAGE_ID_OUT_OF_RANGE: &str = "メッセージIDが有効範囲外です";

// ============================================================================
// Signal-related error messages
// ============================================================================

pub const SIGNAL_NAME_EMPTY: &str = "信号名を空にすることはできません";
pub const SIGNAL_START_BIT_REQUIRED: &str = "start_bitが必要です";
pub const SIGNAL_LENGTH_REQUIRED: &str = "lengthが必要です";
pub const SIGNAL_LENGTH_TOO_SMALL: &str = "信号長は少なくとも1ビットである必要があります";
pub const SIGNAL_LENGTH_TOO_LARGE: &str =
    "信号長は512ビット（CAN FD最大値）を超えることはできません";
pub const SIGNAL_OVERLAP: &str = "信号がメッセージ内で重複しています";
pub const SIGNAL_EXTENDS_BEYOND_DATA: &str = "信号がメッセージデータを超えて拡張しています";

// ============================================================================
// Signal parsing error messages
// ============================================================================

pub const SIGNAL_PARSE_EXPECTED_SG: &str = "行の先頭に'SG_'が必要です";
pub const SIGNAL_PARSE_MISSING_COLON: &str = "信号定義に':'がありません";
pub const SIGNAL_PARSE_MISSING_POSITION: &str = "位置指定がありません";
pub const SIGNAL_PARSE_MISSING_REST: &str = "位置指定の後に残りがありません";
pub const SIGNAL_PARSE_EXPECTED_AT: &str = "startbit|length@...に'@'が必要です";
pub const SIGNAL_PARSE_EXPECTED_PIPE: &str = "startbit|lengthに'|'が必要です";
pub const SIGNAL_PARSE_INVALID_START_BIT: &str = "無効なstart_bit";
pub const SIGNAL_PARSE_INVALID_LENGTH: &str = "無効な長さ";
pub const SIGNAL_PARSE_MISSING_BYTE_ORDER: &str = "バイト順序がありません";
pub const SIGNAL_PARSE_MISSING_SIGN: &str = "符号がありません";
pub const SIGNAL_PARSE_MISSING_CLOSING_PAREN: &str = "factor,offsetの')'がありません";
pub const SIGNAL_PARSE_MISSING_OPENING_PAREN: &str = "factor,offsetの'('がありません";
pub const SIGNAL_PARSE_MISSING_COMMA: &str = "factor,offsetに','がありません";
pub const SIGNAL_PARSE_INVALID_FACTOR: &str = "無効な係数";
pub const SIGNAL_PARSE_INVALID_OFFSET: &str = "無効なオフセット";
pub const SIGNAL_PARSE_MISSING_CLOSING_BRACKET: &str = "min|maxの']'がありません";
pub const SIGNAL_PARSE_MISSING_OPENING_BRACKET: &str = "min|maxの'['がありません";
pub const SIGNAL_PARSE_MISSING_PIPE_IN_RANGE: &str = "min|maxに'|'がありません";
pub const SIGNAL_PARSE_INVALID_MIN: &str = "無効な最小値";
pub const SIGNAL_PARSE_INVALID_MAX: &str = "無効な最大値";
pub const SIGNAL_PARSE_EXPECTED_UNIT_QUOTE: &str = "'unit'文字列の開始'\"'が必要です";
pub const SIGNAL_PARSE_UNIT_TOO_LONG: &str = "単位文字列が最大長256文字を超えています";
