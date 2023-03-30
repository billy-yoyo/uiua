use std::{
    f64::{consts::PI, INFINITY},
    fmt,
};

use crate::{
    array::Array, function::FunctionId, io::*, lex::Simple, value::*, vm::CallEnv, RuntimeResult,
};

macro_rules! primitive {
    ($((
        $($args:literal$(($outputs:expr))?,)?
        $name:ident $({$modifier:ident: $margs:literal})?
        $(,$ident:literal)? $(,$ascii:ident)? $(+ $character:literal)?
    )),* $(,)?) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum Primitive {
            $($name,)*
            Io(IoOp)
        }

        impl Primitive {
            pub const ALL: [Self; 0 $(+ {stringify!($name); 1})*] = [
                $(Self::$name,)*
            ];
            #[allow(path_statements)]
            pub fn name(&self) -> Option<&'static str > {
                match self {
                    $(Primitive::$name => { None::<&'static str> $(;Some($ident))? },)*
                    Primitive::Io(op) => Some(op.name())
                }
            }
            pub fn ascii(&self) -> Option<Simple> {
                match self {
                    $($(Primitive::$name => Some(Simple::$ascii),)?)*
                    _ => None
                }
            }
            pub fn unicode(&self) -> Option<char> {
                match self {
                    $($(Primitive::$name => Some($character),)?)*
                    _ => None
                }
            }
            pub fn from_simple(s: Simple) -> Option<Self> {
                match s {
                    $($(Simple::$ascii => Some(Self::$name),)?)*
                    _ => None
                }
            }
            pub fn from_unicode(c: char) -> Option<Self> {
                match c {
                    $($($character => Some(Self::$name),)?)*
                    _ => None
                }
            }
            pub fn is_modifier(&self) -> bool {
                match self {
                    $($(Primitive::$name => {
                        stringify!($modifier);
                        true
                    },)?)*
                    _ => false
                }
            }
            pub fn modifier_args(&self) -> Option<u8> {
                match self {
                    $($(Primitive::$name => Some($margs),)?)*
                    _ => None
                }
            }
            pub fn args(&self) -> Option<u8> {
                match self {
                    $($(Primitive::$name => Some($args),)?)*
                    Primitive::Io(op) => Some(op.args()),
                    _ => None
                }
            }
            pub fn outputs(&self) -> Option<u8> {
                match self {
                    $($($(Primitive::$name => $outputs.into(),)?)?)*
                    Primitive::Io(op) => op.outputs(),
                    _ => Some(1)
                }
            }
        }
    };
}

primitive!(
    // Stack ops
    (1(2), Dup, "duplicate" + '.'),
    (2(3), Over, "over" + ','),
    (2(2), Flip, "flip" + '~'),
    (1(0), Pop, "pop" + ';'),
    (1(None), Unpack, "unpack" + '⊔'),
    // Pervasive monadic ops
    (1, Sign, "sign" + '$'),
    (1, Not, "not" + '¬'),
    (1, Neg, "negate", Backtick + '¯'),
    (1, Abs, "absolute" + '⌵'),
    (1, Sqrt, "sqrt" + '√'),
    (1, Sin, "sine"),
    (1, Cos, "cosine"),
    (1, Asin),
    (1, Acos),
    (1, Floor, "floor" + '⌊'),
    (1, Ceil, "ceiling" + '⌈'),
    (1, Round, "round" + '⁅'),
    // Pervasive dyadic ops
    (2, Eq, "equals", Equal),
    (2, Ne, "not equals", BangEqual + '≠'),
    (2, Lt, "less than" + '<'),
    (2, Le, "less or equal", LessEqual + '≤'),
    (2, Gt, "greater than" + '>'),
    (2, Ge, "greater or equal", GreaterEqual + '≥'),
    (2, Add, "add" + '+'),
    (2, Sub, "subtract" + '-'),
    (2, Mul, "multiply", Star + '×'),
    (2, Div, "divide", Percent + '÷'),
    (2, Mod, "modulus" + '◿'),
    (2, Pow, "power" + 'ⁿ'),
    (2, Root),
    (2, Min, "minimum" + '↧'),
    (2, Max, "maximum" + '↥'),
    (2, Atan, "atangent"),
    // Monadic array ops
    (1, Len, "length" + '⇀'),
    (1, Rank, "rank" + '⸫'),
    (1, Shape, "shape" + '△'),
    (1, Range, "range" + '⇡'),
    (1, First, "first" + '⊢'),
    (1, Reverse, "reverse" + '⇌'),
    (1, Enclose, "enclose" + '⊓'),
    (1, Normalize, "normalize" + '□'),
    (1, Deshape, "deshape" + '♭'),
    (1, Transpose, "transpose" + '⍉'),
    (1, Sort, "sort" + '∧'),
    (1, Grade, "grade" + '⍋'),
    (1, Indices, "indices" + '⊘'),
    (1, Classify, "classify" + '⊛'),
    (1, Deduplicate, "deduplicate" + '⊝'),
    // Dyadic array ops
    (2, Match, "match" + '≅'),
    (2, NoMatch, "notmatch" + '≇'),
    (2, Join, "join" + '≍'),
    (2, Pair, "pair" + '⚇'),
    (2, Couple, "couple" + '⊟'),
    (2, Pick, "pick" + '⊡'),
    (2, Select, "select" + '⊏'),
    (2, Take, "take" + '↙'),
    (2, Drop, "drop" + '↘'),
    (2, Reshape, "reshape" + '↯'),
    (2, Rotate, "rotate" + '↻'),
    (2, Windows, "windows" + '◫'),
    (2, Replicate, "replicate" + '‡'),
    (2, Member, "member" + '∈'),
    (2, Group, "group" + '⊕'),
    (2, IndexOf, "indexof" + '⊙'),
    // Triadic array op
    (3, Put),
    // Modifiers
    (Reduce { modifier: 1 }, "reduce" + '/'),
    (Fold { modifier: 1 }, "fold" + '⌿'),
    (Scan { modifier: 1 }, "scan" + '\\'),
    (Each { modifier: 1 }, "each" + '⸪'),
    (Cells { modifier: 1 }, "cells" + '≡'),
    (Table { modifier: 1 }, "table" + '⊞'),
    (Repeat { modifier: 1 }, "repeat" + '⍥'),
    (Invert { modifier: 1 }, "invert" + '↩'),
    (Under { modifier: 2 }, "under" + '⍜'),
    (Try { modifier: 2 }, "try" + '?'),
    // Misc
    (2, Assert, "assert" + '!'),
    (0, Nop, "noop" + '·'),
    (1(None), Call, "call" + ':'),
    (1, String, "string"),
    (1, Parse, "parsenumber"),
    (1, Use, "use"),
    // Constants
    (0(1), Pi, "pi" + 'π'),
    (0(1), Infinity, "infinity" + '∞')
);

fn _keep_primitive_small(_: std::convert::Infallible) {
    let _: [u8; 1] = unsafe { std::mem::transmute(Some(Primitive::Not)) };
}

impl fmt::Display for Primitive {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(c) = self.unicode() {
            write!(f, "{}", c)
        } else if let Some(s) = self.ascii() {
            write!(f, "{}", s)
        } else if let Some(s) = self.name() {
            write!(f, "{}", s)
        } else {
            write!(f, "{:?}", self)
        }
    }
}

impl Primitive {
    pub fn inverse(&self) -> Option<Self> {
        use Primitive::*;
        Some(match self {
            Flip => Flip,
            Neg => Neg,
            Not => Not,
            Sin => Asin,
            Cos => Acos,
            Reverse => Reverse,
            Add => Sub,
            Sub => Add,
            Mul => Div,
            Div => Mul,
            Pow => Root,
            Root => Pow,
            Pick => Put,
            _ => return None,
        })
    }
    pub fn from_name(name: &str) -> Option<Self> {
        let lower = name.to_lowercase();
        if let Some(io) = IoOp::from_name(&lower) {
            return Some(Primitive::Io(io));
        }
        if lower == "pi" || lower == "π" {
            return Some(Primitive::Pi);
        }
        if name.len() < 3 {
            return None;
        }
        let mut matching = Primitive::ALL.into_iter().filter(|p| {
            p.name()
                .map_or(false, |i| i.to_lowercase().starts_with(&lower))
        });
        let res = matching.next()?;
        let exact_match = res.name().map_or(false, |i| i == lower);
        (exact_match || matching.next().is_none()).then_some(res)
    }
    pub(crate) fn run<B: IoBackend>(&self, env: &mut CallEnv<B>) -> RuntimeResult {
        match self {
            Primitive::Pi => env.push(PI),
            Primitive::Infinity => env.push(INFINITY),
            Primitive::Nop => {}
            Primitive::Not => env.monadic_env(Value::not)?,
            Primitive::Neg => env.monadic_env(Value::neg)?,
            Primitive::Abs => env.monadic_env(Value::abs)?,
            Primitive::Sign => env.monadic_env(Value::sign)?,
            Primitive::Sqrt => env.monadic_env(Value::sqrt)?,
            Primitive::Sin => env.monadic_env(Value::sin)?,
            Primitive::Cos => env.monadic_env(Value::cos)?,
            Primitive::Asin => env.monadic_env(Value::asin)?,
            Primitive::Acos => env.monadic_env(Value::acos)?,
            Primitive::Floor => env.monadic_env(Value::floor)?,
            Primitive::Ceil => env.monadic_env(Value::ceil)?,
            Primitive::Round => env.monadic_env(Value::round)?,
            Primitive::Eq => env.dyadic_env(Value::is_eq)?,
            Primitive::Ne => env.dyadic_env(Value::is_ne)?,
            Primitive::Lt => env.dyadic_env(Value::is_lt)?,
            Primitive::Le => env.dyadic_env(Value::is_le)?,
            Primitive::Gt => env.dyadic_env(Value::is_gt)?,
            Primitive::Ge => env.dyadic_env(Value::is_ge)?,
            Primitive::Add => env.dyadic_env(Value::add)?,
            Primitive::Sub => env.dyadic_env(Value::sub)?,
            Primitive::Mul => env.dyadic_env(Value::mul)?,
            Primitive::Div => env.dyadic_env(Value::div)?,
            Primitive::Mod => env.dyadic_env(Value::modulus)?,
            Primitive::Pow => env.dyadic_env(Value::pow)?,
            Primitive::Root => env.dyadic_env(Value::root)?,
            Primitive::Min => env.dyadic_env(Value::min)?,
            Primitive::Max => env.dyadic_env(Value::max)?,
            Primitive::Atan => env.dyadic_env(Value::atan2)?,
            Primitive::Match => env.dyadic(|a, b| a == b)?,
            Primitive::NoMatch => env.dyadic(|a, b| a != b)?,
            Primitive::Join => env.dyadic_mut_env(Value::join)?,
            Primitive::Reshape => env.dyadic_mut_env(Value::reshape)?,
            Primitive::Transpose => env.monadic_mut(Value::transpose)?,
            Primitive::Pick => env.dyadic_mut_env(Value::pick)?,
            Primitive::Replicate => env.dyadic_mut_env(Value::replicate)?,
            Primitive::Take => env.dyadic_mut_env(Value::take)?,
            Primitive::Drop => env.dyadic_mut_env(Value::drop)?,
            Primitive::Rotate => env.dyadic_mut_env(Value::rotate)?,
            Primitive::Enclose => env.monadic_mut(Value::enclose)?,
            Primitive::Normalize => env.monadic_mut_env(Value::normalize)?,
            Primitive::Pair => env.dyadic_mut(Value::pair)?,
            Primitive::Couple => env.dyadic_mut_env(Value::couple)?,
            Primitive::Sort => env.monadic_mut_env(Value::sort)?,
            Primitive::Grade => env.monadic_mut_env(Value::grade)?,
            Primitive::Indices => env.monadic_mut_env(Value::indices)?,
            Primitive::Select => env.dyadic_mut_env(Value::select)?,
            Primitive::Windows => env.dyadic_mut_env(Value::windows)?,
            Primitive::Classify => env.monadic_mut_env(Value::classify)?,
            Primitive::Deduplicate => env.monadic_mut_env(Value::deduplicate)?,
            Primitive::Member => env.dyadic_mut(Value::member)?,
            Primitive::Group => env.dyadic_mut_env(Value::group)?,
            Primitive::IndexOf => env.dyadic_mut_env(Value::index_of)?,
            Primitive::Call => env.call()?,
            Primitive::Parse => env.monadic_mut_env(Value::parse_num)?,
            Primitive::Unpack => {
                let value = env.pop(1)?;
                if value.is_array() {
                    for v in value.into_array().into_values().into_iter().rev() {
                        env.push(v);
                    }
                } else {
                    env.push(value);
                }
            }
            Primitive::Put => {
                let mut index = env.pop(1)?;
                let value = env.pop(2)?;
                let array = env.pop(3)?;
                index.put(value, array, &env.env())?;
                env.push(index);
            }
            Primitive::Dup => {
                let x = env.top_mut(1)?.clone();
                env.push(x);
            }
            Primitive::Flip => {
                let a = env.pop(1)?;
                let b = env.pop(2)?;
                env.push(a);
                env.push(b);
            }
            Primitive::Over => {
                let a = env.pop(1)?;
                let b = env.pop(2)?;
                env.push(b.clone());
                env.push(a);
                env.push(b);
            }
            Primitive::Pop => {
                env.pop(1)?;
            }
            Primitive::Invert => {
                let f = env.pop(1)?;
                if !f.is_function() {
                    return Err(env.error("Only functions can be inverted"));
                }
                let f_inv = f.function().inverse(&env.env(), false)?;
                env.push(f_inv);
                env.call()?;
            }
            Primitive::Under => {
                let f = env.pop(1)?;
                let g = env.pop(2)?;
                if !f.is_function() || !g.is_function() {
                    return Err(env.error("Only functions can be inverted"));
                }
                let f_inv = f.function().inverse(&env.env(), true)?;
                env.push(f);
                env.call()?;
                env.push(g);
                env.call()?;
                env.push(f_inv);
                env.call()?;
            }
            Primitive::Fold => {
                let f = env.pop(1)?;
                let mut acc = env.pop(2)?;
                let xs = env.pop(3)?;
                if !xs.is_array() {
                    env.push(acc);
                    env.push(xs);
                    env.push(f);
                    return env.call();
                }
                for cell in xs.into_array().into_values() {
                    env.push(acc);
                    env.push(cell);
                    env.push(f.clone());
                    env.call()?;
                    acc = env.pop("folded function result")?;
                }
                env.push(acc);
            }
            Primitive::Reduce => {
                let f = env.pop(1)?;
                let xs = env.pop(2)?;
                if !xs.is_array() {
                    env.push(xs);
                    return Ok(());
                }
                let mut cells = xs.into_array().into_values().into_iter();
                let Some(mut acc) = cells.next() else {
                    return Err(env.error("Cannot reduce empty array"));
                };
                for cell in cells {
                    env.push(cell);
                    env.push(acc);
                    env.push(f.clone());
                    env.call()?;
                    acc = env.pop("reduced function result")?;
                }
                env.push(acc);
            }
            Primitive::Each => {
                let f = env.pop(1)?;
                let xs = env.pop(2)?;
                if !xs.is_array() {
                    env.push(xs);
                    env.push(f);
                    return env.call();
                }
                let (shape, values) = xs.into_array().into_shape_flat_values();
                let mut new_values = Vec::with_capacity(values.len());
                for val in values {
                    env.push(val);
                    env.push(f.clone());
                    env.call()?;
                    new_values.push(env.pop("each's function result")?);
                }
                env.push(Array::from((shape, new_values)).normalized_type());
            }
            Primitive::Cells => {
                let f = env.pop(1)?;
                let xs = env.pop(2)?;
                if !xs.is_array() {
                    env.push(xs);
                    env.push(f);
                    return env.call();
                }
                let array = xs.into_array();
                let mut cells = Vec::with_capacity(array.len());
                for cell in array.into_values() {
                    env.push(cell);
                    env.push(f.clone());
                    env.call()?;
                    cells.push(env.pop("cells' function result")?);
                }
                env.push(Array::from(cells).normalized());
            }
            Primitive::Table => {
                let f = env.pop(1)?;
                let xs = env.pop(2)?;
                let ys = env.pop(3)?;
                if !xs.is_array() && !ys.is_array() {
                    env.push(ys);
                    env.push(xs);
                    env.push(f);
                    return env.call();
                }
                let a = if xs.is_array() {
                    xs.into_array()
                } else {
                    Array::from(xs)
                };
                let b = if ys.is_array() {
                    ys.into_array()
                } else {
                    Array::from(ys)
                };
                let mut table = Vec::with_capacity(a.len());
                for a in a.into_values() {
                    let mut row = Vec::with_capacity(b.len());
                    for b in b.clone().into_values() {
                        env.push(b);
                        env.push(a.clone());
                        env.push(f.clone());
                        env.call()?;
                        row.push(env.pop("tabled function result")?);
                    }
                    table.push(Value::from(Array::from(row).normalized_type()));
                }
                env.push(Array::from(table).normalized());
            }
            Primitive::Scan => {
                let f = env.pop(1)?;
                let xs = env.pop(2)?;
                if !xs.is_array() {
                    env.push(xs);
                    return Ok(());
                }
                let arr = xs.into_array();
                let ty = arr.ty();
                let len = arr.len();
                let mut cells = arr.into_values().into_iter();
                let Some(mut acc) = cells.next() else {
                    env.push(Array::from(ty));
                    return Ok(())
                };
                let mut scanned = Vec::with_capacity(len);
                scanned.push(acc.clone());
                for cell in cells {
                    env.push(cell);
                    env.push(acc.clone());
                    env.push(f.clone());
                    env.call()?;
                    acc = env.pop("scanned function result")?;
                    scanned.push(acc.clone());
                }
                env.push(Array::from(scanned).normalized());
            }
            Primitive::Repeat => {
                let f = env.pop(1)?;
                let mut acc = env.pop(2)?;
                let n = env.pop(3)?;
                let Some(n) = n.as_nat() else {
                    return Err(env.error("Repetitions must be a natural number"));
                };
                for _ in 0..n {
                    env.push(acc);
                    env.push(f.clone());
                    env.call()?;
                    acc = env.pop("repeated function result")?;
                }
                env.push(acc);
            }
            Primitive::Try => {
                let f = env.pop(1)?;
                let handler = env.pop(2)?;
                let size = env.stack_size();
                env.push(f);
                if let Err(e) = env.call() {
                    env.truncate(size);
                    env.push(e.message());
                    env.push(handler);
                    env.call()?;
                }
            }
            Primitive::Assert => {
                let msg = env.pop(1)?;
                let cond = env.pop(2)?;
                if !(cond.is_num() && (cond.number() - 1.0).abs() < 1e-10) {
                    return Err(env.error(&msg.to_string()));
                }
            }
            Primitive::Len => env.monadic(|v| v.len() as f64)?,
            Primitive::Rank => env.monadic(|v| v.rank() as f64)?,
            Primitive::Shape => {
                env.monadic(|v| Array::from_iter(v.shape().iter().map(|i| *i as f64)))?
            }
            Primitive::Range => env.monadic_mut_env(Value::range)?,
            Primitive::Reverse => env.monadic_mut(Value::reverse)?,
            Primitive::Deshape => env.monadic_mut(Value::deshape)?,
            Primitive::First => env.monadic_mut_env(Value::first)?,
            Primitive::String => env.monadic(|v| v.to_string())?,
            Primitive::Use => {
                let mut lib = env.pop(1)?;
                let name = env.pop(2)?;
                if !name.is_array() || !name.array().is_chars() {
                    return Err(env.error("Use name must be a string"));
                }
                let arr = lib.coerce_array();
                let name = name.array().chars().iter().collect::<String>();
                let lowername = name.to_lowercase();
                let f = arr.data(
                    |_, _| None,
                    |_, _| None,
                    |_, values| {
                        values.iter().filter(|v| v.is_function()).find_map(|v| {
                            let f = v.function();
                            matches!(&f.id, FunctionId::Named(n) if n.as_str().to_lowercase() == lowername)
                                .then(|| f.clone())
                        })
                    },
                ).ok_or_else(|| env.error(format!("No function found for {name:?}")))?;
                env.push(f);
            }
            Primitive::Io(io) => io.run(env)?,
        }
        Ok(())
    }
}

#[test]
fn primitive_from_name() {
    assert_eq!(Primitive::from_name("rev"), Some(Primitive::Reverse));
    assert_eq!(Primitive::from_name("re"), None);
    assert_eq!(Primitive::from_name("resh"), Some(Primitive::Reshape));
}

#[test]
fn glyph_size() {
    use std::{fs::File, io::Write};
    let mut file = File::create("glyph_test.txt").unwrap();
    writeln!(file, "A |").unwrap();
    writeln!(file, "a |").unwrap();
    for p in Primitive::ALL {
        if let Some(glyph) = p.unicode() {
            writeln!(file, "{} |", glyph).unwrap();
        }
    }
}