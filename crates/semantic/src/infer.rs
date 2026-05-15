use std::collections::HashMap;
use crate::ty::SemTy;

pub struct Unifier {
    substitutions: HashMap<usize, SemTy>,
    next_var: usize,
}

impl Unifier {
    pub fn new() -> Self {
        Unifier { substitutions: HashMap::new(), next_var: 0 }
    }

    pub fn new_var(&mut self) -> SemTy {
        let v = self.next_var;
        self.next_var += 1;
        SemTy::Generic(format!("?{}", v))
    }

    pub fn unify(&mut self, t1: &SemTy, t2: &SemTy) -> Result<(), String> {
        if t1 == t2 { return Ok(()); }
        match (t1, t2) {
            (SemTy::Infer, _) | (_, SemTy::Infer) => Ok(()),
            (SemTy::Ref(r1), SemTy::Ref(r2)) => {
                if r1.is_mut != r2.is_mut {
                    return Err("mutable/immutable reference mismatch".into());
                }
                self.unify(&r1.inner, &r2.inner)
            }
            (SemTy::Tuple(t1s), SemTy::Tuple(t2s)) if t1s.len() == t2s.len() => {
                for (a, b) in t1s.iter().zip(t2s.iter()) {
                    self.unify(a, b)?;
                }
                Ok(())
            }
            _ => Err(format!("cannot unify {} and {}", t1.name(), t2.name())),
        }
    }
}

impl Default for Unifier {
    fn default() -> Self {
        Self::new()
    }
}
