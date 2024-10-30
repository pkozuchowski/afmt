#[macro_export]
macro_rules! def_struct {
    ( $( $name:ident ),+ $(,)? ) => {
        $(
            pub struct $name<'a, 'tree> {
                inner: &'a Node<'tree>,
            }

            #[allow(dead_code)]
            impl<'a, 'tree> $name<'a, 'tree> {
                pub fn node(&self) -> &'a Node<'tree> {
                    self.inner
                }

                pub fn prepare<'b>(
                    &self,
                    context: &'b FmtContext,
                ) -> (&'a Node<'tree>, String, &'b str, &'b Config) {
                    let node = self.node();
                    let result = String::new();
                    let source_code = &context.source_code;
                    (node, result, source_code, &context.config)
                }
            }

            impl<'a, 'tree> FromNode<'a, 'tree> for $name<'a, 'tree> {
                fn new(node: &'a Node<'tree>) -> Self {
                    Self { inner: node }
                }
            }
        )+
    };
}

#[macro_export]
macro_rules! match_routing {
    ( $node:ident, $context:ident, $shape:ident;
      $( $kind:literal => $struct_name:ident ),* $(,)? ) => {
        match $node.kind() {
            "line_comment" => {
                LineComment::new(&$node).rewrite($shape, $context)
            },
            "block_comment" => {
                BlockComment::new(&$node).rewrite($shape, $context)
            },
            $(
                $kind => {
                    $struct_name::new(&$node).rewrite($shape, $context)
                }
            )*
            _ => {
                let struct_name = std::any::type_name::<Self>().split("::").last().unwrap();
                panic!("## {} routing - unknown node: {} | {} || {}",
                struct_name.yellow(),
                $node.kind().red(),
                $node
                    .v(&$context.source_code)
                    .chars()
                    .take(100)
                    .collect::<String>(),
                $node.parent().map_or(String::new(), |parent_node| {
                    parent_node
                        .v(&$context.source_code)
                        .chars()
                        .take(100)
                        .collect::<String>()
                })
            );

            }
        }
    };
}

#[macro_export]
macro_rules! static_routing {
    ( $map:expr, $node:ident, $context:ident, $shape:ident ) => {
        if let Some(constructor) = $map.get($node.kind()) {
            let struct_instance: Box<dyn Rewrite> = constructor(&$node);
            struct_instance.rewrite($shape, $context)
        } else {
            let struct_name = std::any::type_name::<Self>().split("::").last().unwrap();
            panic!(
                "## {} routing - unknown node: {} | {} || {}",
                struct_name.yellow(),
                $node.kind().red(),
                $node
                    .v(&$context.source_code)
                    .chars()
                    .take(100)
                    .collect::<String>(),
                $node.parent().map_or(String::new(), |parent_node| {
                    parent_node
                        .v(&$context.source_code)
                        .chars()
                        .take(100)
                        .collect::<String>()
                })
            );
        }
    };
}
