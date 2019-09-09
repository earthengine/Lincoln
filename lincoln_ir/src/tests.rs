#[cfg(test)]
mod test {
    use crate::PreCompileProgram;
    use failure::Error;
    use lincoln_compiled::CodeRef::Termination;
    use lincoln_compiled::Context;
    use lincoln_compiled::EvalFn;
    use lincoln_compiled::ExternEntry;
    use lincoln_compiled::{unwrap, wrap};
    #[test]
    fn test_call_ret() -> Result<(), Error> {
        let mut prog: PreCompileProgram = Default::default();
        // call with a direct return is equal to no doing anything
        prog.define_call("test", "rec1", 2, "rec2").unwrap();
        prog.define_ret("rec1", 0)?;
        prog.set_export("test")?;

        let cprog = prog
            .compile(
                vec![ExternEntry::Eval {
                    name: "rec2".into(),
                    eval: EvalFn::stateless(|c| {
                        assert_eq!(unwrap::<i32>(c.pop().unwrap()).unwrap(), 3);
                        assert_eq!(unwrap::<i32>(c.pop().unwrap()).unwrap(), 2);
                        assert_eq!(unwrap::<i32>(c.pop().unwrap()).unwrap(), 1);
                        Ok(Termination)
                    }),
                }]
                .into_iter(),
            )
            .unwrap();
        let mut ctx = Context::default();
        ctx.push(wrap(1i32));
        ctx.push(wrap(2i32));
        ctx.push(wrap(3i32));
        let mut next = cprog.get_export_ent("test", 0).unwrap();
        next = cprog.eval(&mut ctx, &next).unwrap();
        assert_eq!(format!("{:?}", next), "🎯-0");
        next = cprog.eval(&mut ctx, &next).unwrap();
        assert_eq!(format!("{:?}", next), "🗨-0");
        next = cprog.eval(&mut ctx, &next).unwrap();
        assert_eq!(format!("{:?}", next), "🛑");

        Ok(())
    }
    #[test]
    fn test_call() -> Result<(), Error> {
        let mut prog: PreCompileProgram = Default::default();
        prog.define_call("test", "rec1", 2, "rec2").unwrap();
        prog.set_export("test")?;

        let cprog = prog
            .compile(
                vec![
                    ExternEntry::Eval {
                        name: "rec1".into(),
                        eval: EvalFn::stateless(|c| {
                            let v = c.pop().unwrap();
                            assert_eq!(unwrap::<i32>(c.pop().unwrap()).unwrap(), 2);
                            assert_eq!(unwrap::<i32>(c.pop().unwrap()).unwrap(), 1);
                            v.eval(c, 0)
                        }),
                    },
                    (ExternEntry::Eval {
                        name: "rec2".into(),
                        eval: EvalFn::stateless(|c| {
                            assert_eq!(unwrap::<i32>(c.pop().unwrap()).unwrap(), 3);
                            Ok(Termination)
                        }),
                    }),
                ]
                .into_iter(),
            )
            .unwrap();

        let mut ctx = Context::default();
        ctx.push(wrap(1i32));
        ctx.push(wrap(2i32));
        ctx.push(wrap(3i32));
        let mut next = cprog.get_export_ent("test", 0).unwrap();
        next = cprog.eval(&mut ctx, &next).unwrap();
        assert_eq!(format!("{:?}", next), "🗨-0");
        next = cprog.eval(&mut ctx, &next).unwrap();
        assert_eq!(format!("{:?}", next), "🗨-1");
        next = cprog.eval(&mut ctx, &next).unwrap();
        assert_eq!(format!("{:?}", next), "🛑");

        Ok(())
    }
}
