use std::sync::LazyLock;

use color_eyre::eyre::Result;
use oxc_allocator::Allocator;
use oxc_ast::ast::Program;
use oxc_ast_visit::utf8_to_utf16::Utf8ToUtf16;
use oxc_parser::{Parser, ParserReturn};
use oxc_span::SourceType;
use tracing::instrument;

static ALLOCATOR: LazyLock<Allocator> = LazyLock::new(Allocator::default);

#[instrument(skip(js))]
fn parse_js(js: &str) -> ParserReturn<'_> {
    let source_type = SourceType::mjs();

    Parser::new(&ALLOCATOR, js, source_type).parse()
}

#[instrument(skip(source, program))]
fn to_utf16(source: &str, program: &mut Program<'_>) {
    Utf8ToUtf16::new(source).convert_program(program)
}

#[instrument(skip(js))]
pub async fn get_js_estree<'a>(js: &'a str) -> Result<ParserReturn<'a>> {
    let mut parsed = parse_js(js);

    to_utf16(js, &mut parsed.program);

    Ok(parsed)
}
