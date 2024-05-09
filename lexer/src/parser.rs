use std::string::ParseError;

use crate::chunker::{Chunk, ChunkVariant, Chunker, ChunkingError};

pub(crate) enum ParserError {}

#[derive(PartialEq, Eq, Debug)]
pub(crate) struct Response<'a> {
    text: &'a str,
    label: Option<&'a str>,
}

#[derive(PartialEq, Eq, Debug)]
pub(crate) struct Prompt<'a> {
    text: &'a str,
    label: Option<&'a str>,
    responses: Vec<Response<'a>>,
}

type ResponseParsingResult<'a> = (usize, Vec<Response<'a>>);

pub(crate) fn parse_response_chunks_greedily<'a>(
    chunks: &[Chunk<'a>],
) -> ResponseParsingResult<'a> {
    let mut scan_position = 0;

    while let Some(Chunk {
        variant: ChunkVariant::Response,
        ..
    }) = chunks.get(scan_position)
    {
        scan_position += 1;
    }

    let response: Vec<Response> = chunks[0..scan_position]
        .iter()
        .map(|c| Response {
            text: c.text,
            label: c.label,
        })
        .collect();

    (scan_position - 1, response)
}

pub(crate) struct Parser {
    scan_position: usize,
}

impl Parser {
    pub(crate) fn new() -> Self {
        Parser { scan_position: 0 }
    }

    pub(crate) fn parse_chunks(&mut self, chunks: Vec<Chunk>) -> Result<Vec<Prompt>, ParseError> {
        unimplemented!()
    }
}

#[cfg(test)]
mod test {
    use crate::{
        chunker::{Chunk, ChunkVariant},
        parser::parse_response_chunks_greedily,
    };

    use super::{Parser, Response};

    // Parsing some chunks into responses, until the next prompt.
    #[test]
    fn parse_a_few_responses() {
        let input = vec![
            Chunk {
                variant: ChunkVariant::Response,
                label: Some("NICE"),
                text: "Hello world?",
            },
            Chunk {
                variant: ChunkVariant::Response,
                label: Some("NICE2"),
                text: "Hello me!",
            },
            Chunk {
                variant: ChunkVariant::Prompt,
                label: None,
                text: "Who are you?",
            },
        ];

        let result = parse_response_chunks_greedily(&input);

        let expected_result = vec![
            Response {
                text: "Hello world?",
                label: Some("NICE"),
            },
            Response {
                text: "Hello me!",
                label: Some("NICE2"),
            },
        ];

        assert_eq!(result, (1, expected_result));
    }
}
