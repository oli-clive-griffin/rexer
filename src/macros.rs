use std::collections::HashMap;

use crate::parser::Node;


pub struct Macro {
    pub params: Vec<String>,
    pub body: Node,
}

struct Scope {
    bindings: HashMap<String, Macro>,
}

// (macro (identity node) node)
// (macro (switch a b) '(b a))
//
// ((identity switch) 3 inc)
// expand -> (switch 3 inc)
// expand -> (inc 3)

/// scope.bindings: {
///   "switch": Macro { code: ... }
/// }
///

fn eval_macro(themacro: Macro, )

pub fn expand(node: Node, scope: Scope) -> Node {
    return match node {
        // optionally find a macro associated to the identifier

        // expand all children
        Node::List(list) => {
            let expanded_list = list.iter().cloned().map(|item| expand(item, scope)).collect();
            match list[0] {
                Node::Symbol(name) => {
                    if let Some(val) = scope.bindings.get(&name) {
                        val
                    }
                    todo!();
                }
                Node::List(_) => todo!(),
                Node::Literal(_) => todo!(),
                Node::Op(_) => todo!(),
                Node::Fn => todo!(),
                Node::If => todo!(),
                Node::Let => todo!(),
                Node::Quote => todo!(),
            };
            todo!()
        },
        
        // leaf nodes
        Node::Literal(_) => node,
        Node::Ident(_) => node,
        Node::Op(_) => node,
        Node::Fn => node,
        Node::If => node,
        Node::Let => node,
        Node::Quote => node,
    }
}
