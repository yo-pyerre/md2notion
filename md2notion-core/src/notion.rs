use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlockType {
    Paragraph,
    Heading1,
    Heading2,
    Heading3,
    BulletedListItem,
    NumberedListItem,
    ToDo,
    Toggle,
    Code,
    Quote,
    Divider,
    Callout,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub object: String,
    #[serde(rename = "type")]
    pub block_type: BlockType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paragraph: Option<ParagraphBlock>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heading_1: Option<HeadingBlock>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heading_2: Option<HeadingBlock>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heading_3: Option<HeadingBlock>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bulleted_list_item: Option<ListItemBlock>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub numbered_list_item: Option<ListItemBlock>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_do: Option<ToDoBlock>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<CodeBlock>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote: Option<QuoteBlock>,
    // Divider has no content usually, effectively empty struct or just the type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub divider: Option<DividerBlock>, 
}

impl Block {
    pub fn new(block_type: BlockType) -> Self {
        Self {
            object: "block".to_string(),
            block_type,
            paragraph: None,
            heading_1: None,
            heading_2: None,
            heading_3: None,
            bulleted_list_item: None,
            numbered_list_item: None,
            to_do: None,
            code: None,
            quote: None,
            divider: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParagraphBlock {
    pub rich_text: Vec<RichText>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<Block>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeadingBlock {
    pub rich_text: Vec<RichText>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<Block>>, // Headings can have children in Notion now (toggle headings), but usually for simple md mapping we might not use it immediately, but good to have.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_toggleable: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListItemBlock {
    pub rich_text: Vec<RichText>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<Block>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToDoBlock {
    pub rich_text: Vec<RichText>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<Block>>,
    pub checked: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeBlock {
    pub rich_text: Vec<RichText>,
    pub language: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caption: Option<Vec<RichText>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteBlock {
    pub rich_text: Vec<RichText>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<Block>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DividerBlock {}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RichTextType {
    Text,
    Mention,
    Equation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RichText {
    #[serde(rename = "type")]
    pub text_type: RichTextType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<TextContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<Annotations>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plain_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub href: Option<String>,
}

impl RichText {
    pub fn text(content: String, link: Option<String>) -> Self {
        Self {
            text_type: RichTextType::Text,
            text: Some(TextContent {
                content: content.clone(),
                link: link.map(|l| Link { url: l }),
            }),
            annotations: Some(Annotations::default()),
            plain_text: Some(content),
            href: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextContent {
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link: Option<Link>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Link {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Annotations {
    pub bold: bool,
    pub italic: bool,
    pub strikethrough: bool,
    pub underline: bool,
    pub code: bool,
    pub color: String, // "default" or other colors
}

impl Annotations {
    pub fn default() -> Self {
        Self {
            bold: false,
            italic: false,
            strikethrough: false,
            underline: false,
            code: false,
            color: "default".to_string(),
        }
    }
}
