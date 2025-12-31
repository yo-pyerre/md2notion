use anyhow::Result;
use pulldown_cmark::{Event, Parser, Tag};
use crate::notion::{Block, BlockType, HeadingBlock, ListItemBlock, ParagraphBlock, RichText, Annotations};

pub fn parse_markdown(content: &str) -> Result<Vec<Block>> {
    let parser = Parser::new(content);
    let mut iter = parser.into_iter();
    Ok(parse_events(&mut iter))
}

fn parse_events<'a, I>(iter: &mut I) -> Vec<Block>
where
    I: Iterator<Item = Event<'a>>,
{
    let mut blocks = Vec::new();
    let mut inline_accumulator: Vec<Event<'a>> = Vec::new();

    while let Some(event) = iter.next() {
        let is_block = if let Event::Start(tag) = &event {
            is_block_tag(tag)
        } else {
            false
        };

        if is_block {
            // Flush accumulator
            if !inline_accumulator.is_empty() {
                let rich_text = parse_inline(inline_accumulator.drain(..));
                let mut block = Block::new(BlockType::Paragraph);
                block.paragraph = Some(ParagraphBlock {
                    rich_text,
                    children: None,
                    color: None,
                });
                blocks.push(block);
            }

            match event {
                Event::Start(tag) => {
                    match tag {
                        Tag::Paragraph => {
                            let inline_events = collect_events_until(iter, &Tag::Paragraph);
                            let rich_text = parse_inline(inline_events.into_iter());
                            let mut block = Block::new(BlockType::Paragraph);
                            block.paragraph = Some(ParagraphBlock {
                                rich_text,
                                children: None,
                                color: None,
                            });
                            blocks.push(block);
                        }
                        Tag::Heading(level, _, _) => {
                            let inline_events = collect_events_until(iter, &tag);
                            let rich_text = parse_inline(inline_events.into_iter());

                            let (block_type, heading_block) = match level {
                                pulldown_cmark::HeadingLevel::H1 => (
                                    BlockType::Heading1,
                                    HeadingBlock {
                                        rich_text,
                                        children: None,
                                        color: None,
                                        is_toggleable: None,
                                    },
                                ),
                                pulldown_cmark::HeadingLevel::H2 => (
                                    BlockType::Heading2,
                                    HeadingBlock {
                                        rich_text,
                                        children: None,
                                        color: None,
                                        is_toggleable: None,
                                    },
                                ),
                                _ => (
                                    BlockType::Heading3,
                                    HeadingBlock {
                                        rich_text,
                                        children: None,
                                        color: None,
                                        is_toggleable: None,
                                    },
                                ),
                            };

                            let mut block = Block::new(block_type);
                            match level {
                                pulldown_cmark::HeadingLevel::H1 => block.heading_1 = Some(heading_block),
                                pulldown_cmark::HeadingLevel::H2 => block.heading_2 = Some(heading_block),
                                _ => block.heading_3 = Some(heading_block),
                            }
                            blocks.push(block);
                        }
                        Tag::List(kind) => {
                            let list_events = collect_events_until(iter, &Tag::List(kind));
                            let mut child_blocks = parse_events(&mut list_events.into_iter());

                            if kind.is_some() {
                                // Convert to numbered
                                for block in &mut child_blocks {
                                    if let BlockType::BulletedListItem = block.block_type {
                                        if let Some(data) = block.bulleted_list_item.take() {
                                            block.block_type = BlockType::NumberedListItem;
                                            block.numbered_list_item = Some(data);
                                        }
                                    }
                                }
                            }
                            blocks.extend(child_blocks);
                        }
                        Tag::Item => {
                            let item_events = collect_events_until(iter, &Tag::Item);
                            let mut item_blocks = parse_events(&mut item_events.into_iter());

                            let mut rich_text = Vec::new();
                            let mut children = Vec::new();

                            if !item_blocks.is_empty() {
                                let first = item_blocks.remove(0);
                                if let Some(p) = first.paragraph {
                                    rich_text = p.rich_text;
                                    children = item_blocks;
                                } else {
                                    children = vec![first];
                                    children.extend(item_blocks);
                                }
                            }

                            let mut list_item = Block::new(BlockType::BulletedListItem);
                            list_item.bulleted_list_item = Some(ListItemBlock {
                                rich_text,
                                children: if children.is_empty() { None } else { Some(children) },
                                color: None,
                            });
                            blocks.push(list_item);
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        } else {
            inline_accumulator.push(event);
        }
    }

    if !inline_accumulator.is_empty() {
        let rich_text = parse_inline(inline_accumulator.into_iter());
        let mut block = Block::new(BlockType::Paragraph);
        block.paragraph = Some(ParagraphBlock {
            rich_text,
            children: None,
            color: None,
        });
        blocks.push(block);
    }

    blocks
}

fn is_block_tag(tag: &Tag) -> bool {
    matches!(
        tag,
        Tag::Paragraph
            | Tag::Heading(..)
            | Tag::List(_)
            | Tag::Item
            | Tag::BlockQuote
            | Tag::CodeBlock(_)
    )
}

fn collect_events_until<'a, I>(iter: &mut I, end_tag: &Tag<'a>) -> Vec<Event<'a>>
where
    I: Iterator<Item = Event<'a>>,
{
    let mut events = Vec::new();
    let mut depth = 1;
    let end_discriminant = std::mem::discriminant(end_tag);

    while let Some(event) = iter.next() {
        match &event {
            Event::Start(tag) if std::mem::discriminant(tag) == end_discriminant => {
                depth += 1;
            }
            Event::End(tag) if std::mem::discriminant(tag) == end_discriminant => {
                depth -= 1;
                if depth == 0 {
                    return events;
                }
            }
            _ => {}
        }
        events.push(event);
    }
    events
}

fn parse_inline<'a, I>(iter: I) -> Vec<RichText>
where
    I: Iterator<Item = Event<'a>>,
{
    let mut result = Vec::new();
    let mut current_annotations = Annotations::default();
    let mut current_link: Option<String> = None;

    for event in iter {
        match event {
            Event::Text(text) => {
                let mut rt = RichText::text(text.to_string(), current_link.clone());
                rt.annotations = Some(Annotations {
                    bold: current_annotations.bold,
                    italic: current_annotations.italic,
                    strikethrough: current_annotations.strikethrough,
                    underline: current_annotations.underline,
                    code: current_annotations.code,
                    color: current_annotations.color.clone(),
                });
                result.push(rt);
            }
            Event::Code(text) => {
                let mut rt = RichText::text(text.to_string(), current_link.clone());
                rt.annotations = Some(Annotations {
                    code: true,
                    bold: current_annotations.bold,
                    italic: current_annotations.italic,
                    strikethrough: current_annotations.strikethrough,
                    underline: current_annotations.underline,
                    color: current_annotations.color.clone(),
                });
                result.push(rt);
            }
            Event::Start(Tag::Strong) => current_annotations.bold = true,
            Event::End(Tag::Strong) => current_annotations.bold = false,
            Event::Start(Tag::Emphasis) => current_annotations.italic = true,
            Event::End(Tag::Emphasis) => current_annotations.italic = false,
            Event::Start(Tag::Strikethrough) => current_annotations.strikethrough = true,
            Event::End(Tag::Strikethrough) => current_annotations.strikethrough = false,
            Event::Start(Tag::Link(_, url, _)) => current_link = Some(url.to_string()),
            Event::End(Tag::Link(_, _, _)) => current_link = None,
            _ => {}
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use pulldown_cmark::{Event, Parser, Tag};

    #[test]
    fn test_parse_inline_plain_text() {
        let md = "hello world";
        // pulldown-cmark wraps in Paragraph. We want inner events.
        let parser = Parser::new(md);
        let events: Vec<Event> = parser.into_iter().filter(|e| match e {
            Event::Start(Tag::Paragraph) | Event::End(Tag::Paragraph) => false,
            _ => true
        }).collect();

        let rich_text = parse_inline(events.into_iter());
        assert_eq!(rich_text.len(), 1);
        assert_eq!(rich_text[0].plain_text.as_ref().unwrap(), "hello world");
    }

    #[test]
    fn test_parse_inline_bold() {
        let md = "**bold**";
        let parser = Parser::new(md);
        let events: Vec<Event> = parser.into_iter().filter(|e| match e {
            Event::Start(Tag::Paragraph) | Event::End(Tag::Paragraph) => false,
            _ => true
        }).collect();

        let rich_text = parse_inline(events.into_iter());
        assert_eq!(rich_text.len(), 1);
        assert_eq!(rich_text[0].plain_text.as_ref().unwrap(), "bold");
        assert!(rich_text[0].annotations.as_ref().unwrap().bold);
    }

    #[test]
    fn test_parse_paragraph() {
        let md = "hello world";
        let blocks = parse_markdown(md).unwrap();
        assert_eq!(blocks.len(), 1);
        match blocks[0].block_type {
            crate::notion::BlockType::Paragraph => {
                let p = blocks[0].paragraph.as_ref().unwrap();
                assert_eq!(p.rich_text.len(), 1);
                assert_eq!(p.rich_text[0].plain_text.as_ref().unwrap(), "hello world");
            }
            _ => panic!("Expected Paragraph"),
        }
    }

    #[test]
    fn test_parse_heading() {
        let md = "# Heading 1";
        let blocks = parse_markdown(md).unwrap();
        assert_eq!(blocks.len(), 1);
        match blocks[0].block_type {
            crate::notion::BlockType::Heading1 => {
                 let h = blocks[0].heading_1.as_ref().unwrap();
                 assert_eq!(h.rich_text[0].plain_text.as_ref().unwrap(), "Heading 1");
            }
            _ => panic!("Expected Heading1"),
        }
    }

    #[test]
    fn test_parse_list() {
        let md = "* Item 1\n* Item 2";
        let blocks = parse_markdown(md).unwrap();
        assert_eq!(blocks.len(), 2);
        assert!(matches!(blocks[0].block_type, BlockType::BulletedListItem));
        assert!(matches!(blocks[1].block_type, BlockType::BulletedListItem));
        assert_eq!(blocks[0].bulleted_list_item.as_ref().unwrap().rich_text[0].plain_text.as_ref().unwrap(), "Item 1");
    }

    #[test]
    fn test_parse_nested_list() {
        let md = "* Item 1\n  * Nested";
        let blocks = parse_markdown(md).unwrap();
        assert_eq!(blocks.len(), 1); // Top level: Item 1
        let item = blocks[0].bulleted_list_item.as_ref().unwrap();
        assert!(item.children.is_some());
        let children = item.children.as_ref().unwrap();
        assert_eq!(children.len(), 1);
        assert!(matches!(children[0].block_type, BlockType::BulletedListItem));
        assert_eq!(children[0].bulleted_list_item.as_ref().unwrap().rich_text[0].plain_text.as_ref().unwrap(), "Nested");
    }
}
