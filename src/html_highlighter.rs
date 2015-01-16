#![feature(slicing_syntax)]
#![feature(macro_rules)]
#![feature(globs)]

extern crate syntax;

use syntax::parse::new_parse_sess;
use syntax::parse::parser::Parser;
use syntax::diagnostic::ColorConfig;
use syntax::parse::lexer::StringReader;
use syntax::codemap::CodeMap;
use syntax::ext::quote::rt::ToSource;
use syntax::codemap::Spanned;
use syntax::ast::*;
use syntax::ast;

use std::fmt;

pub type Box<T> = syntax::ptr::P<T>;

trait ToHtml<Style> where Style : ToCss {
    fn to_html(&self, f : &mut fmt::Formatter, original : &str, style : Style) -> fmt::Result {
        let mut span_buff = Vec::new();
        self.add_span_tokens(&mut span_buff);

        restore_whitespace(original, &mut span_buff);

        write!(f, "<pre style=\"background:#efffef\"><code>");

        for token in span_buff.iter() {
            write!(f, "<span {}>{}</span>", style.clone().to_css(&token.class), token.text);
        }     

        write!(f, "</code></pre>");

        Ok(())
    }

    fn add_span_tokens(&self, tokens : &mut Vec<SpanToken>);
}

pub trait ToCss : Clone {
    fn to_css(self, clazz : &Class) -> String;
}

#[deriving(Show)]
pub enum Class {
    //If nothing else
    Default,

    //General 
    PreludeType,
    Type,
    Keyword,
    Comment,
    Operator,
    Attribute,

    //Literals
    NumberLit,
    StringLit,
    BoolLit,

    //Contextual
    Param,
    MacParam,
    CrateDecl,
    UseDecl,
    VarDecl,
    FnDecl,
    MacDecl,
    TyDecl,
    FieldDecl,
    VariantDecl,
    LifeTimeDecl,
    GenDecl,
    GenConstraint,
    Gen,
    LifeTime,
    FnCall,
    ReturnStmt, //for implicit and explicit returns...darker orange?
    MacCall
}

#[deriving(Show)]
struct SpanToken {
    class : Class,
    text  : String
}

impl SpanToken {
    fn new(class : Class, text : String) -> SpanToken {
        SpanToken {
            class : class,
            text  : text
        }
    }
}

struct Highlighted<'a, Markup : ToHtml<Style>, Style : ToCss>(&'a str, Markup, Style);

impl<'a, Markup, Style> fmt::Show for Highlighted<'a, Markup, Style> 
where Markup : ToHtml<Style>, Style : ToCss {

    fn fmt(&self, f : &mut fmt::Formatter) -> fmt::Result {
        let &Highlighted(src, ref markup, ref style) = self;
        
        markup.to_html(f, src, style.clone())
    }
}

//TODO:
//implement ToHtml for Decl(ast::Type) { ... }
//so then you could change the tohtml characteristics 
struct Declaration<T>(T); 

///////////////////////////////////////////////////////////////////////////////
//                                                                           //
//                               Convenience Macros                          //                                      
//                                                                           //
///////////////////////////////////////////////////////////////////////////////

macro_rules! add_token{
    ($vec:ident, $class:ident, $string:expr) => (
        {
            $vec.push(tok!($class, $string));
        }
    );
}

macro_rules! tok{
    ($class:ident, $string:expr) => (
        {
            SpanToken::new(Class::$class, $string.to_string())
        }
    );
}

macro_rules! add_spans{
    ($enchanted_forest:expr, $fairy_dust:ident) => (
        {
            for fairy in $enchanted_forest {
                let tinkerbell = fairy as &ToHtml<Style>;
                tinkerbell.add_span_tokens($fairy_dust);
            }
        }
    );
}

macro_rules! between_each {
    ($thing:pat in $things:expr $outter_stmts:block => $inbetween_stmts:block) 
        => 
    (
        {
            let len = $things.len();
            if len != 0 {
                let mut parts = $things.iter();

                match parts.next() {
                    Some($thing) => $outter_stmts,
                    None   => panic!("called between_each! on an empty container!")
                };

                for $thing in parts.take(len - 2) {
                    $inbetween_stmts
                    $outter_stmts
                }

                match parts.last() {
                    Some($thing) => {
                        $inbetween_stmts
                        $outter_stmts
                    },
                    None   => {}
                };
            }
        }
    );
}

///////////////////////////////////////////////////////////////////////////////
//                                                                           //
//                               Entry point module                          //                                      
//                                                                           //
///////////////////////////////////////////////////////////////////////////////

pub mod highlight {
    use super::{ToCss, Highlighted , str_to_crate/*, str_to_expr, str_to_stmt, str_to_item, str_to_view_item*/};

    pub fn program<Style : ToCss>(src : &str, style : Style) -> String {
        let crayt = str_to_crate(src);
        format!("{}", Highlighted(src, crayt, style))
    }

    // fn expr<Style : ToCss>(src : &str, style : Style) -> String {
    //     let expr = str_to_expr(src);
    //     format!("{}", Highlighted(src, expr, style))
    // }

    // fn stmt<Style : ToCss>(src : &str, style : Style) -> String {
    //     let stmt = str_to_stmt(src);
    //     format!("{}", Highlighted(src, stmt, style))
    // }

    // fn item<Style : ToCss>(src : &str, style : Style) -> String {
    //     let item = str_to_item(src);
    //     format!("{}", Highlighted(src, item, style))
    // }

    // fn view_item<Style : ToCss>(src : &str, style : Style) -> String {
    //     let item = str_to_view_item(src);
    //     format!("{}", Highlighted(src, item, style))
    // }
}

///////////////////////////////////////////////////////////////////////////////
//                                                                           //
//                   Unit type implementations of ToCss                      //                                      
//                                                                           //
///////////////////////////////////////////////////////////////////////////////

#[deriving(Clone)]
struct StyleSheet;
impl ToCss for StyleSheet {
    fn to_css(self, clazz : &Class) -> String {
        match *clazz {
            Class::Type        => "class=\".type\"",
            Class::PreludeType => "class=\".prelude-type\"", 
            _                  => "class=\".default\""
        }.to_string()
    }
}

#[deriving(Clone)]
struct DefaultInline;
impl ToCss for DefaultInline {
    fn to_css(self, clazz : &Class) -> String {
        match *clazz {
            Class::Keyword     => "style=\"color:#ea4646;font-weight:bold\"",//"style=\"color:#07c7dd\"",
            Class::Operator    => "style=\"color:#07c7dd\"",
            Class::CrateDecl   => "style=\"color:#f4824e\"",
            Class::UseDecl | Class::GenDecl | Class::TyDecl    => "style=\"color:#f4824e\"",
            Class::LifeTimeDecl => "style=\"color:#ff4242\"",
            Class::Attribute   => "style=\"color:#7f7f7f\"",

            Class::Type        => "style=\"color:#000000\"",
            Class::PreludeType => "style=\"color:#00ff0b\"",
            _                     => "style=\"\""
        }.to_string()
    }
}

///////////////////////////////////////////////////////////////////////////////
//                                                                           //
//                   AST type implementations of ToHtml                      //                                      
//                                                                           //
///////////////////////////////////////////////////////////////////////////////

impl<Style> ToHtml<Style> for ast::Crate where Style : ToCss {
    fn add_span_tokens(&self, tokens : &mut Vec<SpanToken>) {

        add_spans!(self.module.view_items.iter(), tokens);

        add_spans!(self.module.items.iter(), tokens);
    }
}

impl<Style> ToHtml<Style> for ast::ViewItem where Style : ToCss {
    fn add_span_tokens(&self, tokens : &mut Vec<SpanToken>) {

        add_spans!(self.attrs.iter(), tokens);

        let vis = &self.vis as &ToHtml<Style>;
        vis.add_span_tokens(tokens);

        match self.node {
            ViewItemExternCrate(ref name, _, _) => {
                add_token!(tokens, Keyword, "extern");
                add_token!(tokens, Keyword, "crate");
                add_token!(tokens, CrateDecl, name.as_str());
                add_token!(tokens, Operator, ";");
            },
            ViewItemUse(ref path) => {

                add_token!(tokens, Keyword, "use");

                match path.node {
                    //use a::b::c; and use a::b::c as d;
                    ViewPathSimple(ref name, ref path, _) => {
                        let mut segments = path.segments.iter();

                        match segments.next() {
                            Some(segment) => {
                                if path.global {
                                    add_token!(tokens, Operator, "::");
                                }

                                add_token!(tokens, Default, segment.identifier.as_str());
                            },
                            None => { panic!("this should never happen") }
                        };

                        for segment in segments.take(path.segments.len() - 2) {
                            add_token!(tokens, Operator, "::");
                            add_token!(tokens, Default, segment.identifier.as_str());
                        }

                        add_token!(tokens, Operator, "::");

                        match segments.last() {
                            Some(segment) => {
                                if false/* TODO: it has an 'as' stmt */ {
                                    add_token!(tokens, Default, segment.identifier.as_str());
                                    add_token!(tokens, Keyword, "as");
                                    add_token!(tokens, UseDecl, name.as_str());
                                } else {

                                    add_token!(tokens, UseDecl, segment.identifier.as_str());
                                }
                            },
                            None => { panic!("this should never happen") }
                        };
                    },
                    ViewPathGlob(ref path, _) => {
                        let mut segments = path.segments.iter();

                        match segments.next() {
                            Some(segment) => {
                                if path.global {
                                    add_token!(tokens, Operator, "::");
                                }

                                add_token!(tokens, Default, segment.identifier.as_str());
                            },
                            None => { panic!("this should never happen") }
                        };

                        for segment in segments.take(path.segments.len() - 1) {
                            add_token!(tokens, Operator, "::");
                            add_token!(tokens, Default, segment.identifier.as_str());
                        }

                        add_token!(tokens, Operator, "::");
                        add_token!(tokens, UseDecl, "*");
                    },
                    ViewPathList(ref path, ref imports, _) => {
                        let mut segments = path.segments.iter();

                        match segments.next() {
                            Some(segment) => {
                                if path.global {
                                    add_token!(tokens, Operator, "::");
                                }

                                add_token!(tokens, Default, segment.identifier.as_str());
                            },
                            None => { panic!("this should never happen") }
                        };

                        for segment in segments.take(path.segments.len() - 2) {
                            add_token!(tokens, Operator, "::");
                            add_token!(tokens, Default, segment.identifier.as_str());
                        }

                        add_token!(tokens, Operator, "::");
                        add_token!(tokens, Operator, "{");

                        let len = imports.len() - 1;
                        for import in imports.iter().take(len) {
                            match import.node {
                                PathListIdent{ name , .. } => {
                                    add_token!(tokens, UseDecl, name.as_str());
                                    add_token!(tokens, Operator, ",");
                                },
                                _ => panic!("this should never happen")
                            };
                        }

                        let spanned_import = imports[len];

                        match spanned_import.node {
                            PathListIdent{ name, .. } => {
                                add_token!(tokens, UseDecl, name.as_str());
                            },
                            _ => { panic!("this should never happen") }
                        };

                        
                        add_token!(tokens, Operator, "}");
                    }
                };

                add_token!(tokens, Operator, ";");
            }
        };
    }
}

impl<Style> ToHtml<Style> for ast::Attribute where Style : ToCss {
    fn add_span_tokens(&self, tokens : &mut Vec<SpanToken>) {
        tokens.push(tok!(Attribute, self.node.to_source()));
    }
}

impl<Style> ToHtml<Style> for ast::Visibility where Style : ToCss {
    fn add_span_tokens(&self, tokens : &mut Vec<SpanToken>) {
        match *self {
            ast::Visibility::Public => {
                tokens.push(tok!(Keyword, "pub"));
            },
            ast::Visibility::Inherited => {}
        };
    }
}

impl<Style> ToHtml<Style> for Box<ast::Item> where Style : ToCss {
    fn add_span_tokens(&self, tokens : &mut Vec<SpanToken>) {
        add_spans!(self.attrs.iter(), tokens);

        let vis = &self.vis as &ToHtml<Style>;
        vis.add_span_tokens(tokens);

        match self.node {
            ItemStatic(ref ty, ref mutability, ref expr) => {

            },
            ItemConst(ref ty, ref expr) => {

            },
            ItemFn(ref fn_decl, ref unsafety, ref abi, ref generics, ref block) => {

            },
            ItemMod(ref module) => {

            },
            ItemForeignMod(ref foreign_mod) => {

            },
            ItemTy(ref ty, ref generics) => {
                println!("self: {}\n\nty: {}\n\ngenerics: {}", self, ty, generics);

                add_token!(tokens, Keyword, "type");
                add_token!(tokens, TyDecl, self.ident.name.as_str());

                let gen = &Declaration(generics) as &ToHtml<Style>;
                gen.add_span_tokens(tokens);

                add_token!(tokens, Operator, "=");

                let typ = ty as &ToHtml<Style>;
                typ.add_span_tokens(tokens);

                add_token!(tokens, Operator, ";");
            },
            ItemEnum(ref enum_def, ref generics) => {

            },
            ItemStruct(ref struct_def, ref generics) => {

            },
            ItemTrait(ref unsafety, ref generics, ref ty_param_bounds, ref trait_items) => {

            },
            ItemImpl(ref unsafety, ref generics, ref trait_ref, ref ty, ref  impls) => {

            },
            ItemMac(ref mac) => {

            }
        }
    }
}

impl<Style> ToHtml<Style> for ast::Generics where Style : ToCss {
    fn add_span_tokens(&self, tokens : &mut Vec<SpanToken>) {

        if self.lifetimes.len() == 0 && self.ty_params.len() == 0 && self.where_clause.predicates.len() == 0 {
            return
        }


        add_token!(tokens, Operator, "<");

        between_each!(lifetime in self.lifetimes {
            add_token!(tokens, LifeTime, lifetime.lifetime.name.as_str());
        } => {
            add_token!(tokens, Operator, ",");            
        });

        if self.ty_params.len() > 0 && self.lifetimes.len() > 0 {
            add_token!(tokens, Operator, ",");            
        }

        between_each!(ty_param in self.ty_params.as_slice() {
            add_token!(tokens, Gen, ty_param.ident.name.as_str());
        } => {
            add_token!(tokens, Operator, ",");            
        });

        add_token!(tokens, Operator, ">");

        // if self.where_clause.predicates.len() > 0 {
        //     add_token!(tokens, Keyword, "where");
        // }

        // for predicate in self.where_clause.predicates.iter() {
        //     //do this later 
        // }
    }
}

impl<'a, Style> ToHtml<Style> for Declaration<&'a ast::Generics> where Style : ToCss {
    fn add_span_tokens(&self, tokens : &mut Vec<SpanToken>) {

        let &Declaration(ref _self) = self;

        if _self.lifetimes.len() == 0 && _self.ty_params.len() == 0 && _self.where_clause.predicates.len() == 0 {
            return
        }


        add_token!(tokens, Operator, "<");

        between_each!(lifetime in _self.lifetimes {
            add_token!(tokens, LifeTimeDecl, lifetime.lifetime.name.as_str());
        } => {
            add_token!(tokens, Operator, ",");            
        });

        if _self.ty_params.len() > 0 && _self.lifetimes.len() > 0 {
            add_token!(tokens, Operator, ",");            
        }

        between_each!(ty_param in _self.ty_params.as_slice() {
            add_token!(tokens, GenDecl, ty_param.ident.name.as_str());
        } => {
            add_token!(tokens, Operator, ",");            
        });

        add_token!(tokens, Operator, ">");

        // if _self.where_clause.predicates.len() > 0 {
        //     add_token!(tokens, Keyword, "where");
        // }

        // for predicate in _self.where_clause.predicates.iter() {
        //     //do this later 
        // }
    }
}

impl<Style> ToHtml<Style> for Box<ast::Ty> where Style : ToCss {
    fn add_span_tokens(&self, tokens : &mut Vec<SpanToken>) {
        match self.node {
            TyVec(ref ty) => {
                add_token!(tokens, Operator, "[");

                let ty = ty as &ToHtml<Style>;
                ty.add_span_tokens(tokens);

                add_token!(tokens, Operator, "]");
            },
            TyFixedLengthVec(ref ty, ref expr) => {
                add_token!(tokens, Operator, "[");

                let ty = ty as &ToHtml<Style>;
                ty.add_span_tokens(tokens);

                //TODO:...this should be a ; with the new grammar change
                add_token!(tokens, Operator, ",");
                add_token!(tokens, Operator, "..");

                //TODO:...make this a real call on the expr
                add_token!(tokens, NumberLit, "4");

                add_token!(tokens, Operator, "]");

            },
            TyPtr(ref mut_ty) => {

            },
            TyRptr(ref lifetime, ref mut_ty) => {

            },
            TyClosure(ref closure_ty) => {

            },
            TyBareFn(ref bare_fn_ty) => {

            },
            TyTup(ref types) => {

            },
            TyPath(ref path, _) => {
                if path.global {
                    add_token!(tokens, Operator, "::");
                }

                let len = path.segments.len();
                let segments = path.segments.iter();
                for segment in segments.take(len - 1) {
                    add_token!(tokens, Default, segment.identifier.as_str());
                    add_token!(tokens, Operator, "::");
                }

                let final_type = segments.last().unwrap();
                add_token!(tokens, Type, final_type.identifier.as_str());

                match final_type.parameters {
                    AngleBracketedParameters(ref data) => {
                        let len = data.types.len();

                        if len == 0 {
                            return;
                        }

                        add_token!(tokens, Operator, "<");
                        
                        let types = data.types.iter();
                        for ty in types.take(len - 1) {
                            let ty = ty as &ToHtml<Style>;
                            ty.add_span_tokens(tokens);
                            add_token!(tokens, Operator, ",");
                        }

                        match types.last() {
                            Some(ty) => {
                                let ty = ty as &ToHtml<Style>;
                                ty.add_span_tokens(tokens);
                            },
                            None => {}
                        }

                        add_token!(tokens, Operator, ">");
                    },
                    _ => { /*TODO: handle the other case*/ }
                }
            },
            TyObjectSum(ref ty, ref ty_param_bounds) => {

            },
            TyPolyTraitRef(ref ty_param_bounds) => {

            },
            TyQPath(ref qpath) => {

            },
            TyParen(ref ty) => {

            },
            TyTypeof(ref expr) => {

            },
            TyInfer => {

            },
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
//                                                                           //
//                   User src to ast type functions                          //                                      
//                                                                           //
///////////////////////////////////////////////////////////////////////////////

pub fn str_to_crate(src : &str) -> ast::Crate {
    // let mut parser = parser_from_str(src);

    let name = "".to_string();
    
    let handler    = syntax::diagnostic::default_handler(ColorConfig::Always, None);
    let codemap    = syntax::codemap::CodeMap::new();
    let filemap    = codemap.new_filemap(name, src.to_string());
    let sp_handler = syntax::diagnostic::mk_span_handler(handler, codemap);
    let reader     = StringReader::new(&sp_handler, filemap);

    let cfg = Vec::new();
    let sess = new_parse_sess();

    Parser::new(&sess, cfg, box reader).parse_crate_mod()

    // parser.parse_crate_mod()
}

// pub fn str_to_expr(src : &str) -> Box<ast::Expr> {
//     let mut parser = parser_from_str(src);
//     parser.parse_expr()
// }

// pub fn str_to_stmt(src : &str) -> Box<ast::Stmt> {
//     let mut parser = parser_from_str(src);
//     parser.parse_stmt(Vec::new())
// }

// pub fn str_to_item(src : &str) -> Box<ast::Item> {
//     let mut parser = parser_from_str(src);
//     parser.parse_item_with_outer_attributes().expect(
//         "Tried to parse a source that is not an item as an item."
//     )
// }

// pub fn str_to_view_item(src : &str) -> ast::ViewItem {
//     let mut parser = parser_from_str(src);
//     parser.parse_view_item(Vec::new())
// }

// pub fn parser_from_str(src : &str) -> Parser {
//     let name = "".to_string();
    
//     let handler    = syntax::diagnostic::default_handler(ColorConfig::Always, None);
//     let codemap    = syntax::codemap::CodeMap::new();
//     let filemap    = codemap.new_filemap(name, src.to_string());
//     let sp_handler = syntax::diagnostic::mk_span_handler(handler, codemap);
//     let reader     = StringReader::new(&sp_handler, filemap);

//     let cfg = Vec::new();
//     let sess = new_parse_sess();

//     Parser::new(&sess, cfg, box reader)
// }

///////////////////////////////////////////////////////////////////////////////
//                                                                           //
//                   Whitespace restoration in tokens                        //                                      
//                                                                           //
///////////////////////////////////////////////////////////////////////////////

fn restore_whitespace(original : &str, tokens : &mut Vec<SpanToken>) {
    let mut original = original;

    for token in tokens.iter_mut() {
        let index = match original.find_str(token.text[]) {
            Some(i) => i,
            None    => panic!("When trying to restore whitespace, the token {} wasn't found in the original text", token)
        };
        
        let text_len = token.text.len();
        
        let mut whitespace = original[0..index].to_string();
        whitespace.push_str(token.text[]);
        token.text = whitespace;
        
        original = original[(index + text_len)..];
    }
}

fn main() {
    let src = "
    #[phase(plugin)]
    extern crate highlightrs;

    use highlightrs::{string_to_crate, string_to_item, string_to_view_item};
    use highlightrs::style::themes::Monokai;
    use highlightrs::highlight::*;
    use syntax::ptr;
    pub use highlightrs::Highlited;

    type Box<'l, T> = ptr::Ptr<'l, T>;
    type BorrowedBox<'a, 'b, 'c, T1, T2> = ptr::BorrowedPtr<'a, 'b, 'c, T1, T2>;
    type FourPtrs<T> = [Box<T>, ..4];
    type Ptrs = [Box<T>];

    ";

    println!("{}", ::highlight::program(src, DefaultInline));
}
