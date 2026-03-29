use crate::models::NodeType;

const PROJECT_TEMPLATE: &str = include_str!("../../../templates/project.md");
const PHASE_TEMPLATE: &str = include_str!("../../../templates/phase.md");
const TASK_TEMPLATE: &str = include_str!("../../../templates/task.md");

pub fn get_template(node_type: NodeType) -> &'static str {
    match node_type {
        NodeType::Project => PROJECT_TEMPLATE,
        NodeType::Phase => PHASE_TEMPLATE,
        NodeType::Task => TASK_TEMPLATE,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_templates_are_nonempty() {
        assert!(get_template(NodeType::Project).contains("## Goal"));
        assert!(get_template(NodeType::Phase).contains("## Objective"));
        assert!(get_template(NodeType::Task).contains("## Description"));
    }
}
