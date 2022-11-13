use super::*;

#[derive(Eq, PartialEq)]
pub struct TyVar {
    pub name: String,
}

#[derive(Eq, PartialEq)]
pub enum Kind {
    Star,
    Arrow(Arc<Kind>, Arc<Kind>),
}

#[derive(Eq, PartialEq, Clone)]
pub struct TyCon {
    pub name: String,
    // pub kind: Option<Arc<Kind>>,
    // pub id: i32, // 0 = unknown, >0 = user-defined, <0 = bulit-in
}

// Node of type ast tree with user defined additional information
pub struct TypeNode {
    pub ty: Type,
    pub info: Arc<TypeInfo>,
}

impl Clone for TypeNode {
    fn clone(&self) -> Self {
        TypeNode {
            ty: self.ty.clone(),
            info: self.info.clone(),
        }
    }
}

impl TypeNode {
    // Create new type node with default info.
    fn new(ty: Type) -> Self {
        Self {
            ty: ty,
            info: Arc::new(TypeInfo::default()),
        }
    }

    // Create shared new type node with default info.
    fn new_arc(ty: Type) -> Arc<Self> {
        Arc::new(Self::new(ty))
    }

    // Set new info for shared instance.
    pub fn set_info(self: Arc<Self>, info: Arc<TypeInfo>) -> Arc<Self> {
        let mut ret = (*self).clone();
        ret.info = info;
        Arc::new(ret)
    }

    // Set new type for shared instance.
    pub fn set_ty(self: &Arc<Self>, ty: Type) -> Arc<Self> {
        let mut ret = (**self).clone();
        ret.ty = ty;
        Arc::new(ret)
    }
}

// Variant of type
pub enum Type {
    TyVar(Arc<TyVar>),
    TyCon(Arc<TyCon>),
    TyApp(Arc<TypeNode>, Arc<TypeNode>),
    FunTy(Arc<TypeNode>, Arc<TypeNode>),
}

impl Clone for Type {
    fn clone(&self) -> Self {
        match self {
            Type::TyVar(x) => Type::TyVar(x.clone()),
            Type::TyApp(x, y) => Type::TyApp(x.clone(), y.clone()),
            Type::FunTy(x, y) => Type::FunTy(x.clone(), y.clone()),
            Type::TyCon(tc) => Type::TyCon(tc.clone()),
        }
    }
}

impl TypeNode {
    pub fn to_string(&self) -> String {
        match &self.ty {
            Type::TyVar(v) => v.name.clone(),
            Type::TyApp(tyfun, arg) => {
                // Convert something like `(Pair<Int>)<Double>` to a form `(Pair, {Int, Double})`.
                let mut args: Vec<Arc<TypeNode>> = vec![arg.clone()];
                let mut now_tyfun = tyfun.clone();
                loop {
                    match &now_tyfun.ty {
                        Type::TyApp(next_tyfun, arg) => {
                            args.push(arg.clone());
                            now_tyfun = next_tyfun.clone();
                        }
                        _ => {
                            break;
                        }
                    }
                }
                args.reverse();
                let tycon = now_tyfun.to_string();
                let args: Vec<String> = args.iter().map(|a| a.clone().to_string()).collect();
                let mut res: String = Default::default();
                res += &tycon;
                res += "<";
                res += &args.join(", ");
                res += ">";
                res
            }
            Type::FunTy(src, dst) => {
                let src_brace_needed = match src.ty {
                    Type::FunTy(_, _) => true,
                    _ => false,
                };
                let src = src.clone().to_string();
                let dst = dst.clone().to_string();
                let mut res: String = Default::default();
                if src_brace_needed {
                    res += "(";
                    res += &src;
                    res += ")";
                } else {
                    res += &src;
                }
                res += " => ";
                res += &dst;
                res
            }
            Type::TyCon(tc) => tc.name.clone(),
        }
    }
}

pub fn kind_star() -> Arc<Kind> {
    Arc::new(Kind::Star)
}

pub fn kind_arrow(src: Arc<Kind>, dst: Arc<Kind>) -> Arc<Kind> {
    Arc::new(Kind::Arrow(src, dst))
}

pub fn tyvar_from_name(var_name: &str) -> Arc<TyVar> {
    Arc::new(TyVar {
        name: String::from(var_name),
    })
}

pub fn type_tyvar(var_name: &str) -> Arc<TypeNode> {
    TypeNode::new_arc(Type::TyVar(tyvar_from_name(var_name)))
}

pub fn type_var_from_tyvar(tyvar: Arc<TyVar>) -> Arc<TypeNode> {
    TypeNode::new_arc(Type::TyVar(tyvar))
}

pub fn type_fun(src: Arc<TypeNode>, dst: Arc<TypeNode>) -> Arc<TypeNode> {
    TypeNode::new_arc(Type::FunTy(src, dst))
}

pub fn type_tyapp(tyfun: Arc<TypeNode>, param: Arc<TypeNode>) -> Arc<TypeNode> {
    TypeNode::new_arc(Type::TyApp(tyfun, param))
}

pub fn type_tycon(tycon: &Arc<TyCon>) -> Arc<TypeNode> {
    TypeNode::new_arc(Type::TyCon(tycon.clone()))
}

pub fn tycon(name: &str /*, kind: &Arc<Kind>, id: i32*/) -> Arc<TyCon> {
    Arc::new(TyCon {
        name: String::from(name),
        // kind: kind.clone(),
        // id: id,
    })
}

// Additional information of types.
#[derive(Default, Clone)]
pub struct TypeInfo {}

impl TypeNode {
    // Calculate free type variables.
    pub fn free_vars(self: &Arc<Self>) -> HashSet<String> {
        let mut free_vars = HashSet::<String>::default();
        let ty = match &self.ty {
            Type::TyVar(tv) => {
                free_vars.insert(tv.name.clone());
            }
            Type::TyApp(tyfun, arg) => {
                free_vars.extend(tyfun.free_vars());
                free_vars.extend(arg.free_vars());
            }
            Type::FunTy(input, output) => {
                free_vars.extend(input.free_vars());
                free_vars.extend(output.free_vars());
            }
            Type::TyCon(_) => {}
        };
        free_vars
    }
}

// Scheme = forall<a1,..,> (...type...)
pub struct Scheme {
    pub vars: HashSet<String>,
    pub ty: Arc<TypeNode>,
}

impl Scheme {
    // Create new instance.
    pub fn new_arc(vars: HashSet<String>, ty: Arc<TypeNode>) -> Arc<Scheme> {
        Arc::new(Scheme { vars, ty })
    }

    // Create new instance.
    pub fn new_arc_from_str(vars: &[&str], ty: Arc<TypeNode>) -> Arc<Scheme> {
        Self::new_arc(HashSet::from_iter(vars.iter().map(|s| s.to_string())), ty)
    }

    // Get free type variables.
    pub fn free_vars(&self) -> HashSet<String> {
        let mut ret = self.ty.free_vars();
        for var in &self.vars {
            ret.remove(var);
        }
        ret
    }

    // Stringify.
    pub fn to_string(&self) -> String {
        let mut ret = String::default();
        if self.vars.len() > 0 {
            ret += "for<";
            for (i, var) in self.vars.iter().enumerate() {
                ret += var;
                if i < self.vars.len() - 1 {
                    ret += ",";
                }
            }
            ret += "> ";
        }
        ret += &self.ty.to_string();
        ret
    }
}
