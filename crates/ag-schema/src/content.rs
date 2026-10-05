use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::{SchemaError, StableId, Validate, nonempty, valid_external_url};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum Inline {
    Text {
        text: String,
        #[serde(default)]
        bold: bool,
        #[serde(default)]
        italic: bool,
        #[serde(default)]
        underline: bool,
        #[serde(default)]
        emphasis: bool,
        #[serde(default)]
        link: Option<String>,
    },
    SoftBreak,
}

pub type RichText = Vec<Inline>;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimelineItem {
    pub date: NaiveDate,
    pub content: RichText,
}

/// Declarative content only. No HTML, script, plugin or arbitrary font/size node.
/// Optional video blocks are deferred; private-video remains a storage class.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum ContentBlock {
    Heading {
        level: u8,
        content: RichText,
    },
    Paragraph {
        content: RichText,
    },
    Quote {
        content: RichText,
    },
    Signature {
        content: RichText,
    },
    Divider,
    Image {
        object_id: StableId,
        alt: String,
    },
    ImagePair {
        object_ids: [StableId; 2],
        alt: [String; 2],
    },
    Gallery {
        object_ids: Vec<StableId>,
        alt: Vec<String>,
    },
    Caption {
        content: RichText,
    },
    Timeline {
        items: Vec<TimelineItem>,
    },
    Callout {
        content: RichText,
    },
    DateStamp {
        date: NaiveDate,
    },
    Credits {
        content: RichText,
    },
    Spacer,
    Link {
        label: String,
        url: String,
    },
    Audio {
        object_id: StableId,
        caption: RichText,
    },
}

fn validate_text(text: &[Inline]) -> Result<(), SchemaError> {
    for inline in text {
        if let Inline::Text {
            link: Some(link), ..
        } = inline
            && !valid_external_url(link)
        {
            return Err(SchemaError::Invalid("inline link"));
        }
    }
    Ok(())
}

impl Validate for ContentBlock {
    fn validate(&self) -> Result<(), SchemaError> {
        match self {
            Self::Heading { level, content } => {
                if !(1..=6).contains(level) {
                    return Err(SchemaError::Invalid("heading level"));
                }
                validate_text(content)
            }
            Self::Paragraph { content }
            | Self::Quote { content }
            | Self::Signature { content }
            | Self::Caption { content }
            | Self::Callout { content }
            | Self::Credits { content } => validate_text(content),
            Self::Gallery { object_ids, alt } => {
                if object_ids.is_empty() || object_ids.len() != alt.len() {
                    Err(SchemaError::Invalid("gallery alternatives"))
                } else {
                    Ok(())
                }
            }
            Self::Timeline { items } => {
                for item in items {
                    validate_text(&item.content)?;
                }
                Ok(())
            }
            Self::Link { label, url } => {
                nonempty(label, "link label")?;
                if valid_external_url(url) {
                    Ok(())
                } else {
                    Err(SchemaError::Invalid("link URL"))
                }
            }
            Self::Audio { caption, .. } => validate_text(caption),
            _ => Ok(()),
        }
    }
}
