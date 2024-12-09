use crate::Language;
use tracing::warn;
use tree_sitter::{Node, TreeCursor};

const MAX_CURSOR_DEPTH: u32 = 300;

pub trait Visitor {
    fn process_node(&mut self, cursor: &mut TreeCursor) {
        let node = cursor.node();
        let kind = node.kind();
        let language = self.language();

        if self.skip_node(&node) {
            return;
        }

        if cursor.depth() > MAX_CURSOR_DEPTH {
            // This is a safety check to prevent stack overflow
            // due to infinite recursion on deeply nested trees.
            warn!("Max cursor depth reached");
            return;
        }

        if language.invisible_container_nodes().contains(&kind) {
            self.visit_invisible_container(cursor);
        } else if language.if_nodes().contains(&kind) {
            self.visit_if(cursor);
        } else if language.elsif_nodes().contains(&kind) {
            self.visit_elsif(cursor);
        } else if language.else_nodes().contains(&kind) {
            self.visit_else(cursor);
        } else if language.conditional_assignment_nodes().contains(&kind) {
            self.visit_conditional_assignment(cursor);
        } else if language.ternary_nodes().contains(&kind) {
            self.visit_ternary(cursor);
        } else if language.switch_nodes().contains(&kind) {
            self.visit_switch(cursor);
        } else if language.case_nodes().contains(&kind) {
            self.visit_case(cursor);
        } else if language.loop_nodes().contains(&kind) {
            self.visit_loop(cursor);
        } else if language.except_nodes().contains(&kind) {
            self.visit_except(cursor);
        } else if language.try_expression_nodes().contains(&kind) {
            self.visit_try_expression(cursor);
        } else if language.jump_nodes().contains(&kind) {
            self.visit_jump(cursor);
        } else if language.return_nodes().contains(&kind) {
            self.visit_return(cursor);
        } else if language.binary_nodes().contains(&kind) {
            self.visit_binary(cursor);
        } else if language.field_nodes().contains(&kind) {
            self.visit_field(cursor);
        } else if language.call_nodes().contains(&kind) {
            self.visit_call(cursor);
        } else if language.function_nodes().contains(&kind) {
            self.visit_function(cursor);
        } else if language.closure_nodes().contains(&kind) {
            self.visit_closure(cursor);
        } else if language.comment_nodes().contains(&kind) {
            self.visit_comment(cursor);
        } else if language.string_nodes().contains(&kind) {
            self.visit_string(cursor);
        } else if language.block_nodes().contains(&kind) {
            self.visit_block(cursor);
        } else {
            self.visit_unknown(cursor);
        }
    }

    fn visit_invisible_container(&mut self, cursor: &mut TreeCursor) {
        self.process_children(cursor);
    }

    fn visit_if(&mut self, cursor: &mut TreeCursor) {
        self.process_children(cursor);
    }

    fn visit_elsif(&mut self, cursor: &mut TreeCursor) {
        self.process_children(cursor);
    }

    fn visit_else(&mut self, cursor: &mut TreeCursor) {
        self.process_children(cursor);
    }

    fn visit_conditional_assignment(&mut self, cursor: &mut TreeCursor) {
        self.process_children(cursor);
    }

    fn visit_ternary(&mut self, cursor: &mut TreeCursor) {
        self.process_children(cursor);
    }

    fn visit_switch(&mut self, cursor: &mut TreeCursor) {
        self.process_children(cursor);
    }

    fn visit_case(&mut self, cursor: &mut TreeCursor) {
        self.process_children(cursor);
    }

    fn visit_loop(&mut self, cursor: &mut TreeCursor) {
        self.process_children(cursor);
    }

    fn visit_except(&mut self, cursor: &mut TreeCursor) {
        self.process_children(cursor);
    }

    fn visit_try_expression(&mut self, cursor: &mut TreeCursor) {
        self.process_children(cursor);
    }

    fn visit_jump(&mut self, cursor: &mut TreeCursor) {
        self.process_children(cursor);
    }

    fn visit_return(&mut self, cursor: &mut TreeCursor) {
        self.process_children(cursor);
    }

    fn visit_binary(&mut self, cursor: &mut TreeCursor) {
        self.process_children(cursor);
    }

    fn visit_field(&mut self, cursor: &mut TreeCursor) {
        self.process_children(cursor);
    }

    fn visit_call(&mut self, cursor: &mut TreeCursor) {
        self.process_children(cursor);
    }

    fn visit_function(&mut self, cursor: &mut TreeCursor) {
        self.process_children(cursor);
    }

    fn visit_closure(&mut self, cursor: &mut TreeCursor) {
        self.process_children(cursor);
    }

    fn visit_comment(&mut self, cursor: &mut TreeCursor) {
        self.process_children(cursor);
    }

    fn visit_string(&mut self, cursor: &mut TreeCursor) {
        self.process_children(cursor);
    }

    fn visit_block(&mut self, cursor: &mut TreeCursor) {
        self.process_children(cursor);
    }

    fn visit_unknown(&mut self, cursor: &mut TreeCursor) {
        self.process_children(cursor);
    }

    fn process_children(&mut self, cursor: &mut TreeCursor) {
        if cursor.goto_first_child() {
            loop {
                self.process_node(cursor);

                if !cursor.goto_next_sibling() {
                    break;
                }
            }

            cursor.goto_parent();
        }
    }

    fn skip_node(&self, node: &Node) -> bool {
        !node.is_named()
    }

    #[allow(clippy::borrowed_box)]
    fn language(&self) -> &Box<dyn Language + Sync>;
}
