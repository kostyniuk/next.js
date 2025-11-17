use proc_macro2::TokenTree;
use quote::ToTokens;
use syn::visit::{Visit, visit_block, visit_expr, visit_item, visit_macro};

pub fn is_self_used(block: &syn::Block) -> bool {
    let mut finder = SelfFinder { found: false };
    finder.visit_block(block);
    finder.found
}

struct SelfFinder {
    found: bool,
}

impl Visit<'_> for SelfFinder {
    fn visit_block(&mut self, n: &syn::Block) {
        if self.found {
            return;
        }

        visit_block(self, n);
    }

    fn visit_expr(&mut self, expr: &syn::Expr) {
        if self.found {
            return;
        }

        if let syn::Expr::Path(path) = expr
            && path.path.is_ident("self")
        {
            self.found = true;
            return;
        }

        visit_expr(self, expr);
    }

    fn visit_item(&mut self, n: &syn::Item) {
        if self.found {
            return;
        }

        visit_item(self, n);
    }

    fn visit_item_impl(&mut self, _: &syn::ItemImpl) {
        // skip children of `impl`: the definition of "self" inside of an impl is different than the
        // parent scope's definition of "self"
    }

    fn visit_macro(&mut self, mac: &syn::Macro) {
        if self.found {
            return;
        }

        for token in mac.tokens.to_token_stream() {
            if contains_self_token(&token) {
                self.found = true;
                return;
            }
        }

        visit_macro(self, mac);
    }
}

fn contains_self_token(tok: &TokenTree) -> bool {
    match tok {
        TokenTree::Group(group) => {
            for token in group.stream() {
                if contains_self_token(&token) {
                    return true;
                }
            }
            false
        }
        TokenTree::Ident(ident) => ident == "self",
        TokenTree::Punct(..) | TokenTree::Literal(..) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_self_used() {
        let test_cases = vec![
            (
                "no self usage",
                "{ let x = 42; println!(\"hello\"); }",
                false,
            ),
            ("simple self usage", "{ self.foo(); }", true),
            ("self field access", "{ let x = self.field; }", true),
            (
                "self in nested block",
                "{ let x = 1; { self.method(); } }",
                true,
            ),
            (
                "self in impl block not detected",
                "{ impl Foo { fn bar(&self) { self.baz(); } } }",
                false,
            ),
            (
                "self before impl block",
                "{ self.foo(); impl Bar { fn baz(&self) { self.qux(); } } }",
                true,
            ),
            (
                "self in closure",
                "{ let f = || { self.method(); }; }",
                true,
            ),
            (
                "self in if condition",
                "{ if self.check() { println!(\"true\"); } }",
                true,
            ),
            (
                "self in match arm",
                "{ match x { Some(_) => self.handle(), None => {}, } }",
                true,
            ),
            ("self in macro", "{ println!(\"{:?}\", self); }", true),
            (
                "self in complex macro",
                "{ format!(\"value: {}\", self.field); }",
                true,
            ),
            (
                "no self with similar idents",
                "{ let myself = 42; let selfish = true; }",
                false,
            ),
            ("empty block", "{}", false),
            ("self in return statement", "{ return self.value; }", true),
            (
                "self as function argument",
                "{ some_function(self); }",
                true,
            ),
        ];

        for (description, code, expected) in test_cases {
            let block: syn::Block = syn::parse_str(code)
                .unwrap_or_else(|e| panic!("Failed to parse block for '{}': {}", description, e));
            let result = is_self_used(&block);
            assert_eq!(
                result, expected,
                "Test case '{}' failed: expected {}, got {}",
                description, expected, result
            );
        }
    }
}
