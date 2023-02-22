use codegen::ir::InstBuilderBase;
use cranelift::prelude::*;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{Linkage, Module};

fn brif(v: i32) -> i32 {
    let mut flag_builder = settings::builder();
    // On at least AArch64, "colocated" calls use shorter-range relocations,
    // which might not reach all definitions; we can't handle that here, so
    // we require long-range relocation types.
    flag_builder.set("use_colocated_libcalls", "false").unwrap();
    flag_builder.set("is_pic", "false").unwrap();
    flag_builder.set("opt_level", "speed_and_size").unwrap();
    let isa_builder = cranelift_native::builder().unwrap_or_else(|msg| {
        panic!("host machine is not supported: {}", msg);
    });
    let isa = isa_builder
        .finish(settings::Flags::new(flag_builder))
        .unwrap();
    let builder = JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());

    let mut module = JITModule::new(builder);
    let mut fb_ctx = FunctionBuilderContext::new();
    let mut ctx = module.make_context();
    ctx.want_disasm = true;

    ctx.func.signature.returns.push(AbiParam::new(types::I32));

    let mut fb = FunctionBuilder::new(&mut ctx.func, &mut fb_ctx);

    let entry = fb.create_block();
    fb.switch_to_block(entry);

    let block1 = fb.create_block();
    let block2a = fb.create_block();
    let block2b = fb.create_block();
    let block2c = fb.create_block();
    let block3 = fb.create_block();

    let c = fb.ins().iconst(types::I32, v as i64);
    let loop_count = fb.ins().iconst(types::I32, 10);
    fb.ins().brif(c, block1, &[], block2a, &[loop_count]);

    fb.switch_to_block(block1);
    let c0 = fb.ins().iconst(types::I32, 4);
    fb.ins().jump(block3, &[c0]);

    fb.switch_to_block(block2a);
    let loop_count = fb.append_block_param(block2a, types::I32);
    let one = fb.ins().iconst(types::I32, 1);
    let new_loop_count = fb.ins().isub(loop_count, one);
    fb.ins().jump(block2b, &[]);

    fb.switch_to_block(block2b);
    fb.ins()
        .brif(new_loop_count, block2a, &[new_loop_count], block2c, &[]);

    fb.switch_to_block(block2c);
    let c1 = fb.ins().iconst(types::I32, 8);
    fb.ins().jump(block3, &[c1]);

    fb.switch_to_block(block3);
    let ret_val = fb.append_block_param(block3, types::I32);
    fb.ins().return_(&[ret_val]);

    fb.seal_all_blocks();
    fb.finalize();

    let id = module
        .declare_function("test", Linkage::Export, &ctx.func.signature)
        .unwrap();
    module.define_function(id, &mut ctx).unwrap();

    module.clear_context(&mut ctx);
    module.finalize_definitions().unwrap();
    let code = module.get_finalized_function(id);

    let func: extern "C" fn() -> i32 = unsafe { std::mem::transmute(code) };
    func()
}

fn br_table(v: i32) -> i32 {
    let mut flag_builder = settings::builder();
    // On at least AArch64, "colocated" calls use shorter-range relocations,
    // which might not reach all definitions; we can't handle that here, so
    // we require long-range relocation types.
    flag_builder.set("use_colocated_libcalls", "false").unwrap();
    flag_builder.set("is_pic", "false").unwrap();
    flag_builder.set("opt_level", "speed_and_size").unwrap();
    let isa_builder = cranelift_native::builder().unwrap_or_else(|msg| {
        panic!("host machine is not supported: {}", msg);
    });
    let isa = isa_builder
        .finish(settings::Flags::new(flag_builder))
        .unwrap();
    let builder = JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());

    let mut module = JITModule::new(builder);
    let mut fb_ctx = FunctionBuilderContext::new();
    let mut ctx = module.make_context();
    ctx.want_disasm = true;

    ctx.func.signature.returns.push(AbiParam::new(types::I32));

    let mut fb = FunctionBuilder::new(&mut ctx.func, &mut fb_ctx);

    let entry = fb.create_block();
    let block1 = fb.create_block();
    let block2 = fb.create_block();
    let block3 = fb.create_block();
    let block4 = fb.create_block();
    fb.switch_to_block(entry);

    let c = fb.ins().iconst(types::I32, v as i64);
    let block_call_1 = fb.ins().data_flow_graph_mut().block_call(block1, &[]);
    let block_call_2 = fb.ins().data_flow_graph_mut().block_call(block2, &[]);
    let block_call_3 = fb.ins().data_flow_graph_mut().block_call(block3, &[]);
    let jtd = JumpTableData::new(block_call_1, &[block_call_2, block_call_3]);
    let jt = fb.create_jump_table(jtd);
    fb.ins().br_table(c, jt);

    fb.switch_to_block(block1);
    let c4 = fb.ins().iconst(types::I32, 4);
    fb.ins().jump(block4, &[c4]);

    fb.switch_to_block(block2);
    let c8 = fb.ins().iconst(types::I32, 8);
    fb.ins().jump(block4, &[c8]);

    fb.switch_to_block(block3);
    let c16 = fb.ins().iconst(types::I32, 16);
    fb.ins().jump(block4, &[c16]);

    fb.switch_to_block(block4);
    let ret_val = fb.append_block_param(block4, types::I32);
    fb.ins().return_(&[ret_val]);

    fb.seal_all_blocks();
    fb.finalize();

    let id = module
        .declare_function("test", Linkage::Export, &ctx.func.signature)
        .unwrap();
    module.define_function(id, &mut ctx).unwrap();

    module.clear_context(&mut ctx);
    module.finalize_definitions().unwrap();
    let code = module.get_finalized_function(id);

    let func: extern "C" fn() -> i32 = unsafe { std::mem::transmute(code) };
    func()
}

fn both(v: i32) -> i32 {
    let mut flag_builder = settings::builder();
    // On at least AArch64, "colocated" calls use shorter-range relocations,
    // which might not reach all definitions; we can't handle that here, so
    // we require long-range relocation types.
    flag_builder.set("use_colocated_libcalls", "false").unwrap();
    flag_builder.set("is_pic", "false").unwrap();
    flag_builder.set("opt_level", "speed_and_size").unwrap();
    let isa_builder = cranelift_native::builder().unwrap_or_else(|msg| {
        panic!("host machine is not supported: {}", msg);
    });
    let isa = isa_builder
        .finish(settings::Flags::new(flag_builder))
        .unwrap();
    let builder = JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());

    let mut module = JITModule::new(builder);
    let mut fb_ctx = FunctionBuilderContext::new();
    let mut ctx = module.make_context();
    ctx.want_disasm = true;

    ctx.func.signature.returns.push(AbiParam::new(types::I32));

    let mut fb = FunctionBuilder::new(&mut ctx.func, &mut fb_ctx);

    let entry = fb.create_block();
    fb.switch_to_block(entry);

    let block1 = fb.create_block();
    let block2 = fb.create_block();
    let block3 = fb.create_block();

    let v = fb.ins().iconst(types::I32, v as i64);
    fb.ins().brif(v, block1, &[], block2, &[]);

    fb.switch_to_block(block1);
    let c0 = fb.ins().iconst(types::I32, 0);
    fb.ins().jump(block3, &[c0]);

    fb.switch_to_block(block2);
    let c1 = fb.ins().iconst(types::I32, 1);
    fb.ins().jump(block3, &[c1]);

    let block4 = fb.create_block();
    let block5 = fb.create_block();
    let block6 = fb.create_block();
    let block7 = fb.create_block();

    fb.switch_to_block(block3);
    let jump_idx = fb.append_block_param(block3, types::I32);
    let block_call_4 = fb.ins().data_flow_graph_mut().block_call(block4, &[]);
    let block_call_5 = fb.ins().data_flow_graph_mut().block_call(block5, &[]);
    let block_call_6 = fb.ins().data_flow_graph_mut().block_call(block6, &[]);
    let jtd = JumpTableData::new(block_call_4, &[block_call_5, block_call_6]);
    let jt = fb.create_jump_table(jtd);
    fb.ins().br_table(jump_idx, jt);

    fb.switch_to_block(block4);
    let c4 = fb.ins().iconst(types::I32, 4);
    fb.ins().jump(block7, &[c4]);

    fb.switch_to_block(block5);
    let c8 = fb.ins().iconst(types::I32, 8);
    fb.ins().jump(block7, &[c8]);

    fb.switch_to_block(block6);
    let c8 = fb.ins().iconst(types::I32, 16);
    fb.ins().jump(block7, &[c8]);

    fb.switch_to_block(block7);
    let ret_val = fb.append_block_param(block7, types::I32);
    fb.ins().return_(&[ret_val]);

    fb.seal_all_blocks();
    fb.finalize();

    let id = module
        .declare_function("test", Linkage::Export, &ctx.func.signature)
        .unwrap();
    module.define_function(id, &mut ctx).unwrap();

    module.clear_context(&mut ctx);
    module.finalize_definitions().unwrap();
    let code = module.get_finalized_function(id);

    let func: extern "C" fn() -> i32 = unsafe { std::mem::transmute(code) };
    func()
}

fn main() {
    if cfg!(debug_assertions) {
        simple_logger::SimpleLogger::new().init().unwrap();
    }

    macro_rules! test {
        ($fn:ident, $c:literal, $expect:literal) => {
            let got = $fn($c);
            if got != $expect {
                panic!(
                    "{}({}) failed.  Expected {}, got {}",
                    stringify!($fn),
                    $c,
                    $expect,
                    got
                );
            } else {
                println!("{}({}) ok", stringify!($fn), $c);
            }
        };
    }

    test!(brif, 0, 8);
    test!(brif, 1, 4);
    // test!(br_table, 0, 8);
    // test!(br_table, 1, 16);
    // test!(br_table, 2, 4);
    // test!(br_table, -1, 4);
    // test!(both, 1, 8);
    // test!(both, 0, 16);
}
