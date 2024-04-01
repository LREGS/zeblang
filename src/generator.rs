use crate::parser::{AssignNode, ExitNode, ExpressionNode, StatementNode};

use std::collections::HashMap;

#[derive(Debug)]
pub struct Generator {
    assembly: String,
    stack_pointer: usize,
    loops: usize,
    variables: HashMap<String, usize>,
}

impl Generator {
    pub fn new() -> Self {
        Self {
            assembly: String::from("global _start\n_start:\n"),
            stack_pointer: 0,
            loops: 0,
            variables: HashMap::new(),
        }
    }

    fn indent(level: usize) -> String {
        "    ".repeat(level).to_string()
    }

    fn push(&mut self, register: &str, level: usize) -> () {
        self.assembly += format!("{}push {}\n", Self::indent(level), register).as_str();
        self.stack_pointer += 1;
    }

    fn pop(&mut self, register: &str, level: usize) -> () {
        self.assembly += format!("{}pop {}\n", Self::indent(level), register).as_str();
        self.stack_pointer -= 1;
    }

    fn generic(&mut self, cmd: &str, level: usize) -> () {
        self.assembly += format!("{}{}\n", Self::indent(level), cmd).as_str();
    }

    // more research needed
    fn parse_print(&mut self) -> () {
        todo!();
    }

    // add a speacial terminator value like 0x?? or something
    fn parse_range(&mut self) -> () {
        self.pop("rax", 1);
        self.generic("mov rbx, 0", 1);
        self.generic(format!("loop{}:", &self.loops).as_str(), 1);
        self.push("rbx", 2);
        self.generic("inc rbx", 2);
        self.generic("cmp rax, rbx", 2);
        self.generic(format!("je exit{}", &self.loops).as_str(), 2);
        self.generic(format!("jmp loop{}", &self.loops).as_str(), 2);
        self.generic(format!("exit{}:", &self.loops).as_str(), 1);
        self.loops += 1;
    }

    fn generate_expr(&mut self, expr: ExpressionNode) -> () {
        match expr {
            ExpressionNode::Value(value) => {
                self.generic(format!("mov rax, {}", value).as_str(), 1);
                self.push("rax", 1);
            }
            ExpressionNode::Var(value) => {
                let variable_position = self.variables.get(&value).unwrap();
                let var = format!(
                    "[rsp + {}]",
                    (self.stack_pointer - variable_position - 1) * 8
                );
                self.generic(format!("mov rax, {}", var).as_str(), 1);
                self.push("rax", 1);
            }
            ExpressionNode::Infix(expr_1, op, expr_2) => {
                self.generate_expr(*expr_1);
                self.generate_expr(*expr_2);
                self.pop("rbx", 1);
                self.pop("rax", 1);
                match op.as_str() {
                    "+" => self.generic("add rax, rbx", 1),
                    "-" => self.generic("sub rax, rbx", 1),
                    "*" => self.generic("imul rbx", 1),
                    _ => todo!(),
                }
                self.push("rax", 1);
            }
            ExpressionNode::Callable(name, expr) => {
                self.generate_expr(*expr);
                match name.as_str() {
                    "print(" => self.parse_print(),
                    "range(" => self.parse_range(),
                    _ => todo!("undeclared function"),
                }
            }
        }
    }

    pub fn generate(&mut self, program: Vec<StatementNode>) -> String {
        dbg!(&program);
        for line in program.into_iter() {
            match line {
                StatementNode::Exit(exit_node) => {
                    let ExitNode::Expression(expr_node) = exit_node;
                    self.generate_expr(expr_node);
                    self.generic("mov rax, 60", 1);
                    self.pop("rdi", 1);
                    self.generic("syscall", 1);
                }
                StatementNode::Assign(name, assign_node) => {
                    self.variables.insert(name, self.stack_pointer);
                    let AssignNode::Expression(expr_node) = assign_node;
                    self.generate_expr(expr_node);
                }
                StatementNode::For(var, expr_node) => {
                    self.variables.insert(var, self.stack_pointer);
                    self.generate_expr(expr_node);
                    self.loops += 1;
                }
                StatementNode::EndFor => todo!(),
            };
        }
        self.assembly.to_owned()
    }
}
