use crate::chunker::{Chunk, ChunkVariant};

pub(crate) enum ParserError {}

#[derive(PartialEq, Eq, Debug)]
pub struct Response<'a> {
    pub text: &'a str,
    pub label: Option<&'a str>,
}

#[derive(PartialEq, Eq, Debug)]
pub struct Prompt<'a> {
    pub text: &'a str,
    pub label: Option<&'a str>,
    pub responses: Vec<Response<'a>>,
}

// TODO: Really weird design here, clean it up
type ResponseParsingResult<'a> = (Option<usize>, Vec<Response<'a>>);

// Scans given slice greedily and returns the index after the scan of the slice
// and a vector of possible responses.
fn parse_response_chunks_greedily<'a>(chunks: &[Chunk<'a>]) -> ResponseParsingResult<'a> {
    let mut scan_position: usize = 0;

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

    if scan_position == 0 {
        return (None, response);
    } else {
        return (Some(scan_position - 1), response);
    }
}

pub(crate) struct Parser {
    scan_position: usize,
}

impl Parser {
    pub(crate) fn new() -> Self {
        Parser { scan_position: 0 }
    }

    pub(crate) fn parse_chunks<'a>(&mut self, chunks: Vec<Chunk<'a>>) -> Vec<Prompt<'a>> {
        let mut prompts = vec![];

        while let Some(Chunk {
            variant: ChunkVariant::Prompt,
            text,
            label,
        }) = chunks.get(self.scan_position)
        {
            let (relative_scan_position, responses) =
                parse_response_chunks_greedily(&chunks[self.scan_position + 1..]);
            prompts.push(Prompt {
                text,
                label: *label,
                responses,
            });

            match relative_scan_position {
                Some(relative_scan_pos) => self.scan_position += relative_scan_pos + 2,
                None => self.scan_position += 1,
            }
        }

        prompts
    }
}

#[cfg(test)]
mod test {
    use crate::{
        chunker::{Chunk, ChunkVariant},
        parser::{parse_response_chunks_greedily, Prompt},
    };

    use super::{Parser, Response};

    // Parsing chunks fully
    #[test]
    fn parse_chunks_with_prompts_and_responses() {
        let input = vec![
            Chunk {
                variant: ChunkVariant::Prompt,
                text: "Are you human?",
                label: Some("NONHUMAN"),
            },
            Chunk {
                variant: ChunkVariant::Response,
                text: "Yes",
                label: Some("HUMAN"),
            },
            Chunk {
                variant: ChunkVariant::Response,
                text: "No",
                label: Some("NONHUMAN"),
            },
            Chunk {
                variant: ChunkVariant::Prompt,
                text: "Nice to meet you",
                label: Some("HUMAN"),
            },
        ];

        let expected_result = vec![
            Prompt {
                text: "Are you human?",
                label: Some("NONHUMAN"),
                responses: vec![
                    Response {
                        text: "Yes",
                        label: Some("HUMAN"),
                    },
                    Response {
                        text: "No",
                        label: Some("NONHUMAN"),
                    },
                ],
            },
            Prompt {
                text: "Nice to meet you",
                label: Some("HUMAN"),
                responses: vec![],
            },
        ];

        let mut parser = Parser::new();
        let parse_results = parser.parse_chunks(input);

        assert_eq!(parse_results, expected_result);
    }

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

        assert_eq!(result, (Some(1), expected_result));
    }

    #[test]
    fn parse_no_respinse() {
        let input = vec![Chunk {
            variant: ChunkVariant::Prompt,
            label: None,
            text: "foobar",
        }];

        let results = parse_response_chunks_greedily(&input);

        let expected_relative_index: Option<usize> = None;
        let resultant_vec: Vec<Response> = vec![];

        assert_eq!(results, (expected_relative_index, resultant_vec));
    }
}
