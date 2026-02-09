use dafhne_core::{ExpectedAnswer, TestQuestion, TestSuite};

/// Parse test questions from markdown format.
pub fn parse_test_questions(content: &str) -> TestSuite {
    let mut questions: Vec<TestQuestion> = Vec::new();
    let mut current_category = String::new();

    let mut current_id: Option<String> = None;
    let mut current_question = String::new();
    let mut current_answer: Option<ExpectedAnswer> = None;
    let mut current_chain = String::new();

    for line in content.lines() {
        let trimmed = line.trim();

        // Skip empty lines, blockquotes, title, separators, scoring table
        if trimmed.is_empty() || trimmed.starts_with('>') || trimmed.starts_with("# dict") {
            continue;
        }

        if trimmed == "---" {
            // Finalize current question if any
            if let (Some(id), Some(answer)) = (current_id.take(), current_answer.take()) {
                questions.push(TestQuestion {
                    id,
                    question: current_question.clone(),
                    expected: answer,
                    chain: current_chain.clone(),
                    category: current_category.clone(),
                });
                current_question.clear();
                current_chain.clear();
            }
            continue;
        }

        // Section header
        if trimmed.starts_with("## ") {
            // Finalize current question if any
            if let (Some(id), Some(answer)) = (current_id.take(), current_answer.take()) {
                questions.push(TestQuestion {
                    id,
                    question: current_question.clone(),
                    expected: answer,
                    chain: current_chain.clone(),
                    category: current_category.clone(),
                });
                current_question.clear();
                current_chain.clear();
            }
            current_category = trimmed[3..].trim().to_string();
            // Strip the parenthetical: "DIRECT LOOKUP (answer is explicitly stated)" -> "DIRECT LOOKUP"
            if let Some(paren_pos) = current_category.find('(') {
                current_category = current_category[..paren_pos].trim().to_string();
            }
            continue;
        }

        // Question line: **Q01**: Is a dog an animal?
        if trimmed.starts_with("**Q") {
            // Finalize previous question
            if let (Some(id), Some(answer)) = (current_id.take(), current_answer.take()) {
                questions.push(TestQuestion {
                    id,
                    question: current_question.clone(),
                    expected: answer,
                    chain: current_chain.clone(),
                    category: current_category.clone(),
                });
                current_question.clear();
                current_chain.clear();
            }

            if let Some(close) = trimmed.find("**:") {
                let id = trimmed[2..close].to_string();
                let question = trimmed[close + 3..].trim().to_string();
                current_id = Some(id);
                current_question = question;
            }
            continue;
        }

        // Answer line: **A**: Yes
        if trimmed.starts_with("**A**:") {
            let answer_text = trimmed["**A**:".len()..].trim();
            current_answer = Some(parse_expected_answer(answer_text));
            continue;
        }

        // Chain line: **Chain**: reasoning...
        if trimmed.starts_with("**Chain**:") {
            current_chain = trimmed["**Chain**:".len()..].trim().to_string();
            continue;
        }
    }

    // Finalize last question
    if let (Some(id), Some(answer)) = (current_id.take(), current_answer.take()) {
        questions.push(TestQuestion {
            id,
            question: current_question,
            expected: answer,
            chain: current_chain,
            category: current_category,
        });
    }

    TestSuite { questions }
}

fn parse_expected_answer(text: &str) -> ExpectedAnswer {
    let lower = text.to_lowercase();
    match lower.as_str() {
        "yes" => ExpectedAnswer::Yes,
        "no" => ExpectedAnswer::No,
        "i don't know" | "i don\u{2019}t know" => ExpectedAnswer::IDontKnow,
        _ => ExpectedAnswer::Word(lower),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_expected_answer() {
        assert_eq!(parse_expected_answer("Yes"), ExpectedAnswer::Yes);
        assert_eq!(parse_expected_answer("No"), ExpectedAnswer::No);
        assert_eq!(parse_expected_answer("I don't know"), ExpectedAnswer::IDontKnow);
        assert_eq!(
            parse_expected_answer("an animal"),
            ExpectedAnswer::Word("an animal".to_string())
        );
    }

    #[test]
    fn test_parse_test_questions() {
        let content = std::fs::read_to_string("../../dictionaries/dict5_test.md").unwrap();
        let suite = parse_test_questions(&content);

        assert_eq!(suite.questions.len(), 20, "Expected 20 questions, got {}", suite.questions.len());

        // Q01: Is a dog an animal? -> Yes
        assert_eq!(suite.questions[0].id, "Q01");
        assert_eq!(suite.questions[0].expected, ExpectedAnswer::Yes);

        // Q11: Is a dog a cat? -> No
        let q11 = suite.questions.iter().find(|q| q.id == "Q11").unwrap();
        assert_eq!(q11.expected, ExpectedAnswer::No);

        // Q15: What color is a dog? -> I don't know
        let q15 = suite.questions.iter().find(|q| q.id == "Q15").unwrap();
        assert_eq!(q15.expected, ExpectedAnswer::IDontKnow);

        // Q19: What is a dog? -> an animal
        let q19 = suite.questions.iter().find(|q| q.id == "Q19").unwrap();
        assert_eq!(q19.expected, ExpectedAnswer::Word("an animal".to_string()));
    }

    #[test]
    fn test_question_categories() {
        let content = std::fs::read_to_string("../../dictionaries/dict5_test.md").unwrap();
        let suite = parse_test_questions(&content);

        // Direct lookup: Q01-Q05
        for id in &["Q01", "Q02", "Q03", "Q04", "Q05"] {
            let q = suite.questions.iter().find(|q| q.id == *id).unwrap();
            assert!(q.category.contains("DIRECT"), "Q {} should be DIRECT, got {}", id, q.category);
        }

        // Unknown: Q15-Q18
        for id in &["Q15", "Q16", "Q17", "Q18"] {
            let q = suite.questions.iter().find(|q| q.id == *id).unwrap();
            assert!(q.category.contains("UNKNOWN"), "Q {} should be UNKNOWN, got {}", id, q.category);
        }
    }

    #[test]
    fn test_parse_dict12_test_questions() {
        let content = std::fs::read_to_string("../../dictionaries/dict12_test.md").unwrap();
        let suite = parse_test_questions(&content);

        assert_eq!(suite.questions.len(), 20, "Expected 20 questions, got {}", suite.questions.len());

        // Q01: Is a dog a mammal? -> Yes
        assert_eq!(suite.questions[0].id, "Q01");
        assert_eq!(suite.questions[0].expected, ExpectedAnswer::Yes);

        // Q08: Is a wolf an animal? -> Yes
        let q08 = suite.questions.iter().find(|q| q.id == "Q08").unwrap();
        assert_eq!(q08.expected, ExpectedAnswer::Yes);

        // Q11: Is a plant an animal? -> No
        let q11 = suite.questions.iter().find(|q| q.id == "Q11").unwrap();
        assert_eq!(q11.expected, ExpectedAnswer::No);

        // Q15: What is the name of the sun? -> I don't know
        let q15 = suite.questions.iter().find(|q| q.id == "Q15").unwrap();
        assert_eq!(q15.expected, ExpectedAnswer::IDontKnow);

        // Q19: What is a cat? -> a mammal
        let q19 = suite.questions.iter().find(|q| q.id == "Q19").unwrap();
        assert_eq!(q19.expected, ExpectedAnswer::Word("a mammal".to_string()));

        // Q20: What is a wolf? -> an animal
        let q20 = suite.questions.iter().find(|q| q.id == "Q20").unwrap();
        assert_eq!(q20.expected, ExpectedAnswer::Word("an animal".to_string()));
    }

    #[test]
    fn test_dict12_question_categories() {
        let content = std::fs::read_to_string("../../dictionaries/dict12_test.md").unwrap();
        let suite = parse_test_questions(&content);

        // Direct lookup: Q01-Q05
        for id in &["Q01", "Q02", "Q03", "Q04", "Q05"] {
            let q = suite.questions.iter().find(|q| q.id == *id).unwrap();
            assert!(q.category.contains("DIRECT"), "Q {} should be DIRECT, got {}", id, q.category);
        }

        // Transitive: Q06-Q10
        for id in &["Q06", "Q07", "Q08", "Q09", "Q10"] {
            let q = suite.questions.iter().find(|q| q.id == *id).unwrap();
            assert!(q.category.contains("TRANSITIVE"), "Q {} should be TRANSITIVE, got {}", id, q.category);
        }

        // Negation: Q11-Q14
        for id in &["Q11", "Q12", "Q13", "Q14"] {
            let q = suite.questions.iter().find(|q| q.id == *id).unwrap();
            assert!(q.category.contains("NEGATION"), "Q {} should be NEGATION, got {}", id, q.category);
        }

        // Unknown: Q15-Q18
        for id in &["Q15", "Q16", "Q17", "Q18"] {
            let q = suite.questions.iter().find(|q| q.id == *id).unwrap();
            assert!(q.category.contains("UNKNOWN"), "Q {} should be UNKNOWN, got {}", id, q.category);
        }

        // Property query: Q19-Q20
        for id in &["Q19", "Q20"] {
            let q = suite.questions.iter().find(|q| q.id == *id).unwrap();
            assert!(q.category.contains("PROPERTY"), "Q {} should be PROPERTY, got {}", id, q.category);
        }
    }
}
