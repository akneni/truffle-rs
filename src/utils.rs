use std::collections::HashMap;

use crate::parser::DataType;

pub struct DeclaredObjects<T: PartialOrd + PartialEq> {
    objects: Vec<Vec<T>>,
}

impl<T: PartialOrd + PartialEq> DeclaredObjects<T> {
    pub fn new() -> Self {
        DeclaredObjects {
            objects: vec![vec![]]
        }
    }

    pub fn push(&mut self, object: T) {
        self.objects.last_mut().unwrap().push(object);
    }

    pub fn contains(&self, object: &T) -> bool {
        for scope in self.objects.iter().rev() {
            if scope.contains(object) {
                return true;
            }
        }
        false
    }

    pub fn push_scope(&mut self) {
        self.objects.push(vec![]);
    }

    pub fn pop_scope(&mut self) {
        self.objects.pop();
    }
}


pub struct VarLst {
    vars: Vec<HashMap<String, DataType>>
}

impl VarLst {
    pub fn new() -> Self {
        VarLst {
            vars: vec![HashMap::new()]
        }
    }

    pub fn insert(&mut self, var: String, dtype: DataType) {
        self.vars.last_mut().unwrap().insert(var, dtype);
    }

    pub fn get(&self, var: &String) -> Option<DataType> {
        for scope in self.vars.iter().rev() {
            if let Some(d) = scope.get(var) {
                return Some(d.clone());
            }
        }
        None
    }

    pub fn push_scope(&mut self) {
        self.vars.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        self.vars.pop();
    }
}