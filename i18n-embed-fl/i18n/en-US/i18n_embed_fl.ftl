hello-world = Hello World!
hello-arg = Hello {$name}!
    .attr = Hello {$name}'s attribute!
hello-arg-2 = Hello {$name1} and {$name2}!
hello-attr = Uninspiring.
    .text = Hello, attribute!
hello-recursive = Hello { hello-recursive-descent }
    .attr = Why hello { hello-recursive-descent }
    .again = Why hello { hello-recursive-descent.attr }
hello-recursive-descent = to you, {$name}!
    .attr = again, {$name}!
hello-select = { $attr ->
    *[no] { hello-recursive }
    [yes] { hello-recursive.attr }
}
