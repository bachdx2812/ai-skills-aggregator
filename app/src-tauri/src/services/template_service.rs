use crate::models::{AgentType, SkillFormat};

pub struct TemplateService;

impl TemplateService {
    pub fn get_template(agent: &AgentType, format: &SkillFormat) -> String {
        match (agent, format) {
            (AgentType::Claude, SkillFormat::Markdown) => Self::claude_md_template(),
            (AgentType::Claude, SkillFormat::Python) => Self::claude_py_template(),
            (AgentType::Cursor, SkillFormat::PlainText) => Self::cursor_template(),
            (AgentType::Cursor, SkillFormat::Markdown) => Self::cursor_template(),
            (AgentType::ContinueDev, SkillFormat::Json) => Self::continue_template(),
            (AgentType::Aider, SkillFormat::Yaml) => Self::aider_template(),
            (AgentType::Aider, SkillFormat::PlainText) => Self::aider_prompt_template(),
            _ => Self::generic_md_template(),
        }
    }

    fn claude_md_template() -> String {
        r#"# Skill Name

Brief description of what this skill does.

## Usage

When to use this skill and how it helps.

## Instructions

1. First instruction
2. Second instruction
3. Third instruction

## Examples

```
Example usage here
```

## Notes

- Additional notes
- Limitations or caveats
"#.to_string()
    }

    fn claude_py_template() -> String {
        r#"#!/usr/bin/env python3
"""
Skill Name

Brief description of what this skill does.
"""

import sys

def main():
    """Main entry point for the skill."""
    # Your skill implementation here
    print("Hello from skill!")
    return 0

if __name__ == "__main__":
    sys.exit(main())
"#.to_string()
    }

    fn cursor_template() -> String {
        r#"# Cursor Rules

You are an expert assistant following these guidelines:

## Code Style
- Write clean, readable code
- Follow best practices
- Add meaningful comments

## Behavior
- Be concise and helpful
- Explain your reasoning
- Suggest improvements

## Restrictions
- Don't make assumptions
- Ask for clarification when needed
"#.to_string()
    }

    fn continue_template() -> String {
        r#"{
  "name": "Custom Skill",
  "version": "1.0.0",
  "description": "Brief description",
  "systemMessage": "You are a helpful assistant.",
  "contextProviders": [],
  "slashCommands": []
}
"#.to_string()
    }

    fn aider_template() -> String {
        r#"# Aider Configuration

model: gpt-4
edit-format: diff
auto-commits: true

# Custom settings
map-tokens: 1024
"#.to_string()
    }

    fn aider_prompt_template() -> String {
        r#"You are an expert developer. Follow these guidelines:

1. Write clean, maintainable code
2. Add tests for new features
3. Document complex logic
4. Follow project conventions
"#.to_string()
    }

    fn generic_md_template() -> String {
        r#"# Skill Name

## Description

What this skill does.

## Instructions

1. Step one
2. Step two
3. Step three
"#.to_string()
    }
}
