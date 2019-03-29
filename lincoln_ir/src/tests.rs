#[cfg(test)]
mod test {
    use crate::PreCompileProgram;
    use lincoln_compiled::ExternEntry;
    use lincoln_compiled::EvalFn;
    use lincoln_compiled::CodeRef::Termination;
    use lincoln_compiled::Context;
    use lincoln_compiled::{wrap,unwrap};
    #[test]
    fn test_call_ret() {
        let mut prog:PreCompileProgram = Default::default();
        prog.define_call("test", "rec1", 2, "rec2").unwrap();
        prog.define_ret("rec1", 0);
        prog.set_export("test");

        let cprog = prog.compile([
            (|| ExternEntry::Eval {
                name: "rec2".into(),
                eval: EvalFn::stateless(|p,mut c| {
                    assert_eq!(unwrap::<i32>(c.pop().unwrap(), p).unwrap(), 3);
                    assert_eq!(unwrap::<i32>(c.pop().unwrap(), p).unwrap(), 2);
                    assert_eq!(unwrap::<i32>(c.pop().unwrap(), p).unwrap(), 1);
                    Ok((Termination,c))
                }),
            }) as fn() -> ExternEntry
        ]).unwrap();
        let mut ctx = Context::default();
        ctx.push(wrap(1i32));
        ctx.push(wrap(2i32));
        ctx.push(wrap(3i32));
        let next = cprog.get_export_ent("test", 0).unwrap();
        let (next, ctx) = cprog.eval(ctx, &next).unwrap();
        assert_eq!(format!("{:?}", next),"^#0");
        let (next, ctx) = cprog.eval(ctx, &next).unwrap();
        assert_eq!(format!("{:?}", next),"^@0");
        let (next, ctx) = cprog.eval(ctx, &next).unwrap();
        assert_eq!(format!("{:?}", next),"^⟂");
    }
    #[test]
    fn test_call() {
        let mut prog:PreCompileProgram = Default::default();
        prog.define_call("test", "rec1", 2, "rec2").unwrap();
        prog.set_export("test");

        let cprog = prog.compile([
            || ExternEntry::Eval {
                name: "rec1".into(),
                eval: EvalFn::stateless(|p,mut c| {
                    let v = c.pop().unwrap();
                    assert_eq!(unwrap::<i32>(c.pop().unwrap(), p).unwrap(), 2);
                    assert_eq!(unwrap::<i32>(c.pop().unwrap(), p).unwrap(), 1);
                    v.eval(p, c, 0)
                }),
            },
            (|| ExternEntry::Eval {
                name: "rec2".into(),
                eval: EvalFn::stateless(|p,mut c| {
                    assert_eq!(unwrap::<i32>(c.pop().unwrap(), p).unwrap(), 3);
                    Ok((Termination,c))
                }),
            }) as fn() -> ExternEntry
        ]).unwrap();

        let mut ctx = Context::default();
        ctx.push(wrap(1i32));
        ctx.push(wrap(2i32));
        ctx.push(wrap(3i32));
        let next = cprog.get_export_ent("test", 0).unwrap();
        let (next, ctx) = cprog.eval(ctx, &next).unwrap();
        assert_eq!(format!("{:?}", next),"^@0");
        let (next, ctx) = cprog.eval(ctx, &next).unwrap();
        assert_eq!(format!("{:?}", next),"^@1");
        let (next, ctx) = cprog.eval(ctx, &next).unwrap();
        assert_eq!(format!("{:?}", next),"^⟂");
    }
}