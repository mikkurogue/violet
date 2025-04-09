use tree_sitter::{Parser, Query, QueryCursor};
use tree_sitter_rust::{HIGHLIGHT_QUERY, language};

use crate::editor::theme::{StyleInfo, Theme};

pub struct Highlighter {
    parser: Parser,
    query: Query,
    theme: Theme,
}

impl Highlighter {
    pub fn new(theme: Theme) -> anyhow::Result<Highlighter> {
        let mut parser = Parser::new();

        let lang = language();
        parser.set_language(lang)?;

        let query = Query::new(lang, HIGHLIGHT_QUERY)?;

        Ok(Highlighter {
            parser,
            query,
            theme,
        })
    }

    pub fn highlight(&mut self, code: &str) -> anyhow::Result<Vec<StyleInfo>> {
        let tree = self.parser.parse(code, None).expect("parse works");

        let mut colors = Vec::new();
        let mut cursor = QueryCursor::new();
        let matches = cursor.matches(&self.query, tree.root_node(), code.as_bytes());

        for mat in matches {
            for cap in mat.captures {
                let node = cap.node;
                let start = node.start_byte();
                let end = node.end_byte();
                let scope = self.query.capture_names()[cap.index as usize].as_str();
                let style = self.theme.get_style(scope);

                if let Some(style) = style {
                    colors.push(StyleInfo { start, end, style });
                }
            }
        }

        Ok(colors)
    }
}
