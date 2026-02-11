use stencila_codec_markdown::to_markdown;
use stencila_schema::Skill;

use crate::SkillInstance;

/// Escape a string for use in XML text content
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Escape a string for use in an XML attribute value (double-quoted)
fn escape_xml_attr(s: &str) -> String {
    escape_xml(s).replace('"', "&quot;")
}

/// Serialize a skill to XML for context injection
///
/// Follows the format from <https://agentskills.io/integrate-skills#injecting-into-context>.
pub fn to_xml(skill: &Skill) -> String {
    let name = escape_xml_attr(&skill.name);
    let description = escape_xml(&skill.description);
    let instructions = escape_xml(&to_markdown(&skill.content));

    let mut xml = format!(
        "<skill name=\"{name}\">\n  <description>{description}</description>\n  <instructions>\n{instructions}  </instructions>\n"
    );

    if let Some(compat) = &skill.compatibility {
        xml.push_str(&format!(
            "  <compatibility>{}</compatibility>\n",
            escape_xml(compat)
        ));
    }

    if let Some(tools) = &skill.allowed_tools {
        let tools_str = tools
            .iter()
            .map(|t| escape_xml(t))
            .collect::<Vec<_>>()
            .join(" ");
        xml.push_str(&format!("  <allowed-tools>{tools_str}</allowed-tools>\n"));
    }

    xml.push_str("</skill>");
    xml
}

/// Serialize skill metadata to XML for progressive disclosure
///
/// Only includes name and description, suitable for the initial metadata
/// loading phase (~100 tokens per skill).
pub fn metadata_to_xml(skills: &[SkillInstance]) -> String {
    let mut xml = String::from("<skills>\n");

    for skill in skills {
        xml.push_str(&format!(
            "  <skill name=\"{}\" description=\"{}\" />\n",
            escape_xml_attr(&skill.name),
            escape_xml_attr(&skill.description)
        ));
    }

    xml.push_str("</skills>");
    xml
}

#[cfg(test)]
mod tests {
    use super::*;
    use stencila_schema::{Block, Paragraph, Text};

    fn make_test_skill() -> Skill {
        Skill {
            name: "data-analysis".into(),
            description: "Analyze datasets and generate summary statistics.".into(),
            content: vec![Block::Paragraph(Paragraph::new(vec![
                stencila_schema::Inline::Text(Text::from("Use pandas for data manipulation.")),
            ]))],
            compatibility: Some("Requires Python 3.10+".into()),
            allowed_tools: Some(vec!["Bash(python:*)".into(), "Read".into()]),
            ..Default::default()
        }
    }

    #[test]
    fn skill_to_xml() -> eyre::Result<()> {
        let skill = make_test_skill();
        let xml = to_xml(&skill);

        assert!(xml.starts_with("<skill name=\"data-analysis\">"));
        assert!(xml.contains(
            "<description>Analyze datasets and generate summary statistics.</description>"
        ));
        assert!(xml.contains("<instructions>"));
        assert!(xml.contains("Use pandas for data manipulation."));
        assert!(xml.contains("<compatibility>Requires Python 3.10+</compatibility>"));
        assert!(xml.contains("<allowed-tools>Bash(python:*) Read</allowed-tools>"));
        assert!(xml.ends_with("</skill>"));

        Ok(())
    }

    #[test]
    fn xml_escaping() -> eyre::Result<()> {
        let skill = Skill {
            name: "test-skill".into(),
            description: "Handles <html> & \"quotes\"".into(),
            content: vec![Block::Paragraph(Paragraph::new(vec![
                stencila_schema::Inline::Text(Text::from("Use x < y && a > b")),
            ]))],
            compatibility: Some("Requires foo & bar".into()),
            ..Default::default()
        };

        let xml = to_xml(&skill);

        // In text content, " does not need escaping (only in attributes)
        assert!(xml.contains("&lt;html&gt; &amp; \"quotes\""));
        assert!(xml.contains("x &lt; y &amp;&amp; a &gt; b"));
        assert!(xml.contains("foo &amp; bar"));

        Ok(())
    }

    #[test]
    fn metadata_xml_escaping() -> eyre::Result<()> {
        let skills = vec![SkillInstance {
            inner: Skill {
                name: "test".into(),
                description: "Has \"quotes\" & <tags>".into(),
                ..Default::default()
            },
            ..Default::default()
        }];

        let xml = metadata_to_xml(&skills);

        assert!(xml.contains("&quot;quotes&quot; &amp; &lt;tags&gt;"));

        Ok(())
    }

    #[test]
    fn skills_metadata_to_xml() -> eyre::Result<()> {
        let skills = vec![
            SkillInstance {
                inner: Skill {
                    name: "data-analysis".into(),
                    description: "Analyze datasets.".into(),
                    ..Default::default()
                },
                ..Default::default()
            },
            SkillInstance {
                inner: Skill {
                    name: "code-review".into(),
                    description: "Review code for issues.".into(),
                    ..Default::default()
                },
                ..Default::default()
            },
        ];

        let xml = metadata_to_xml(&skills);

        assert!(xml.starts_with("<skills>"));
        assert!(xml.contains("name=\"data-analysis\""));
        assert!(xml.contains("description=\"Analyze datasets.\""));
        assert!(xml.contains("name=\"code-review\""));
        assert!(xml.ends_with("</skills>"));

        Ok(())
    }
}
