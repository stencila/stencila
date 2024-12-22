use codec_markdown_trait::MarkdownCodec;
use format::Format;
use node_type::NodeProperty;

use crate::ModelParameters;

impl MarkdownCodec for ModelParameters {
    fn to_markdown(&self, context: &mut codec_markdown_trait::MarkdownEncodeContext) {
        if matches!(context.format, Format::Myst) {
            if let Some(model_ids) = &self.model_ids {
                if !model_ids.is_empty() {
                    context.myst_directive_option(
                        NodeProperty::ModelIds,
                        Some("models"),
                        &model_ids.join(", "),
                    );
                }
            }

            if let Some(value) = &self.replicates {
                if value != 1 {
                    context.myst_directive_option(
                        NodeProperty::Replicates,
                        Some("reps"),
                        &replicates.to_string(),
                    );
                }
            }

            if let Some(value) = self.quality_weight {
                if value != 0 {
                    context.myst_directive_option(
                        NodeProperty::QualityWeight,
                        Some("quality"),
                        &value.to_string(),
                    );
                }
            }

            if let Some(value) = self.cost_weight {
                if value != 0 {
                    context.myst_directive_option(
                        NodeProperty::CostWeight,
                        Some("cost"),
                        &value.to_string(),
                    );
                }
            }

            if let Some(value) = self.speed_weight {
                if value != 0 {
                    context.myst_directive_option(
                        NodeProperty::SpeedWeight,
                        Some("speed"),
                        &value.to_string(),
                    );
                }
            }

            if let Some(value) = self.minimum_score {
                if value != 100 {
                    context.myst_directive_option(
                        NodeProperty::MinimumScore,
                        Some("min-score"),
                        &value.to_string(),
                    );
                }
            }

            if let Some(value) = &self.temperature {
                context.myst_directive_option(
                    NodeProperty::Temperature,
                    Some("temp"),
                    &value.to_string(),
                );
            }

            if let Some(value) = &self.random_seed {
                context.myst_directive_option(
                    NodeProperty::RandomSeed,
                    Some("seed"),
                    &value.to_string(),
                );
            }
        } else {
            if let Some(model_ids) = &self.model_ids {
                if !model_ids.is_empty() {
                    context
                        .push_str("[")
                        .push_prop_str(NodeProperty::ModelIds, &model_ids.join(", "))
                        .push_str("] ");
                }
            }

            if let Some(value) = self.replicates {
                if value != 1 {
                    context
                        .push_str("x")
                        .push_prop_str(NodeProperty::Replicates, &value.to_string())
                        .push_str(" ");
                }
            }

            if let Some(value) = self.quality_weight {
                if value != 0 {
                    context
                        .push_str("q")
                        .push_prop_str(NodeProperty::QualityWeight, &value.to_string())
                        .push_str(" ");
                }
            }

            if let Some(value) = self.cost_weight {
                if value != 0 {
                    context
                        .push_str("c")
                        .push_prop_str(NodeProperty::CostWeight, &value.to_string())
                        .push_str(" ");
                }
            }

            if let Some(value) = self.speed_weight {
                if value != 0 {
                    context
                        .push_str("s")
                        .push_prop_str(NodeProperty::SpeedWeight, &value.to_string())
                        .push_str(" ");
                }
            }

            if let Some(value) = self.minimum_score {
                if value != 100 {
                    context
                        .push_str("m")
                        .push_prop_str(NodeProperty::MinimumScore, &value.to_string())
                        .push_str(" ");
                }
            }

            if let Some(value) = &self.temperature {
                context
                    .push_str("t")
                    .push_prop_str(NodeProperty::Temperature, &value.to_string())
                    .push_str(" ");
            }

            if let Some(value) = &self.random_seed {
                context
                    .push_str("r")
                    .push_prop_str(NodeProperty::RandomSeed, &value.to_string())
                    .push_str(" ");
            }
        }
    }
}
