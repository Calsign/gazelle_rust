use protobuf::RepeatedField;

use messages_rust_proto as pb;

pub fn cfg_atom_to_proto(atom: parser::BExprAtom) -> pb::BExprAtom {
    let mut ret = pb::BExprAtom::default();

    match atom {
        parser::BExprAtom::Option { option } => {
            ret.set_value(option);
        }
        parser::BExprAtom::KeyOption { key, value } => {
            let mut key_value = pb::KeyValue::default();
            key_value.set_key(key);
            key_value.set_value(value);
            ret.set_key_value(key_value);
        }
    }

    ret
}

pub fn cfg_to_proto(config: parser::ConfigFlag) -> pb::BExpr {
    let mut ret = pb::BExpr::default();

    match config {
        parser::ConfigFlag::Terminal(atom) => {
            ret.set_atom(cfg_atom_to_proto(atom));
        }
        parser::ConfigFlag::Const(c) => {
            ret.set_constant(c);
        }
        parser::ConfigFlag::And(left, right) => {
            let mut seq = pb::BExprSeq::default();
            seq.set_values(RepeatedField::from_vec(vec![
                cfg_to_proto(*left),
                cfg_to_proto(*right),
            ]));
            ret.set_and(seq);
        }
        parser::ConfigFlag::Or(left, right) => {
            let mut seq = pb::BExprSeq::default();
            seq.set_values(RepeatedField::from_vec(vec![
                cfg_to_proto(*left),
                cfg_to_proto(*right),
            ]));
            ret.set_or(seq);
        }
        parser::ConfigFlag::Not(inner) => {
            ret.set_not(cfg_to_proto(*inner));
        }
    }

    ret
}

pub fn proto_atom_to_cfg(atom: pb::BExprAtom) -> parser::BExprAtom {
    match atom.atom.expect("no atom") {
        pb::BExprAtom_oneof_atom::value(value) => parser::BExprAtom::Option { option: value },
        pb::BExprAtom_oneof_atom::key_value(key_value) => parser::BExprAtom::KeyOption {
            key: key_value.key,
            value: key_value.value,
        },
    }
}

pub fn proto_to_cfg(proto: pb::BExpr) -> parser::ConfigFlag {
    match proto.expr.expect("no expr") {
        pb::BExpr_oneof_expr::atom(atom) => parser::ConfigFlag::Terminal(proto_atom_to_cfg(atom)),
        pb::BExpr_oneof_expr::constant(constant) => parser::ConfigFlag::Const(constant),
        pb::BExpr_oneof_expr::not(not) => parser::ConfigFlag::Not(Box::new(proto_to_cfg(*not))),
        pb::BExpr_oneof_expr::and(and) => parser::bexpr_join(
            parser::ConfigFlag::And,
            and.values.into_iter().map(proto_to_cfg),
        )
        .expect("empty and"),
        pb::BExpr_oneof_expr::or(or) => parser::bexpr_join(
            parser::ConfigFlag::Or,
            or.values.into_iter().map(proto_to_cfg),
        )
        .expect("empty or"),
    }
}
