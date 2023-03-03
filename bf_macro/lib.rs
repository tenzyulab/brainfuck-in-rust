use quote::quote;

use proc_macro::{Delimiter, TokenStream, TokenTree as TT};
use proc_macro2::TokenStream as TokenStream2;

enum Oper {
    IncrementPointer,
    DecrementPointer,
    Increment,
    Decrement,
    Write,
    Read,
    Loop(Vec<Oper>),
}

struct BrainFuck {
    inner: Vec<Oper>,
}

impl BrainFuck {
    fn parse(tokens: TokenStream) -> Self {
        fn _inner(tokens: TokenStream) -> Vec<Oper> {
            tokens
                .into_iter()
                .map(|tt| match tt {
                    TT::Group(group) if group.delimiter() == Delimiter::Bracket => {
                        Oper::Loop(_inner(group.stream()))
                    }
                    TT::Punct(punct) => match punct.as_char() {
                        '>' => Oper::IncrementPointer,
                        '<' => Oper::DecrementPointer,
                        '+' => Oper::Increment,
                        '-' => Oper::Decrement,
                        '.' => Oper::Write,
                        ',' => Oper::Read,
                        _ => panic!("Invalid character"),
                    },
                    _ => panic!("Invalid character"),
                })
                .collect()
        }

        Self {
            inner: _inner(tokens),
        }
    }

    fn into_tokens(self) -> TokenStream {
        let mut program = quote! {
            let mut pointer = 0;
            let mut buf = vec![0_u8; 1024];
            let mut stdin = ::std::io::stdin();
            let mut stdout = ::std::io::stdout();
        };

        fn process_ops(ops: Vec<Oper>) -> TokenStream2 {
            ops.into_iter() // Iterator<Item=Operation>
                .map(|op| match op {
                    Oper::IncrementPointer => quote! { pointer = pointer.wrapping_add(1); },
                    Oper::DecrementPointer => quote! { pointer = pointer.wrapping_sub(1); },
                    Oper::Increment => quote! { buf[pointer] = buf[pointer].wrapping_add(1); },
                    Oper::Decrement => quote! { buf[pointer] = buf[pointer].wrapping_sub(1); },
                    Oper::Write => quote! {
                        ::std::io::Write::write(
                            &mut stdout,
                            ::std::slice::from_ref(&buf[pointer]),
                        ).expect("failed to write");
                    },
                    Oper::Read => quote! {
                        ::std::io::Read::read(
                            &mut stdin,
                            ::std::slice::from_mut(&mut buf[pointer]),
                        ).expect("failed to read");
                    },
                    Oper::Loop(ops) => {
                        let ops = process_ops(ops);
                        quote! {
                            while buf[pointer] != 0 {
                                #ops
                            };
                        }
                    }
                })
                .collect()
        }

        program.extend(process_ops(self.inner));
        quote! {
            { #program }
        }
        .into()
    }
}

#[proc_macro]
pub fn bf(tokens: TokenStream) -> TokenStream {
    BrainFuck::parse(tokens).into_tokens()
}
