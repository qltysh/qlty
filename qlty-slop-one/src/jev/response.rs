//! Validation of a raw provider body into one value per question.

use serde_json::{Map, Value};

use super::{JevProvider, JEV_MODEL};
use crate::error::{Error, Result};
use crate::features::{questions, Question, QuestionKind};
use crate::fsum::fsum;

/// One excerpt's answers, in `features::questions()` order, each in `[0, 1]`.
#[derive(Clone, Debug, PartialEq)]
pub struct Answers {
    values: Vec<f64>,
}

impl Answers {
    /// Answers in question order. Fails unless there is exactly one value
    /// per question.
    pub fn try_new(values: Vec<f64>) -> Result<Self> {
        if values.len() != questions().len() {
            return Err(Error::Jev(
                "Jev returned missing or extra answers.".to_owned(),
            ));
        }
        Ok(Self { values })
    }

    pub fn values(&self) -> &[f64] {
        &self.values
    }
}

/// The values slopdetect's `validate` derives from a raw response: `noul`
/// probabilities as they are, rubric distributions as
/// `sum(k * p_k) / ((levels - 1) * sum(p_k))`.
pub fn validate(response: &Value, provider: JevProvider) -> Result<Answers> {
    let shape = Shape::for_provider(provider);
    let object = response.as_object();
    if shape.requires_model
        && object
            .and_then(|object| object.get("model"))
            .and_then(Value::as_str)
            != Some(JEV_MODEL)
    {
        return Err(jev("Jev returned an unexpected model version."));
    }
    let answers = object
        .and_then(|object| object.get("answers"))
        .and_then(Value::as_object);
    let Some(answers) = answers.filter(|answers| has_exactly_the_question_ids(answers)) else {
        return Err(jev("Jev returned missing or extra answers."));
    };
    let values = questions()
        .iter()
        .map(|question| answer_value(&answers[&question.id], question, shape))
        .collect::<Result<Vec<f64>>>()?;
    Answers::try_new(values)
}

fn has_exactly_the_question_ids(answers: &Map<String, Value>) -> bool {
    answers.len() == questions().len()
        && questions()
            .iter()
            .all(|question| answers.contains_key(&question.id))
}

/// The provider's spelling of the boolean answer.
#[derive(Clone, Copy, Debug)]
struct Shape {
    requires_model: bool,
    boolean_type: &'static str,
    boolean_key: &'static str,
}

impl Shape {
    fn for_provider(provider: JevProvider) -> Self {
        match provider {
            JevProvider::TypeSafe => Self {
                requires_model: true,
                boolean_type: "noul",
                boolean_key: "noul",
            },
            JevProvider::Vercel => Self {
                requires_model: false,
                boolean_type: "boolean",
                boolean_key: "probability",
            },
        }
    }

    fn type_word(self, kind: QuestionKind) -> &'static str {
        match kind {
            QuestionKind::Noul => self.boolean_type,
            QuestionKind::Score => "score",
        }
    }
}

fn answer_value(answer: &Value, question: &Question, shape: Shape) -> Result<f64> {
    let Some(answer) = answer.as_object() else {
        return Err(jev("Jev returned an unexpected answer type."));
    };
    if answer.get("type").and_then(Value::as_str) != Some(shape.type_word(question.kind)) {
        return Err(jev("Jev returned an unexpected answer type."));
    }
    match question.kind {
        QuestionKind::Noul => answer
            .get(shape.boolean_key)
            .and_then(unit_interval)
            .ok_or_else(|| jev("Jev returned an invalid probability.")),
        QuestionKind::Score => rubric_value(answer, question.criteria.len()),
    }
}

fn rubric_value(answer: &Map<String, Value>, count: usize) -> Result<f64> {
    let distribution = match answer.get("probabilities") {
        None => Map::new(),
        Some(Value::Object(distribution)) => distribution.clone(),
        Some(_) => return Err(jev("Jev returned an invalid rubric distribution.")),
    };
    let has_every_level = distribution.len() == count
        && (0..count).all(|level| distribution.contains_key(&level.to_string()));
    if !has_every_level {
        return Err(jev("Jev returned an invalid rubric distribution."));
    }
    let mut weighted = Vec::with_capacity(count);
    let mut probabilities = Vec::with_capacity(count);
    for (level, probability) in &distribution {
        let Some(probability) = unit_interval(probability) else {
            return Err(jev("Jev returned an invalid rubric probability."));
        };
        let level: f64 = level
            .parse::<u64>()
            .map_err(|_| jev("Jev returned an invalid rubric distribution."))?
            as f64;
        probabilities.push(probability);
        weighted.push(level * probability);
    }
    let total = fsum(probabilities);
    if (total - 1.0).abs() > 0.02 {
        return Err(jev("Jev rubric probabilities do not sum to one."));
    }
    Ok(fsum(weighted) / ((count - 1) as f64 * total))
}

/// A finite JSON number in `[0, 1]`. Booleans are not numbers here, as in
/// slopdetect's `finite`.
fn unit_interval(value: &Value) -> Option<f64> {
    value
        .as_number()
        .and_then(serde_json::Number::as_f64)
        .filter(|value| value.is_finite() && (0.0..=1.0).contains(value))
}

fn jev(message: &str) -> Error {
    Error::Jev(message.to_owned())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn typesafe_response() -> Value {
        json!({
            "model": "jev-1.13.0",
            "answers": {
                "direct_unmaintainable": {"type": "noul", "noul": 0.66},
                "boilerplate": {"type": "noul", "noul": 0.45},
                "deep_reasoning": {"type": "noul", "noul": 1},
                "control_flow": {"type": "score", "score": 1.1,
                    "probabilities": {"0": 0.01, "1": 0.88, "2": 0.11}},
                "duplicated_logic": {"type": "score", "score": 1.82,
                    "probabilities": {"0": 0.01, "1": 0.17, "2": 0.81, "3": 0.01}},
                "mirrored_state": {"type": "score", "score": 0.88,
                    "probabilities": {"0": 0.41, "1": 0.36, "2": 0.19, "3": 0.04}},
                "hand_enumerated_data": {"type": "score", "score": 0.86,
                    "probabilities": {"0": 0.35, "1": 0.46, "2": 0.18, "3": 0.01}}
            },
            "usage": {"input_tokens": 6364, "output_tokens": 124}
        })
    }

    fn vercel_response() -> Value {
        json!({
            "answers": {
                "direct_unmaintainable": {"type": "boolean", "probability": 0.66},
                "boilerplate": {"type": "boolean", "probability": 0.45},
                "deep_reasoning": {"type": "boolean", "probability": 1},
                "control_flow": {"type": "score", "score": 1.1,
                    "probabilities": {"0": 0.01, "1": 0.88, "2": 0.11}},
                "duplicated_logic": {"type": "score", "score": 1.82,
                    "probabilities": {"0": 0.01, "1": 0.17, "2": 0.81, "3": 0.01}},
                "mirrored_state": {"type": "score", "score": 0.88,
                    "probabilities": {"0": 0.41, "1": 0.36, "2": 0.19, "3": 0.04}},
                "hand_enumerated_data": {"type": "score", "score": 0.86,
                    "probabilities": {"0": 0.35, "1": 0.46, "2": 0.18, "3": 0.01}}
            },
            "usage": {"inputTokens": 6364, "outputTokens": 124}
        })
    }

    fn message(outcome: Result<Answers>) -> String {
        outcome.unwrap_err().to_string()
    }

    #[test]
    fn accepts_a_typesafe_response() {
        let answers = validate(&typesafe_response(), JevProvider::TypeSafe).unwrap();
        assert_eq!(answers.values()[0], 0.66);
        assert_eq!(answers.values()[2], 1.0);
        assert_eq!(
            answers.values()[3],
            fsum([0.0, 0.88, 0.22]) / (2.0 * fsum([0.01, 0.88, 0.11]))
        );
    }

    #[test]
    fn accepts_a_vercel_response_for_the_vercel_provider() {
        let typesafe = validate(&typesafe_response(), JevProvider::TypeSafe).unwrap();
        let vercel = validate(&vercel_response(), JevProvider::Vercel).unwrap();
        assert_eq!(typesafe, vercel);
    }

    #[test]
    fn typesafe_provider_rejects_a_response_without_the_model() {
        assert_eq!(
            message(validate(&vercel_response(), JevProvider::TypeSafe)),
            "Jev returned an unexpected model version."
        );
    }

    #[test]
    fn rejects_another_model_version() {
        let mut response = typesafe_response();
        response["model"] = json!("other");
        assert_eq!(
            message(validate(&response, JevProvider::TypeSafe)),
            "Jev returned an unexpected model version."
        );
    }

    #[test]
    fn rejects_a_non_object_response() {
        assert_eq!(
            message(validate(&json!([]), JevProvider::TypeSafe)),
            "Jev returned an unexpected model version."
        );
    }

    #[test]
    fn rejects_missing_answers() {
        let mut response = typesafe_response();
        response["answers"]
            .as_object_mut()
            .unwrap()
            .remove("boilerplate");
        assert_eq!(
            message(validate(&response, JevProvider::TypeSafe)),
            "Jev returned missing or extra answers."
        );
    }

    #[test]
    fn rejects_extra_answers() {
        let mut response = typesafe_response();
        response["answers"]["extra"] = json!({"type": "noul", "noul": 0.5});
        assert_eq!(
            message(validate(&response, JevProvider::TypeSafe)),
            "Jev returned missing or extra answers."
        );
    }

    #[test]
    fn rejects_a_mismatched_answer_type() {
        let mut response = typesafe_response();
        response["answers"]["boilerplate"]["type"] = json!("score");
        assert_eq!(
            message(validate(&response, JevProvider::TypeSafe)),
            "Jev returned an unexpected answer type."
        );
    }

    #[test]
    fn rejects_a_boolean_probability() {
        let mut response = typesafe_response();
        response["answers"]["boilerplate"]["noul"] = json!(true);
        assert_eq!(
            message(validate(&response, JevProvider::TypeSafe)),
            "Jev returned an invalid probability."
        );
    }

    #[test]
    fn rejects_a_probability_above_one() {
        let mut response = typesafe_response();
        response["answers"]["boilerplate"]["noul"] = json!(1.1);
        assert_eq!(
            message(validate(&response, JevProvider::TypeSafe)),
            "Jev returned an invalid probability."
        );
    }

    #[test]
    fn rejects_a_negative_probability() {
        let mut response = typesafe_response();
        response["answers"]["boilerplate"]["noul"] = json!(-0.1);
        assert!(validate(&response, JevProvider::TypeSafe).is_err());
    }

    #[test]
    fn rejects_a_missing_probability() {
        let mut response = typesafe_response();
        response["answers"]["boilerplate"]
            .as_object_mut()
            .unwrap()
            .remove("noul");
        assert!(validate(&response, JevProvider::TypeSafe).is_err());
    }

    #[test]
    fn rejects_a_rubric_with_a_missing_level() {
        let mut response = typesafe_response();
        response["answers"]["control_flow"]["probabilities"] = json!({"0": 0.5, "2": 0.5});
        assert_eq!(
            message(validate(&response, JevProvider::TypeSafe)),
            "Jev returned an invalid rubric distribution."
        );
    }

    #[test]
    fn rejects_a_rubric_without_probabilities() {
        let mut response = typesafe_response();
        response["answers"]["control_flow"]
            .as_object_mut()
            .unwrap()
            .remove("probabilities");
        assert_eq!(
            message(validate(&response, JevProvider::TypeSafe)),
            "Jev returned an invalid rubric distribution."
        );
    }

    #[test]
    fn rejects_a_rubric_probability_outside_the_unit_interval() {
        let mut response = typesafe_response();
        response["answers"]["control_flow"]["probabilities"]["1"] = json!(1.5);
        assert_eq!(
            message(validate(&response, JevProvider::TypeSafe)),
            "Jev returned an invalid rubric probability."
        );
    }

    #[test]
    fn rejects_a_rubric_that_does_not_sum_to_one() {
        let mut response = typesafe_response();
        response["answers"]["control_flow"]["probabilities"] =
            json!({"0": 0.5, "1": 0.4, "2": 0.05});
        assert_eq!(
            message(validate(&response, JevProvider::TypeSafe)),
            "Jev rubric probabilities do not sum to one."
        );
    }

    #[test]
    fn tolerates_a_rubric_sum_within_two_percent() {
        let mut response = typesafe_response();
        response["answers"]["control_flow"]["probabilities"] =
            json!({"0": 0.5, "1": 0.4, "2": 0.09});
        let answers = validate(&response, JevProvider::TypeSafe).unwrap();
        assert_eq!(
            answers.values()[3],
            fsum([0.0, 0.4, 0.18]) / (2.0 * fsum([0.5, 0.4, 0.09]))
        );
    }

    #[test]
    fn answers_require_one_value_per_question() {
        assert!(Answers::try_new(vec![0.5; 6]).is_err());
        assert!(Answers::try_new(vec![0.5; 7]).is_ok());
    }
}
