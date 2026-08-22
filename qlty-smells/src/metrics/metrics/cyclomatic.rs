use qlty_analysis::code::{File, NodeFilter, Visitor};
use tree_sitter::Node;
use tree_sitter::TreeCursor;

pub fn count<'a>(source_file: &'a File, node: &Node<'a>, filter: &NodeFilter) -> usize {
    let mut processor = CyclomaticComplexity::new(source_file, filter);
    processor.process_node(&mut node.walk());
    processor.count
}

pub struct CyclomaticComplexity<'a> {
    pub count: usize,
    source_file: &'a File,
    filter: &'a NodeFilter,
}

impl<'a> CyclomaticComplexity<'a> {
    fn new(source_file: &'a File, filter: &'a NodeFilter) -> Self {
        Self {
            count: 1,
            source_file,
            filter,
        }
    }
}

impl Visitor for CyclomaticComplexity<'_> {
    fn source_file(&self) -> &File {
        self.source_file
    }

    fn skip_node(&self, node: &Node) -> bool {
        !node.is_named() || self.filter.exclude(node)
    }

    fn visit_if(&mut self, cursor: &mut TreeCursor) {
        self.count += 1;
        self.process_children(cursor);
    }

    fn visit_elsif(&mut self, cursor: &mut TreeCursor) {
        self.count += 1;
        self.process_children(cursor);
    }

    fn visit_conditional_assignment(&mut self, cursor: &mut TreeCursor) {
        self.count += 1;
        self.process_children(cursor);
    }

    fn visit_ternary(&mut self, cursor: &mut TreeCursor) {
        self.count += 1;
        self.process_children(cursor);
    }

    fn visit_case(&mut self, cursor: &mut TreeCursor) {
        self.count += 1;
        self.process_children(cursor);
    }

    fn visit_loop(&mut self, cursor: &mut TreeCursor) {
        self.count += 1;
        self.process_children(cursor);
    }

    fn visit_except(&mut self, cursor: &mut TreeCursor) {
        self.count += 1;
        self.process_children(cursor);
    }

    fn visit_try_expression(&mut self, cursor: &mut TreeCursor) {
        self.count += 1;
        self.process_children(cursor);
    }

    fn visit_binary(&mut self, cursor: &mut TreeCursor) {
        self.count += 1;
        self.process_children(cursor);
    }

    fn visit_call(&mut self, cursor: &mut TreeCursor) {
        let node = cursor.node();
        let (_, method_name) = self.language().call_identifiers(self.source_file, &node);

        if self
            .language()
            .iterator_method_identifiers()
            .contains(&method_name.as_str())
        {
            self.count += 1;
        }

        self.process_children(cursor);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    mod java {
        use super::*;

        #[test]
        fn cyclo_basic() {
            let source_file = File::from_string(
                "java",
                r#"
                public int foo() {
                    x = 1;
                }
            "#,
            );
            assert_eq!(
                1,
                count(
                    &source_file,
                    &source_file.parse().root_node(),
                    &NodeFilter::empty()
                )
            );
        }

        #[test]
        fn cyclo_if() {
            let source_file = File::from_string(
                "java",
                r#"
                public int foo() {
                    if(x) {
                        y = 1;
                    }
                }
            "#,
            );
            assert_eq!(
                2,
                count(
                    &source_file,
                    &source_file.parse().root_node(),
                    &NodeFilter::empty()
                )
            );
        }

        #[test]
        fn cyclo_if_else() {
            let source_file = File::from_string(
                "java",
                r#"
                public int foo() {
                    if(x) {
                        y = 1;
                    } else {
                        y = 2;
                    }
                }
            "#,
            );
            assert_eq!(
                2,
                count(
                    &source_file,
                    &source_file.parse().root_node(),
                    &NodeFilter::empty()
                )
            );
        }

        #[test]
        fn cyclo_if_elsif() {
            let source_file = File::from_string(
                "java",
                r#"
                public int foo() {
                    if(x) {
                        y = 1;
                    } else if(z) {
                        y = 2;
                    }
                }
            "#,
            );
            assert_eq!(
                3,
                count(
                    &source_file,
                    &source_file.parse().root_node(),
                    &NodeFilter::empty()
                )
            );
        }

        #[test]
        fn cyclo_if_elsif_else() {
            let source_file = File::from_string(
                "java",
                r#"
                public int foo() {
                    if(x) {
                        y = 1;
                    } else if(z) {
                        y = 2;
                    } else {
                        y = 3;
                    }
                }
            "#,
            );
            assert_eq!(
                3,
                count(
                    &source_file,
                    &source_file.parse().root_node(),
                    &NodeFilter::empty()
                )
            );
        }
    }

    mod elixir {
        use super::*;

        fn cyclomatic(source: &str) -> usize {
            let source_file = File::from_string("elixir", source);
            count(
                &source_file,
                &source_file.parse().root_node(),
                &NodeFilter::empty(),
            )
        }

        #[test]
        fn cyclo_counts_one_per_case_arm() {
            assert_eq!(
                3,
                cyclomatic(
                    "defmodule M do\n def f(x) do\n  case x do\n   1 -> :a\n   _ -> :b\n  end\n end\nend"
                )
            );
        }

        #[test]
        fn cyclo_ignores_matches_and_pipes() {
            assert_eq!(
                1,
                cyclomatic(
                    "defmodule M do\n def f(x) do\n  y = x |> String.trim() |> String.upcase()\n  y\n end\nend"
                )
            );
        }

        #[test]
        fn cyclo_counts_enum_iteration_calls() {
            assert_eq!(
                3,
                cyclomatic(
                    "defmodule M do\n def f(l) do\n  l\n  |> Enum.map(fn x -> x * 2 end)\n  |> Enum.filter(fn x -> x > 1 end)\n end\nend"
                )
            );
        }

        #[test]
        fn cyclo_counts_short_circuit_operators() {
            assert_eq!(
                2,
                cyclomatic("defmodule M do\n def f(a, b), do: a and b\nend")
            );
        }

        #[test]
        fn cyclo_ignores_anonymous_function_clauses() {
            assert_eq!(
                1,
                cyclomatic(
                    "defmodule M do\n def f(pid) do\n  Agent.update(pid, fn\n   nil -> 0\n   n -> n + 1\n  end)\n end\nend"
                )
            );
        }

        #[test]
        fn cyclo_ignores_typespec_whose_name_collides_with_an_iterator() {
            let with_spec =
                cyclomatic("defmodule M do\n @spec map(list) :: list\n def map(l), do: l\nend");
            let without_spec = cyclomatic("defmodule M do\n def map(l), do: l\nend");
            assert_eq!(without_spec, with_spec);
        }

        #[test]
        fn cyclo_ignores_callback_and_spec_declarations() {
            assert_eq!(
                1,
                cyclomatic(
                    "defmodule M do\n @callback filter(list) :: list\n @spec reduce(list) :: list\n def go(l), do: l\nend"
                )
            );
        }

        #[test]
        fn cyclo_ignores_attribute_name_that_collides_with_an_iterator() {
            assert_eq!(
                1,
                cyclomatic("defmodule M do\n @map %{a: 1}\n def go, do: @map\nend")
            );
        }

        #[test]
        fn cyclo_ignores_type_names_in_spec_argument_position() {
            let with_spec = cyclomatic(
                "defmodule M do\n @spec fetch(map()) :: term()\n def fetch(m), do: m\nend",
            );
            let without_spec = cyclomatic("defmodule M do\n def fetch(m), do: m\nend");
            assert_eq!(without_spec, with_spec);
        }

        #[test]
        fn cyclo_ignores_repeated_type_names_across_a_spec() {
            assert_eq!(
                1,
                cyclomatic(
                    "defmodule M do\n @spec merge(map(), map()) :: map()\n def merge(a, b), do: a\nend"
                )
            );
        }

        #[test]
        fn cyclo_ignores_type_names_nested_inside_a_type_declaration() {
            assert_eq!(
                1,
                cyclomatic(
                    "defmodule M do\n @type t :: %{a: map(), b: [map()]}\n def f(x), do: x\nend"
                )
            );
        }

        #[test]
        fn cyclo_ignores_type_names_inside_a_function_type() {
            let colliding = cyclomatic(
                "defmodule M do\n @spec apply_fun((map() -> map())) :: map()\n def apply_fun(f), do: f\nend",
            );
            let plain = cyclomatic(
                "defmodule M do\n @spec apply_fun((term() -> term())) :: term()\n def apply_fun(f), do: f\nend",
            );
            assert_eq!(plain, colliding);
            assert_eq!(1, colliding);
        }

        #[test]
        fn cyclo_ignores_type_names_inside_a_callback_function_type() {
            assert_eq!(
                1,
                cyclomatic("defmodule M do\n @callback handler((map() -> :ok)) :: :ok\nend")
            );
        }

        #[test]
        fn cyclo_ignores_type_names_inside_a_type_alias_function_type() {
            assert_eq!(
                1,
                cyclomatic("defmodule M do\n @type handler :: (map() -> :ok)\nend")
            );
        }

        #[test]
        fn cyclo_ignores_type_names_inside_a_multi_argument_function_type() {
            assert_eq!(
                1,
                cyclomatic(
                    "defmodule M do\n @callback reduce(Enumerable.t(), (map(), map() -> map())) :: map()\nend"
                )
            );
        }

        #[test]
        fn cyclo_ignores_type_names_in_a_spec_when_constraint() {
            assert_eq!(
                1,
                cyclomatic("defmodule M do\n @spec f(x) :: x when x: map()\n def f(x), do: x\nend")
            );
        }

        #[test]
        fn cyclo_ignores_type_names_in_union_and_container_types() {
            assert_eq!(
                1,
                cyclomatic(
                    "defmodule M do\n @type t :: map() | [map()] | {map(), map()} | %{k: map()}\n def f(x), do: x\nend"
                )
            );
        }

        #[test]
        fn cyclo_ignores_delegated_function_name() {
            let iterator_named =
                cyclomatic("defmodule M do\n defdelegate map(list, fun), to: Enum\nend");
            let plainly_named =
                cyclomatic("defmodule M do\n defdelegate transform(list, fun), to: Enum\nend");
            assert_eq!(plainly_named, iterator_named);
            assert_eq!(1, iterator_named);
        }

        #[test]
        fn cyclo_counts_iteration_in_a_function_body() {
            assert_eq!(
                2,
                cyclomatic("defmodule M do\n def go(l), do: Enum.map(l, fn n -> n end)\nend")
            );
        }

        #[test]
        fn cyclo_counts_iteration_inside_an_attribute_value() {
            assert_eq!(
                2,
                cyclomatic(
                    "defmodule M do\n @names Enum.map([1], fn n -> n end)\n def go, do: @names\nend"
                )
            );
        }

        #[test]
        fn cyclo_does_not_count_a_single_clause_function() {
            assert_eq!(1, cyclomatic("defmodule M do\n def f(x), do: x\nend"));
        }

        #[test]
        fn cyclo_counts_each_clause_after_the_first() {
            assert_eq!(
                3,
                cyclomatic(
                    "defmodule M do\n def f(0), do: 1\n def f(n), do: n\n def f(_), do: 0\nend"
                )
            );
        }

        #[test]
        fn cyclo_counts_guarded_clauses() {
            assert_eq!(
                3,
                cyclomatic(
                    "defmodule M do\n def f(x) when x < 0, do: :neg\n def f(x) when x > 0, do: :pos\n def f(_), do: :zero\nend"
                )
            );
        }

        #[test]
        fn cyclo_tracks_the_case_form_it_replaces() {
            let clauses = cyclomatic(
                "defmodule M do\n def f(:a), do: 1\n def f(:b), do: 2\n def f(_), do: 3\nend",
            );
            let arms = cyclomatic(
                "defmodule M do\n def f(x) do\n  case x do\n   :a -> 1\n   :b -> 2\n   _ -> 3\n  end\n end\nend",
            );
            assert_eq!(arms - 1, clauses);
        }

        #[test]
        fn cyclo_groups_clauses_by_arity() {
            assert_eq!(
                1,
                cyclomatic("defmodule M do\n def f(a), do: a\n def f(a, b), do: {a, b}\nend")
            );
        }

        #[test]
        fn cyclo_groups_clauses_by_name() {
            assert_eq!(
                1,
                cyclomatic("defmodule M do\n def f(a), do: a\n def g(a), do: a\nend")
            );
        }

        #[test]
        fn cyclo_does_not_group_clauses_across_modules() {
            assert_eq!(
                1,
                cyclomatic(
                    "defmodule A do\n def f(x), do: x\nend\n\ndefmodule B do\n def f(x), do: x\nend"
                )
            );
        }

        #[test]
        fn cyclo_is_unchanged_by_the_else_spelling() {
            let keyword =
                cyclomatic("defmodule M do\n def f(x) do\n  if x, do: 1, else: 2\n end\nend");
            let block = cyclomatic(
                "defmodule M do\n def f(x) do\n  if x do\n   1\n  else\n   2\n  end\n end\nend",
            );
            assert_eq!(block, keyword);
        }

        #[test]
        fn cyclo_counts_rescue_block_once() {
            assert_eq!(
                3,
                cyclomatic(
                    "defmodule M do\n def f do\n  try do\n   risky()\n  rescue\n   e -> e\n  end\n end\nend"
                )
            );
        }
    }
}
