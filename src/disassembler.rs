use crate::vm::{BytecodeChunk, ConstantObject, ConstantValue, Op};

pub fn disassemble(bc: &BytecodeChunk) -> String {
    let mut pc = 0;
    let mut lines = "".to_string();
    while pc < bc.code.len() {
        let op: Op = bc.code[pc].try_into().expect("invalid opcode");
        let line: String = match op {
            Op::Add => "Add".to_string(),
            Op::Sub => "Sub".to_string(),
            Op::Mul => "Mul".to_string(),
            Op::Div => "Div".to_string(),
            Op::Return => "Return".to_string(),
            Op::Cons => "Cons".to_string(),
            Op::DebugEnd => "DebugEnd".to_string(),
            Op::Pop => "pop".to_string(),
            Op::Constant => {
                pc += 1;
                let idx = bc.code[pc];
                format!("Constant\n  val: {:?}", bc.constants[idx as usize])
            }
            Op::Jump => {
                pc += 1;
                let offset = bc.code[pc];
                format!("Jump\n  offset: {offset}")
            }
            Op::CondJump => {
                pc += 1;
                let offset = bc.code[pc];
                format!("CondJump\n  offset: {offset}")
            }
            Op::FuncCall => {
                pc += 1;
                let arity = bc.code[pc];
                format!("FuncCall\n  arity: {arity}")
            }
            Op::DeclareGlobal => {
                pc += 1;
                let name_idx = bc.code[pc];
                let name = match &bc.constants[name_idx as usize] {
                    ConstantValue::Object(o) => match o {
                        ConstantObject::String(s) => s,
                        got => panic!("expected string for global name, got {:?}", got),
                    },
                    got => panic!("expected object for global name, got {:?}", got),
                };

                // value is on the stack
                format!("DeclareGlobal\n  name: {name} (value on stack)")
            }
            Op::ReferenceGlobal => {
                pc += 1;
                let name_idx = bc.code[pc];
                let name = match &bc.constants[name_idx as usize] {
                    ConstantValue::Object(o) => match o {
                        ConstantObject::String(s) => s,
                        got => panic!("expected string for global name, got {:?}", got),
                    },
                    got => panic!("expected object for global name, got {:?}", got),
                };

                format!("ReferenceGlobal\n  name: {name}")
            }
            Op::ReferenceLocal => {
                pc += 1;
                let idx = bc.code[pc];
                format!("ReferenceLocal\n  idx: {idx}")
            }
            Op::Define => {
                pc += 1;
                let idx = bc.code[pc];
                format!("Define\n  idx: {idx}")
            }
            Op::GT => "GT".to_string(),
            Op::LT => "LT".to_string(),
            Op::GTE => "GTE".to_string(),
            Op::LTE => "LTE".to_string(),
            Op::Print => "PRINT".to_string(),
            Op::Quote => "QUOTE".to_string(),
            Op::ReferenceUpvalue => unimplemented!(),
            Op::SetUpvalue => unimplemented!(),
            Op::Closure => {
                let mut s = "CLOSURE\n".to_string();
                pc += 1;
                let idx = bc.code[pc];
                let closure = match &bc.constants[idx as usize] {
                    ConstantValue::Object(o) => match o {
                        ConstantObject::Closure(c) => c,
                        got => panic!("expected function for closure, got {:?}", got),
                    },
                    got => panic!("expected object for closure, got {:?}", got),
                };
                let num_uv = closure.num_upvalues;
                for i in 0..num_uv {
                    pc += 1;
                    let is_local = bc.code[pc];
                    pc += 1;
                    let idx = bc.code[pc];
                    s.push_str(
                        format!("  upvalue: {i} is_local: {is_local} idx: {idx}\n").as_str(),
                    );
                }

                s
            }
        };
        lines.push_str(line.as_str());
        lines.push('\n');
        pc += 1;
    }
    lines
}
