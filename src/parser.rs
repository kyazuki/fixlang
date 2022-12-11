#[derive(Parser)]
#[grammar = "grammer.pest"]
struct FixParser;

use pest::error::Error;

use super::*;

// lifetime-free version of pest::Span
#[derive(Clone)]
pub struct Span {
    input: Arc<String>,
    start: usize,
    end: usize,
}

impl Span {
    pub fn from_pair(src: &Arc<String>, pair: &Pair<Rule>) -> Self {
        let span = pair.as_span();
        Self {
            input: src.clone(),
            start: span.start(),
            end: span.end(),
        }
    }

    pub fn unite(&self, other: &Self) -> Self {
        Self {
            input: self.input.clone(),
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }

    // Show source codes around this span.
    pub fn to_string(&self) -> String {
        let span = pest::Span::new(self.input.as_str(), self.start, self.end).unwrap();

        let mut linenum_str_size = 0;
        for line_span in span.lines_span() {
            let linenum = line_span.start_pos().line_col().0;
            linenum_str_size = linenum_str_size.max(linenum.to_string().len());
        }

        let mut ret: String = String::default();
        ret += &format!(
            "at {}:{}-{}:{}\n",
            span.start_pos().line_col().0,
            span.start_pos().line_col().1,
            span.end_pos().line_col().0,
            span.end_pos().line_col().1,
        );
        ret += &(" ".repeat(linenum_str_size) + " | " + "\n");
        for line_span in span.lines_span() {
            let linenum_str = line_span.start_pos().line_col().0.to_string();
            ret +=
                &(linenum_str.clone() + &" ".repeat(linenum_str_size - linenum_str.len()) + " | ");
            ret += String::from(line_span.as_str()).trim_end();
            ret += "\n";
            ret += &(" ".repeat(linenum_str_size) + " | ");
            let start_pos = span.start_pos().max(line_span.start_pos());
            let end_pos = span.end_pos().min(line_span.end_pos());
            let start_col = start_pos.line_col().1;
            let span_len = end_pos.pos() - start_pos.pos();
            ret += &(" ".repeat(start_col - 1) + &"^".repeat(span_len));
            ret += "\n";
        }
        ret
    }
}

fn unite_span(lhs: &Option<Span>, rhs: &Option<Span>) -> Option<Span> {
    match lhs {
        None => rhs.clone(),
        Some(s) => rhs.clone().map(|t| s.unite(&t)),
    }
}

pub fn parse_source(source: &str) -> FixModule {
    let source = Arc::new(String::from(source));
    let file = FixParser::parse(Rule::file, &source);
    let file = match file {
        Ok(res) => res,
        Err(e) => error_exit(&message_parse_error(e)),
    };
    parse_file(file, &source)
}

fn parse_file(mut file: Pairs<Rule>, src: &Arc<String>) -> FixModule {
    let pair = file.next().unwrap();
    match pair.as_rule() {
        Rule::module => return parse_module(pair, src),
        _ => unreachable!(),
    }
}

fn parse_module(pair: Pair<Rule>, src: &Arc<String>) -> FixModule {
    assert_eq!(pair.as_rule(), Rule::module);
    let mut pairs = pair.into_inner();
    let module_name = parse_module_decl(pairs.next().unwrap(), src);
    let mut fix_mod = FixModule::new(module_name);

    let mut type_decls: Vec<TypeDecl> = Vec::new();
    let mut global_symbols_defns: HashMap<Name, (Option<Arc<Scheme>>, Option<Arc<ExprNode>>)> =
        Default::default();
    let mut trait_infos: Vec<TraitInfo> = vec![];
    let mut trait_impls: Vec<TraitInstance> = vec![];

    for pair in pairs {
        match pair.as_rule() {
            Rule::type_decl => {
                type_decls.push(parse_type_decl(pair, src));
            }
            Rule::global_symbol_type_defn => {
                let (name, ty) = parse_global_symbol_type_defn(pair, src);
                if !global_symbols_defns.contains_key(&name) {
                    global_symbols_defns.insert(name, (Some(ty), None));
                } else {
                    let gs = global_symbols_defns.get_mut(&name).unwrap();
                    if gs.0.is_some() {
                        error_exit(&format!("duplicated type declaration for `{}`", name));
                    } else {
                        gs.0 = Some(ty);
                    }
                }
            }
            Rule::global_symbol_defn => {
                let (name, expr) = parse_global_symbol_defn(pair, src);
                if !global_symbols_defns.contains_key(&name) {
                    global_symbols_defns.insert(name, (None, Some(expr)));
                } else {
                    let gs = global_symbols_defns.get_mut(&name).unwrap();
                    if gs.1.is_some() {
                        error_exit(&format!("duplicated definition for `{}`", name));
                    } else {
                        gs.1 = Some(expr);
                    }
                }
            }
            Rule::trait_defn => {
                trait_infos.push(parse_trait_defn(pair, src));
            }
            Rule::trait_impl => {
                trait_impls.push(parse_trait_impl(pair, src));
            }
            _ => unreachable!(),
        }
    }

    fix_mod.set_type_decls(type_decls);
    fix_mod.calculate_tycons();
    fix_mod.set_traits(trait_infos, trait_impls);

    let mut global_symbols: HashMap<NameSpacedName, GlobalSymbol> = Default::default();
    for (name, (ty, expr)) in global_symbols_defns {
        if ty.is_none() {
            error_exit(&format!("symbol `{}` has no type declaration", name));
        }
        if expr.is_none() {
            error_exit(&format!("symbol `{}` has no definition", name));
        }
        global_symbols.insert(
            fix_mod.get_namespaced_name(&name),
            GlobalSymbol {
                ty: ty.unwrap(),
                expr: SymbolExpr::Simple(expr.unwrap()),
            },
        );
    }

    fix_mod.global_symbols = global_symbols;
    fix_mod
}

fn parse_trait_defn(pair: Pair<Rule>, src: &Arc<String>) -> TraitInfo {
    assert_eq!(pair.as_rule(), Rule::trait_defn);
    let mut pairs = pair.into_inner();
    let tyvar = pairs.next().unwrap().as_str().to_string();
    let trait_name = pairs.next().unwrap().as_str().to_string();
    let methods: HashMap<Name, QualType> = pairs
        .map(|pair| parse_trait_member_defn(pair, src))
        .collect();
    TraitInfo {
        id: TraitId { name: trait_name },
        type_var: tyvar_from_name(&tyvar, &kind_star()),
        methods,
        instances: vec![],
    }
}

fn parse_trait_member_defn(pair: Pair<Rule>, src: &Arc<String>) -> (Name, QualType) {
    assert_eq!(pair.as_rule(), Rule::trait_member_defn);
    let mut pairs = pair.into_inner();
    let method_name = pairs.next().unwrap().as_str().to_string();
    let qual_type = parse_qualified_type(pairs.next().unwrap(), src);
    (method_name, qual_type)
}

fn parse_trait_impl(pair: Pair<Rule>, src: &Arc<String>) -> TraitInstance {
    assert_eq!(pair.as_rule(), Rule::trait_impl);
    let mut pairs = pair.into_inner();
    let qual_pred = parse_qual_predicate(pairs.next().unwrap(), src);
    let methods: HashMap<Name, Arc<ExprNode>> = pairs
        .map(|pair| parse_trait_member_impl(pair, src))
        .collect();
    TraitInstance { qual_pred, methods }
}

fn parse_trait_member_impl(pair: Pair<Rule>, src: &Arc<String>) -> (Name, Arc<ExprNode>) {
    assert_eq!(pair.as_rule(), Rule::trait_member_impl);
    let mut pairs = pair.into_inner();
    let method_name = pairs.next().unwrap().as_str().to_string();
    let expr = parse_expr(pairs.next().unwrap(), src);
    (method_name, expr)
}

fn parse_qual_predicate(pair: Pair<Rule>, src: &Arc<String>) -> QualPredicate {
    assert_eq!(pair.as_rule(), Rule::qual_predicate);
    let mut pairs = pair.into_inner();
    let predicate = parse_predicate(pairs.next().unwrap(), src);
    let mut context: Vec<Predicate> = vec![];
    match pairs.next() {
        Some(pair) => {
            context = parse_predicates(pair, src);
        }
        None => {}
    }
    QualPredicate { context, predicate }
}

fn parse_global_symbol_type_defn(pair: Pair<Rule>, src: &Arc<String>) -> (Name, Arc<Scheme>) {
    assert_eq!(pair.as_rule(), Rule::global_symbol_type_defn);
    let mut pairs = pair.into_inner();
    let name = pairs.next().unwrap().as_str().to_string();
    let qual_type = parse_qualified_type(pairs.next().unwrap(), src);
    let preds = qual_type.preds.clone();
    let ty = qual_type.ty.clone();
    (name, Scheme::generalize(ty.free_vars(), preds, ty))
}

fn parse_global_symbol_defn(pair: Pair<Rule>, src: &Arc<String>) -> (Name, Arc<ExprNode>) {
    assert_eq!(pair.as_rule(), Rule::global_symbol_defn);
    let mut pairs = pair.into_inner();
    let name = pairs.next().unwrap().as_str().to_string();
    let expr = parse_expr(pairs.next().unwrap(), src);
    (name, expr)
}

fn parse_qualified_type(pair: Pair<Rule>, src: &Arc<String>) -> QualType {
    assert_eq!(pair.as_rule(), Rule::type_qualified);
    let mut pairs = pair.into_inner();
    let ty = parse_type(pairs.next().unwrap());
    let preds = match pairs.next() {
        Some(pair) => parse_predicates(pair, src),
        None => vec![],
    };
    QualType { preds, ty }
}

fn parse_predicates(pair: Pair<Rule>, src: &Arc<String>) -> Vec<Predicate> {
    assert_eq!(pair.as_rule(), Rule::predicates);
    let mut pairs = pair.into_inner();
    let mut ps: Vec<Predicate> = Default::default();
    for pair in pairs {
        ps.push(parse_predicate(pair, src));
    }
    ps
}

fn parse_predicate(pair: Pair<Rule>, src: &Arc<String>) -> Predicate {
    assert_eq!(pair.as_rule(), Rule::predicate);
    let mut pairs = pair.into_inner();
    let ty = parse_type(pairs.next().unwrap());
    let trait_name = pairs.next().unwrap().as_str().to_string();
    Predicate {
        trait_id: TraitId { name: trait_name },
        ty,
    }
}

fn parse_module_decl(pair: Pair<Rule>, src: &Arc<String>) -> String {
    pair.into_inner().next().unwrap().as_str().to_string()
}

fn parse_type_decl(pair: Pair<Rule>, src: &Arc<String>) -> TypeDecl {
    assert_eq!(pair.as_rule(), Rule::type_decl);
    let mut pairs = pair.into_inner();
    let name = pairs.next().unwrap().as_str();
    let mut fields: Vec<StructField> = Vec::new();
    for pair in pairs {
        fields.push(parse_type_field(pair, src));
    }
    TypeDecl {
        name: name.to_string(),
        value: TypeDeclValue::Struct(Struct { fields }),
    }
}

fn parse_type_field(pair: Pair<Rule>, src: &Arc<String>) -> StructField {
    assert_eq!(pair.as_rule(), Rule::type_field);
    let mut pairs = pair.into_inner();
    let name = pairs.next().unwrap().as_str();
    let ty = parse_type(pairs.next().unwrap());
    StructField {
        name: name.to_string(),
        ty,
    }
}

fn parse_expr(pair: Pair<Rule>, src: &Arc<String>) -> Arc<ExprNode> {
    assert_eq!(pair.as_rule(), Rule::expr);
    let pair = pair.into_inner().next().unwrap();
    parse_expr_type_annotation(pair, src)
}

fn parse_expr_type_annotation(pair: Pair<Rule>, src: &Arc<String>) -> Arc<ExprNode> {
    assert_eq!(pair.as_rule(), Rule::expr_type_annotation);
    let span = Span::from_pair(src, &pair);
    let mut pairs = pair.into_inner();
    let mut expr = parse_expr_rtl_app(pairs.next().unwrap(), src);
    match pairs.next() {
        None => {}
        Some(ty) => {
            expr = expr_tyanno(expr, parse_type(ty), Some(span));
        }
    }
    expr
}

// Parse combinator sequence, e.g., `f x y` or `x & f & g`
fn parse_combinator_sequence(
    pair: Pair<Rule>,
    src: &Arc<String>,
    inner_parser: fn(Pair<Rule>, &Arc<String>) -> Arc<ExprNode>,
) -> Vec<Arc<ExprNode>> {
    pair.into_inner()
        .map(|pair| inner_parser(pair, src))
        .collect()
}

// Parse right to left application sequence, e.g., `g $ f $ x`. (right-associative)
fn parse_expr_rtl_app(pair: Pair<Rule>, src: &Arc<String>) -> Arc<ExprNode> {
    assert_eq!(pair.as_rule(), Rule::expr_rtl_app);
    let exprs = parse_combinator_sequence(pair, src, parse_expr_ltr_app);
    let mut exprs_iter = exprs.iter().rev();
    let mut ret = exprs_iter.next().unwrap().clone();
    for expr in exprs_iter {
        let span = unite_span(&expr.source, &ret.source);
        ret = expr_app(expr.clone(), ret, span);
    }
    ret
}

// Parse left to right application sequence, e.g., `x & f & g`. (left-associative)
fn parse_expr_ltr_app(pair: Pair<Rule>, src: &Arc<String>) -> Arc<ExprNode> {
    assert_eq!(pair.as_rule(), Rule::expr_ltr_app);
    let exprs = parse_combinator_sequence(pair, src, parse_expr_app);
    let mut exprs_iter = exprs.iter();
    let mut ret = exprs_iter.next().unwrap().clone();
    for expr in exprs_iter {
        let span = unite_span(&expr.source, &ret.source);
        ret = expr_app(expr.clone(), ret, span)
            .set_app_order(AppSourceCodeOrderType::ArgumentIsFormer);
    }
    ret
}

// Parse application sequence, e.g., `f x y`. (left-associative)
fn parse_expr_app(pair: Pair<Rule>, src: &Arc<String>) -> Arc<ExprNode> {
    assert_eq!(pair.as_rule(), Rule::expr_app);
    let exprs = parse_combinator_sequence(pair, src, parse_expr_nlr);
    let mut exprs_iter = exprs.iter();
    let mut ret = exprs_iter.next().unwrap().clone();
    for expr in exprs_iter {
        let span = unite_span(&expr.source, &ret.source);
        ret = expr_app(ret, expr.clone(), span);
    }
    ret
}

fn parse_expr_nlr(pair: Pair<Rule>, src: &Arc<String>) -> Arc<ExprNode> {
    assert_eq!(pair.as_rule(), Rule::expr_nlr);
    let pair = pair.into_inner().next().unwrap();
    match pair.as_rule() {
        Rule::expr_lit => parse_expr_lit(pair, src),
        Rule::expr_var => parse_expr_var(pair, src),
        Rule::expr_let => parse_expr_let(pair, src),
        Rule::expr_if => parse_expr_if(pair, src),
        Rule::expr_lam => parse_expr_lam(pair, src),
        Rule::expr_braced => parse_bracket_expr(pair, src),
        _ => unreachable!(),
    }
}

fn parse_expr_var(pair: Pair<Rule>, src: &Arc<String>) -> Arc<ExprNode> {
    assert_eq!(pair.as_rule(), Rule::expr_var);
    let span = Span::from_pair(&src, &pair);
    let mut pairs = pair.into_inner();
    let names = parse_namespace(pairs.next().unwrap(), src);
    let var = pairs.next().unwrap().as_str().to_string();
    let ns = if names.len() > 0 {
        Some(NameSpace::new(names))
    } else {
        None
    };
    expr_var(&var, ns, Some(span))
}

fn parse_namespace(pair: Pair<Rule>, src: &Arc<String>) -> Vec<String> {
    assert_eq!(pair.as_rule(), Rule::namespace);
    let pairs = pair.into_inner();
    let mut ret: Vec<String> = Vec::new();
    for pair in pairs {
        ret.push(pair.as_str().to_string());
    }
    ret
}

fn parse_expr_lit(expr: Pair<Rule>, src: &Arc<String>) -> Arc<ExprNode> {
    let pair = expr.into_inner().next().unwrap();
    match pair.as_rule() {
        Rule::expr_int_lit => parse_expr_int_lit(pair, src),
        Rule::expr_bool_lit => parse_expr_bool_lit(pair, src),
        _ => unreachable!(),
    }
}

fn parse_expr_let(expr: Pair<Rule>, src: &Arc<String>) -> Arc<ExprNode> {
    let span = Span::from_pair(&src, &expr);
    let mut pairs = expr.into_inner();
    let var = pairs.next().unwrap();
    let _eq_of_let = pairs.next().unwrap();
    let bound = pairs.next().unwrap();
    let _in_of_let = pairs.next().unwrap();
    let val = pairs.next().unwrap();
    expr_let(
        parse_var_with_type(var, src),
        parse_expr(bound, src),
        parse_expr(val, src),
        Some(span),
    )
}

fn parse_var_with_type(pair: Pair<Rule>, src: &Arc<String>) -> Arc<Var> {
    assert_eq!(pair.as_rule(), Rule::var_with_type);
    let span = Span::from_pair(&src, &pair);
    let mut pairs = pair.into_inner();
    let var_name = pairs.next().unwrap().as_str();
    let scm = pairs.next().map(|ty| {
        let ty = parse_type(ty);
        Scheme::generalize(ty.free_vars(), vec![], ty)
    });
    var_local(var_name, scm, Some(span))
}

fn parse_var(pair: Pair<Rule>, src: &Arc<String>) -> Arc<Var> {
    assert_eq!(pair.as_rule(), Rule::var);
    var_local(pair.as_str(), None, Some(Span::from_pair(&src, &pair)))
}

fn parse_expr_lam(expr: Pair<Rule>, src: &Arc<String>) -> Arc<ExprNode> {
    let span = Span::from_pair(&src, &expr);
    let mut pairs = expr.into_inner();
    let var = pairs.next().unwrap();
    let _arrow_of_lam = pairs.next().unwrap();
    let val = pairs.next().unwrap();
    expr_abs(parse_var(var, src), parse_expr(val, src), Some(span))
}

fn parse_expr_if(expr: Pair<Rule>, src: &Arc<String>) -> Arc<ExprNode> {
    let span = Span::from_pair(&src, &expr);
    let mut pairs = expr.into_inner();
    let cond = pairs.next().unwrap();
    let then_val = pairs.next().unwrap();
    let else_val = pairs.next().unwrap();
    expr_if(
        parse_expr(cond, src),
        parse_expr(then_val, src),
        parse_expr(else_val, src),
        Some(span),
    )
}

fn parse_bracket_expr(expr: Pair<Rule>, src: &Arc<String>) -> Arc<ExprNode> {
    let span = Span::from_pair(&src, &expr);
    let inner = expr.into_inner().next().unwrap();
    parse_expr(inner, src).set_source(Some(span))
}

fn parse_expr_int_lit(expr: Pair<Rule>, src: &Arc<String>) -> Arc<ExprNode> {
    let span = Span::from_pair(&src, &expr);
    let val = expr.as_str().parse::<i64>().unwrap();
    int(val, Some(span))
}

fn parse_expr_bool_lit(expr: Pair<Rule>, src: &Arc<String>) -> Arc<ExprNode> {
    let val = expr.as_str().parse::<bool>().unwrap();
    let span = Span::from_pair(&src, &expr);
    bool(val, Some(span))
}

fn parse_type(type_expr: Pair<Rule>) -> Arc<TypeNode> {
    assert_eq!(type_expr.as_rule(), Rule::type_expr);
    let mut pairs = type_expr.into_inner();
    let pair = pairs.next().unwrap();
    match pair.as_rule() {
        Rule::type_fun => parse_type_fun(pair),
        _ => unreachable!(),
    }
}

fn parse_type_fun(type_expr: Pair<Rule>) -> Arc<TypeNode> {
    assert_eq!(type_expr.as_rule(), Rule::type_fun);
    let mut pairs = type_expr.into_inner();
    let src_ty = parse_type_tyapp(pairs.next().unwrap());
    match pairs.next() {
        Some(pair) => {
            let dst_ty = parse_type(pair);
            type_fun(src_ty, dst_ty)
        }
        None => src_ty,
    }
}

fn parse_type_tyapp(type_expr: Pair<Rule>) -> Arc<TypeNode> {
    assert_eq!(type_expr.as_rule(), Rule::type_tyapp);
    let mut pairs = type_expr.into_inner();
    let mut pair = pairs.next().unwrap();
    let mut ret = parse_type_nlr(pair);
    for pair in pairs {
        ret = type_tyapp(ret, parse_type(pair));
    }
    ret
}

fn parse_type_nlr(type_expr: Pair<Rule>) -> Arc<TypeNode> {
    assert_eq!(type_expr.as_rule(), Rule::type_nlr);
    let mut pairs = type_expr.into_inner();
    let pair = pairs.next().unwrap();
    match pair.as_rule() {
        Rule::type_tycon => parse_type_tycon(pair),
        Rule::type_braced => parse_type_braced(pair),
        _ => unreachable!(),
    }
}

fn parse_type_tycon(type_expr: Pair<Rule>) -> Arc<TypeNode> {
    assert_eq!(type_expr.as_rule(), Rule::type_tycon);
    type_tycon(&tycon(type_expr.as_str()))
}

fn parse_type_braced(type_expr: Pair<Rule>) -> Arc<TypeNode> {
    assert_eq!(type_expr.as_rule(), Rule::type_braced);
    let mut pairs = type_expr.into_inner();
    let pair = pairs.next().unwrap();
    parse_type(pair)
}

fn rule_to_string(r: &Rule) -> String {
    match r {
        Rule::EOI => "end-of-input".to_string(),
        Rule::expr_int_lit => "integer".to_string(),
        Rule::expr_bool_lit => "boolean".to_string(),
        Rule::expr_nlr => "expression".to_string(),
        Rule::var => "variable".to_string(),
        Rule::in_of_let => "`in` or `;`".to_string(),
        Rule::eq_of_let => "`=`".to_string(),
        Rule::type_expr => "type".to_string(),
        Rule::arrow_of_lam => "`->`".to_string(),
        _ => format!("{:?}", r),
    }
}

fn message_parse_error(e: Error<Rule>) -> String {
    let mut msg: String = Default::default();

    // Show error content.
    msg += "parse error: expected ";
    match &e.variant {
        pest::error::ErrorVariant::ParsingError {
            positives,
            negatives,
        } => {
            fn concat_words(words: Vec<String>, sep: &str) -> String {
                let mut msg = String::from("");
                for (i, word) in words.iter().enumerate() {
                    let i = i as i32;
                    msg += word;
                    if i <= words.len() as i32 - 2 {
                        msg += &format!(" {} ", sep);
                    }
                }
                msg
            }
            if positives.len() > 0 {
                let words: Vec<String> = positives.iter().map(rule_to_string).collect();
                msg += &concat_words(words, "or");
                if negatives.len() > 0 {
                    msg += " and ";
                }
            }
            if negatives.len() > 0 {
                msg += "neither ";
                let words: Vec<String> = negatives.iter().map(rule_to_string).collect();
                msg += &concat_words(words, "nor");
            }
        }
        pest::error::ErrorVariant::CustomError { message: _ } => unreachable!(),
    };
    msg += "\n";

    // Show line and column number.
    // TODO: Show filename here.
    let (line, col) = match e.line_col {
        pest::error::LineColLocation::Pos(s) => s,
        pest::error::LineColLocation::Span(s, _) => s,
    };
    msg += &format!("at {}:{}", line, col);
    msg += "\n";

    // Show source code.
    let linenum_str = line.to_string();
    let linnum_chars = linenum_str.len();
    msg += &(" ".repeat(linnum_chars) + " | " + "\n");
    msg += &(linenum_str.clone() + " | ");
    msg += e.line();
    msg += "\n";
    msg += &(" ".repeat(linnum_chars) + " | ");
    msg += &(" ".repeat(col - 1) + "^");
    msg += "\n";
    msg
}
