use lexer::parser::{Prompt, Response};

#[derive(Debug)]
pub struct PromptStartErr;

#[derive(Debug)]
pub enum PrompterErr {
    BadResponse,
    NoMoreQ,
}

#[derive(Clone)]
pub struct Prompter<'a> {
    prompts: Vec<Prompt<'a>>,
    next: Prompt<'a>,
    next_idx: usize,
}

const STARTING_LABEL: &str = "START";

impl<'a> Prompter<'a> {
    pub fn new(prompts: Vec<Prompt<'a>>) -> Result<Prompter, PromptStartErr> {
        let (start_idx, start) = prompts
            .clone()
            .into_iter()
            .enumerate()
            .find(|(_, item)| item.label == Some(STARTING_LABEL))
            .or_else(|| prompts.first().map(|x| (0, x.clone())))
            .ok_or(PromptStartErr)?;

        Ok(Self {
            prompts,
            next: start,
            next_idx: start_idx,
        })
    }

    pub fn next(&self) -> Prompt<'a> {
        self.next.clone()
    }

    pub fn answer(self, response: &Response) -> Result<Prompter<'a>, PrompterErr> {
        // When there is a label try to find the question with the given label
        // otherwise, move on to the next question

        if let Some(label) = response.label {
            let (next_idx, next_prompt) = self
                .prompts
                .clone()
                .into_iter()
                .enumerate()
                .find(|(_, item)| item.label == Some(label))
                .ok_or(PrompterErr::NoMoreQ)?;

            let result = Self {
                next: next_prompt,
                next_idx,
                ..self
            };

            return Ok(result);
        }

        let next_prompt = self
            .prompts
            .get(self.next_idx + 1)
            .ok_or(PrompterErr::NoMoreQ)?
            .clone();

        let result = Self {
            next: next_prompt,
            next_idx: self.next_idx + 1,
            ..self
        };

        return Ok(result);
    }
}

#[cfg(test)]
mod tests {
    use std::fs::read_to_string;

    use lexer::parse;

    use crate::Prompter;

    #[test]
    fn it_works_with_looping() {
        let data = read_to_string("./simple_prompt.txt").unwrap();
        let prompts = parse(&data).unwrap();
        let seed_prompt = Prompter::new(prompts).unwrap();

        assert_eq!(seed_prompt.next.label, Some("START"));

        let next_prompt = seed_prompt
            .clone()
            .answer(&seed_prompt.next.responses[1])
            .unwrap();
        assert_eq!(next_prompt.next.label, Some("ANS_NO"));

        let next_prompt = next_prompt
            .clone()
            .answer(&next_prompt.next.responses[0])
            .unwrap();
        assert_eq!(next_prompt.next.label, Some("START"));
    }

    #[test]
    fn it_works_until_finish() {
        let data = read_to_string("./simple_prompt.txt").unwrap();
        let prompts = parse(&data).unwrap();
        let seed_prompt = Prompter::new(prompts).unwrap();

        let next_prompt = seed_prompt
            .clone()
            .answer(&seed_prompt.next.responses[0].clone())
            .unwrap();

        assert_eq!(next_prompt.next.label, Some("YES"));
        assert_eq!(next_prompt.next.text, "Nice! Glad to meet you human!");
        assert_eq!(next_prompt.next.responses.len(), 0);
    }

    #[test]
    fn works_without_labels() {
        let data = read_to_string("./labelless_prompt.txt").unwrap();
        let prompts = parse(&data).unwrap();
        let seed_prompt = Prompter::new(prompts).unwrap();
        assert_eq!(seed_prompt.next.text, "Are you a human?");

        let next_prompt = seed_prompt
            .clone()
            .answer(&seed_prompt.next.responses[0])
            .unwrap();
        assert_eq!(next_prompt.next.label, None);
        assert_eq!(next_prompt.next.text, "Nice! Glad to meet you human!");
    }

    // Write test for the error cases
}
