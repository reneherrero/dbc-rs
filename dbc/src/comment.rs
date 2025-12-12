/// Object type for comments
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommentObjectType {
    /// General comment (not associated with a specific object)
    General,
    /// Node (BU_) comment
    Node,
    /// Message (BO_) comment
    Message,
    /// Signal (SG_) comment
    Signal,
    /// Environment variable (EV_) comment
    EnvironmentVariable,
}

/// A comment in a DBC file
#[derive(Debug, Clone, PartialEq)]
pub struct Comment {
    object_type: CommentObjectType,
    object_name: Option<std::string::String>,
    object_id: Option<u32>,
    text: std::string::String,
}

impl Comment {
    pub(crate) fn new(
        object_type: CommentObjectType,
        object_name: Option<std::string::String>,
        object_id: Option<u32>,
        text: std::string::String,
    ) -> Self {
        Self {
            object_type,
            object_name,
            object_id,
            text,
        }
    }

    #[must_use]
    pub fn object_type(&self) -> CommentObjectType {
        self.object_type
    }

    #[must_use]
    pub fn object_name(&self) -> Option<&str> {
        self.object_name.as_deref()
    }

    #[must_use]
    pub fn object_id(&self) -> Option<u32> {
        self.object_id
    }

    #[must_use]
    pub fn text(&self) -> &str {
        self.text.as_str()
    }
}
