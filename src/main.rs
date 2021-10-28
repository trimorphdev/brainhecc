use cranelift::prelude::*;
use cranelift_codegen::binemit::{NullStackMapSink, NullTrapSink};
use cranelift_module::{DataContext, default_libcall_names, FuncId, Linkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule};
use std::fs::read_to_string;
use std::str::Chars;

#[derive(Debug, PartialEq)]
enum Ast {
    /// +
    IncData,

    /// >
    IncPtr,

    /// -
    DecData,

    /// <
    DecPtr,

    /// .
    Output,

    /// ,
    Input,

    /// []
    Loop {
        insts: Vec<Ast>,
    },
}

fn parse_loop(chars: &mut Chars) -> Vec<Ast> {
    let mut ast = vec![];
    
    while let Some(char) = chars.next() {
        if char == '+' {
            ast.push(Ast::IncData);
        } else if char == '>' {
            ast.push(Ast::IncPtr);
        } else if char == '-' {
            ast.push(Ast::DecData);
        } else if char == '<' {
            ast.push(Ast::DecPtr);
        } else if char == '.' {
            ast.push(Ast::Output);
        } else if char == ',' {
            ast.push(Ast::Input);
        } else if char == '[' {
            ast.push(Ast::Loop {
                insts: parse_loop(chars)
            });
        } else if char == ']' {
            return ast;
        }
    }

    ast
}

fn parse(str: String) -> Vec<Ast> {
    let mut chars = str.chars();
    let mut ast = vec![];

    while let Some(char) = chars.next() {
        if char == '+' {
            ast.push(Ast::IncData);
        } else if char == '>' {
            ast.push(Ast::IncPtr);
        } else if char == '-' {
            ast.push(Ast::DecData);
        } else if char == '<' {
            ast.push(Ast::DecPtr);
        } else if char == '.' {
            ast.push(Ast::Output);
        } else if char == ',' {
            ast.push(Ast::Input);
        } else if char == '[' {
            ast.push(Ast::Loop {
                insts: parse_loop(&mut chars)
            });
        }
    }

    ast
}

fn compile_instruction(iter: &mut std::slice::Iter<Ast>, bcx: &mut FunctionBuilder, 
    module: &mut ObjectModule, putchar: FuncId, getchar: FuncId) {
    if let Some(ast) = iter.next() {
        match ast {
            Ast::IncData => {
                // Calculate amount to increment
                let mut iter_peek = iter.clone();
                let mut inc = 1;
                while let Some(ast) = iter_peek.next() {
                    if ast == &Ast::IncData {
                        iter.next();
                        inc += 1;
                    } else {
                        break;
                    }
                }

                let ptr = bcx.use_var(Variable::new(0));
                let value = bcx.ins().load(types::I8, MemFlags::new(), ptr, 0);
                let value1 = bcx.ins().iconst(types::I8, inc);
                let other = bcx.ins().iadd(value, value1);
                let ptr = bcx.use_var(Variable::new(0));
                bcx.ins().store(MemFlags::new(), other, ptr, 0);
            },
            Ast::IncPtr => {
                let ptr = bcx.use_var(Variable::new(0));
                let mut iter_peek = iter.clone();
                let mut inc = 1;
                while let Some(ast) = iter_peek.next() {
                    if ast == &Ast::IncPtr {
                        iter.next();
                        inc += 1;
                    } else {
                        break;
                    }
                }
                let value1 = bcx.ins().iadd_imm(ptr, inc);
                bcx.def_var(Variable::new(0), value1);
            },
            Ast::DecData => {
                // Calculate amount to increment
                let mut iter_peek = iter.clone();
                let mut inc = 1;
                while let Some(ast) = iter_peek.next() {
                    if ast == &Ast::DecData {
                        iter.next();
                        inc += 1;
                    } else {
                        break;
                    }
                }

                let ptr = bcx.use_var(Variable::new(0));
                let value = bcx.ins().load(types::I8, MemFlags::new(), ptr, 0);
                let value1 = bcx.ins().iconst(types::I8, inc);
                let other = bcx.ins().isub(value, value1);
                let ptr = bcx.use_var(Variable::new(0));
                bcx.ins().store(MemFlags::new(), other, ptr, 0);
            },
            Ast::DecPtr => {
                let ptr = bcx.use_var(Variable::new(0));
                let mut iter_peek = iter.clone();
                let mut inc = 1;
                while let Some(ast) = iter_peek.next() {
                    if ast == &Ast::DecPtr {
                        iter.next();
                        inc += 1;
                    } else {
                        break;
                    }
                }
                let inc_value = bcx.ins().iconst(
                    types::Type::triple_pointer_type(&target_lexicon::Triple::host()), inc);
                let value1 = bcx.ins().isub(ptr, inc_value);
                bcx.def_var(Variable::new(0), value1);
            },
            Ast::Output => {
                let ptr = bcx.use_var(Variable::new(0));
                let value = bcx.ins().load(types::I8, MemFlags::new(), ptr, 0);
                let putchar_ = module.declare_func_in_func(putchar, bcx.func);
                bcx.ins().call(putchar_, &[value]);
            },
            Ast::Input => {
                let ptr = bcx.use_var(Variable::new(0));
                //let value = bcx.ins().load(types::I8, MemFlags::new(), ptr, 0);
                let getchar_ = module.declare_func_in_func(getchar, bcx.func);
                let res = {
                    let res = bcx.ins().call(getchar_, &[]);
                    let value = bcx.inst_results(res);
                    value[0].clone()
                };
                bcx.ins().store(MemFlags::new(), res, ptr, 0);
            },
            Ast::Loop {
                insts
            } => {
                let header = bcx.create_block();
                let body = bcx.create_block();
                let exit = bcx.create_block();

                bcx.ins().jump(header, &[]);

                bcx.switch_to_block(header);

                let ptr = bcx.use_var(Variable::new(0));
                let value = bcx.ins().load(types::I8, MemFlags::new(), ptr, 0);
                bcx.ins().brnz(value, body, &[]);
                bcx.ins().jump(exit, &[]);

                bcx.switch_to_block(body);

                let mut inst_iter = insts.iter();
                while let Some(_) = inst_iter.clone().next() {
                    compile_instruction(&mut inst_iter, bcx, module, putchar, getchar);
                }
                bcx.ins().jump(header, &[]);

                bcx.switch_to_block(exit);

                bcx.seal_block(header);
                bcx.seal_block(body);
                bcx.seal_block(exit);
            },
        }
    }
}

fn compile(ast: Vec<Ast>) -> Vec<u8> {
    let mut flag_builder = settings::builder();
    flag_builder.set("use_colocated_libcalls", "false").unwrap();
    flag_builder.set("is_pic", "false").unwrap();

    let isa_builder = cranelift_native::builder().unwrap_or_else(|msg| {
        panic!("host machine not supported: {}", msg);
    });
    let isa = isa_builder.finish(settings::Flags::new(flag_builder));

    let mut module = ObjectModule::new(ObjectBuilder::new(isa, [1, 2, 3, 4], default_libcall_names())
        .unwrap());
    
    let mut ctx = module.make_context();
    let mut func_ctx = FunctionBuilderContext::new();

    let mut sig_main = module.make_signature();
    sig_main.returns.push(AbiParam::new(types::I32));

    let func_main = module
        .declare_function("main", Linkage::Export, &sig_main)
        .unwrap();
    
    ctx.func.signature = sig_main;
    ctx.func.name = ExternalName::user(0, func_main.as_u32());

    let mut bcx = FunctionBuilder::new(&mut ctx.func, &mut func_ctx);
    
    // Compile the AST
    let block = bcx.create_block();
    bcx.switch_to_block(block);
    bcx.seal_block(block);
    
    // Declare the data in the main function
    let data_ptr_addr = {
        let data = module.declare_data("data", Linkage::Local, true, false).unwrap();
        let mut main_data_ctx = DataContext::new();
        main_data_ctx.define_zeroinit(30000); // 30000 bytes of zero data.
        module.define_data(data, &mut main_data_ctx).unwrap();

        //let main_data = module.declare_data_in_func(data, &mut bcx.func);

        let mut data_ctx = DataContext::new();
        data_ctx.define_zeroinit(8);
        //data_ctx.write_data_addr(0, main_data, 0);

        let ptr_data = module.declare_anonymous_data(true, false).unwrap();
        module.declare_data_in_data(ptr_data, &mut data_ctx);
        module.define_data(ptr_data, &data_ctx).unwrap();

        module.declare_data_in_func(ptr_data, &mut bcx.func)
    };

    let data_ptr = Variable::new(0);
    bcx.declare_var(data_ptr, types::Type::triple_pointer_type(&target_lexicon::Triple::host()));

    let putchar = {
        let mut sig = module.make_signature();
        sig.params.push(AbiParam::new(types::I8)); // char

        module.declare_function("putchar", Linkage::Import, &sig).unwrap()
    };

    let getchar = {
        let mut sig = module.make_signature();
        sig.returns.push(AbiParam::new(types::I8)); // char

        module.declare_function("getchar", Linkage::Import, &sig).unwrap()
    };

    {
        let value = bcx.ins().global_value(
            types::Type::triple_pointer_type(&target_lexicon::Triple::host()), data_ptr_addr);
        bcx.def_var(data_ptr, value);
    }

    let mut a = ast.iter();
    while let Some(_) = a.clone().next() {
        compile_instruction(&mut a, &mut bcx, &mut module, putchar, getchar);
    }

    let v = bcx.ins().iconst(types::I32, 0);
    bcx.ins().return_(&[v]);

    let mut trap_sink = NullTrapSink {};
    let mut stack_map_sink = NullStackMapSink {};
    module
        .define_function(func_main, &mut ctx, &mut trap_sink, &mut stack_map_sink)
        .unwrap();
    
    module.finish().emit().unwrap()
}

fn main() -> Result<(), ()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 3 {
        // There was an incorrect amount of arguments.
        println!("brainheck: usage: brainheck <input> <output>");
        return Err(());
    }

    if let Ok(f) = read_to_string(&args[1]) {
        //println!("{:x?}", parse(f));
        std::fs::write(&args[2], compile(parse(f))).unwrap();
    } else {
        println!("brainheck: input file doesn't exist");
    }

    Ok(())
}
