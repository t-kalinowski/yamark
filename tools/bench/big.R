#!/usr/bin/env Rscript

# 4 MB keeps every comparison tool in play: panache refuses inputs larger
# than 4 MiB (PreallocationSizeLimit).
DEFAULT_MARKDOWN_BYTES <- 4000000
DEFAULT_YAML_BYTES <- 4000000
DEFAULT_FRONTMATTER_BYTES <- 4000000
DEFAULT_FRONTMATTER_YAML_RATIO <- 0.05
DEFAULT_SEED <- 20260602L
DEFAULT_OUT_DIR <- file.path("target", "bench-big", "corpus")

BIG_MARKDOWN <- "big.md"
BIG_YAML <- "big.yaml"
BIG_FRONTMATTER <- "big-with-frontmatter.md"

generate_big_corpus <- function(
  out_dir = DEFAULT_OUT_DIR,
  markdown_bytes = DEFAULT_MARKDOWN_BYTES,
  yaml_bytes = DEFAULT_YAML_BYTES,
  frontmatter_bytes = DEFAULT_FRONTMATTER_BYTES,
  frontmatter_yaml_bytes = NULL,
  seed = DEFAULT_SEED
) {
  stopifnot(is.character(out_dir), length(out_dir) == 1L, nzchar(out_dir))
  stopifnot(markdown_bytes > 0, yaml_bytes > 0, frontmatter_bytes > 0)
  if (is.null(frontmatter_yaml_bytes) || is.na(frontmatter_yaml_bytes)) {
    frontmatter_yaml_bytes <- default_frontmatter_yaml_bytes(frontmatter_bytes)
  }
  stopifnot(frontmatter_yaml_bytes > 0, frontmatter_yaml_bytes < frontmatter_bytes)
  stopifnot(length(seed) == 1L, !is.na(seed))
  stopifnot(requireNamespace("stringi", quietly = TRUE))
  stopifnot(requireNamespace("yaml12", quietly = TRUE))

  dir.create(out_dir, recursive = TRUE, showWarnings = FALSE)

  paths <- c(
    markdown = file.path(out_dir, BIG_MARKDOWN),
    yaml = file.path(out_dir, BIG_YAML),
    frontmatter = file.path(out_dir, BIG_FRONTMATTER)
  )

  write_big_markdown(paths[["markdown"]], markdown_bytes, seed)
  write_big_yaml(paths[["yaml"]], yaml_bytes, seed + 1L)
  write_big_frontmatter(
    paths[["frontmatter"]],
    frontmatter_bytes,
    seed + 2L,
    yaml_bytes = frontmatter_yaml_bytes
  )

  invisible(paths)
}

# 5% of bytes lands at roughly a third of the document's lines in YAML,
# since generated YAML lines are much shorter than generated Markdown prose
# lines.
default_frontmatter_yaml_bytes <- function(frontmatter_bytes) {
  max(
    1,
    min(
      frontmatter_bytes - 1,
      round(frontmatter_bytes * DEFAULT_FRONTMATTER_YAML_RATIO)
    )
  )
}

write_big_markdown <- function(path, target_bytes, seed) {
  con <- file(path, open = "wb")
  on.exit(close(con), add = TRUE)
  set.seed(seed)
  write_markdown_content(con, target_bytes, include_heading = TRUE)
}

write_big_yaml <- function(path, target_bytes, seed) {
  con <- file(path, open = "wb")
  on.exit(close(con), add = TRUE)
  set.seed(seed)
  write_yaml_content(con, target_bytes)
}

write_big_frontmatter <- function(path, target_bytes, seed, yaml_bytes) {
  con <- file(path, open = "wb")
  on.exit(close(con), add = TRUE)
  set.seed(seed)

  written <- write_text(con, "---\n")
  written <- written + write_text(con, frontmatter_preamble())
  written <- written + write_text(con, "contents:\n")
  yaml_target <- max(1, yaml_bytes - written)
  written <- written + write_yaml_content(con, yaml_target, indent = "  ")
  written <- written + write_text(con, "---\n\n")

  markdown_target <- max(1, target_bytes - written)
  write_markdown_content(con, markdown_target, include_heading = TRUE)
}

# Deliberately unformatted: alignment padding and cramped flow collections,
# so any formatter that touches front matter must rewrite the block.
frontmatter_preamble <- function() {
  paste(
    c(
      'title:   "Generated benchmark document"',
      "tags: [benchmarks,yaml,markdown]",
      "params: {retries: 3,timeout: 30s}",
      ""
    ),
    collapse = "\n"
  )
}

write_markdown_content <- function(con, target_bytes, include_heading) {
  bytes <- 0
  paragraph_index <- 0L
  list_index <- 0L

  if (include_heading) {
    bytes <- bytes + write_text(con, "# Generated Markdown Benchmark Document\n\n")
  }

  while (bytes < target_bytes) {
    paragraph_count <- sample(3:5, 1L)
    for (i in seq_len(paragraph_count)) {
      paragraph_index <- paragraph_index + 1L
      paragraph <- markdown_paragraph(paragraph_index)
      bytes <- bytes + write_text(con, paste0(paragraph, "\n\n"))
      if (bytes >= target_bytes) {
        break
      }
    }

    list_index <- list_index + 1L
    bytes <- bytes + write_text(con, markdown_list_block(list_index))
  }

  bytes
}

markdown_paragraph <- function(index) {
  paragraph <- stringi::stri_rand_lipsum(
    1L,
    start_lipsum = identical(index, 1L)
  )
  decorate_paragraph(paragraph, index)
}

decorate_paragraph <- function(paragraph, index) {
  if (index == 1L) {
    return(paste(
      paragraph,
      markdown_link("short", index),
      "continues after the short link."
    ))
  }
  if (index == 2L) {
    return(paste(
      paragraph,
      markdown_link("long", index),
      "continues after the unwrappable link."
    ))
  }
  if (index == 3L) {
    return(paste(
      paragraph,
      markdown_link("short", index),
      "and",
      markdown_link("long", index),
      "both appear in one sentence."
    ))
  }

  mode <- sample(c("none", "short", "long", "multi"), 1L, prob = c(5, 3, 1, 1))
  if (identical(mode, "short")) {
    return(paste(paragraph, markdown_link("short", index)))
  }
  if (identical(mode, "long")) {
    return(paste(paragraph, markdown_link("long", index)))
  }
  if (identical(mode, "multi")) {
    return(paste(
      paragraph,
      markdown_link("short", index),
      "and",
      markdown_link("short", index + 10000L)
    ))
  }
  paragraph
}

markdown_link <- function(kind, index) {
  if (identical(kind, "short")) {
    return(sprintf(
      "[short reference](https://example.com/short/%05d)",
      as.integer(index)
    ))
  }

  slug <- paste(rep(sprintf("segment-%05d", as.integer(index)), 18L), collapse = "-")
  sprintf(
    "[long unwrappable reference](https://example.com/long/%s)",
    slug
  )
}

markdown_list_block <- function(list_index) {
  styles <- list_styles()
  top_style <- styles[((list_index - 1L) %% length(styles)) + 1L]
  nested_style <- styles[(list_index %% length(styles)) + 1L]
  deep_style <- styles[((list_index + 1L) %% length(styles)) + 1L]
  out <- character()

  for (item in 1:5) {
    out <- c(out, list_item_line(0L, top_style, item, list_index, "top"))
    if (item %in% c(2L, 4L)) {
      for (nested in 1:5) {
        out <- c(out, list_item_line(1L, nested_style, nested, list_index, "nested"))
        if (item == 4L && nested == 3L) {
          for (deep in 1:5) {
            out <- c(out, list_item_line(2L, deep_style, deep, list_index, "deep"))
          }
        }
      }
    }
  }

  paste0(paste(out, collapse = "\n"), "\n\n")
}

list_item_line <- function(depth, style, item, list_index, label) {
  indent <- paste(rep("  ", depth), collapse = "")
  text <- sprintf(
    "%s item %02d-%02d carries lorem prose and %s.",
    label,
    as.integer(list_index),
    as.integer(item),
    markdown_link(if (item %% 2L == 0L) "short" else "long", list_index * 100L + item)
  )
  paste0(indent, list_marker(style, item), " ", text)
}

list_styles <- function() {
  c(
    "dash",
    "asterisk",
    "plus",
    "decimal-dot",
    "decimal-paren",
    "parenthesized-decimal",
    "alpha",
    "roman"
  )
}

list_marker <- function(style, item) {
  if (identical(style, "dash")) {
    return("-")
  }
  if (identical(style, "asterisk")) {
    return("*")
  }
  if (identical(style, "plus")) {
    return("+")
  }
  if (identical(style, "decimal-dot")) {
    return(sprintf("%d.", as.integer(item)))
  }
  if (identical(style, "decimal-paren")) {
    return(sprintf("%d)", as.integer(item)))
  }
  if (identical(style, "parenthesized-decimal")) {
    return(sprintf("(%d)", as.integer(item)))
  }
  if (identical(style, "alpha")) {
    return(sprintf("%s.", letters[[item]]))
  }
  if (identical(style, "roman")) {
    numerals <- c("I", "II", "III", "IV", "V")
    return(sprintf("%s)", numerals[[item]]))
  }
  stop("unknown list style: ", style, call. = FALSE)
}

write_yaml_content <- function(con, target_bytes, indent = "") {
  bytes <- 0
  node <- yaml_mixed_node_template()
  node_lines <- strsplit(node, "\n", fixed = TRUE)[[1]]
  node_index <- 0L

  while (bytes < target_bytes) {
    node_index <- node_index + 1L
    bytes <- bytes + write_yaml_node(con, node_lines, node_index, indent)
  }

  bytes
}

write_yaml_node <- function(con, node_lines, node_index, indent = "") {
  bytes <- 0
  comment_before <- sort(unique(c(
    1L,
    sample(seq_along(node_lines), size = 2L)
  )))

  for (line_index in seq_along(node_lines)) {
    line <- paste0(indent, node_lines[[line_index]])
    if (line_index %in% comment_before) {
      comment_indent <- sub("^(\\s*).*", "\\1", line)
      comment <- sprintf(
        "%s# benchmark comment %06d.%02d\n",
        comment_indent,
        as.integer(node_index),
        as.integer(line_index)
      )
      bytes <- bytes + write_text(con, comment)
    }
    bytes <- bytes + write_text(con, paste0(line, "\n"))
  }

  bytes
}

yaml_mixed_node_template <- function() {
  mixed_node <- list(
    str = c(
      "Lorem ipsum dolor sit amet, vel accumsan vitae faucibus ultrices leo",
      "neque? Et cursus lacinia, ut, sit donec facilisi eu interdum. Dui",
      "ipsum, vitae ligula commodo convallis ac sed nunc. Ipsum at nec lacus",
      "eros suscipit vitae."
    ),
    block_str = "lorem \n ipsum \n dolor\n",
    bools = c(TRUE, FALSE),
    ints = c(123L, -123L),
    floats = c(123.456, -123.456),
    null = NULL
  )
  yaml12::format_yaml(list(mixed_node))
}

write_text <- function(con, text) {
  text <- enc2utf8(text)
  raw <- charToRaw(text)
  writeBin(raw, con)
  length(raw)
}

parse_args <- function(args) {
  out <- list(
    out_dir = DEFAULT_OUT_DIR,
    markdown_bytes = DEFAULT_MARKDOWN_BYTES,
    yaml_bytes = DEFAULT_YAML_BYTES,
    frontmatter_bytes = DEFAULT_FRONTMATTER_BYTES,
    frontmatter_yaml_bytes = NULL,
    seed = DEFAULT_SEED
  )

  index <- 1L
  while (index <= length(args)) {
    arg <- args[[index]]
    if (arg %in% c(
      "--out-dir", "--markdown-bytes", "--yaml-bytes",
      "--frontmatter-bytes", "--frontmatter-yaml-bytes",
      "--target-bytes", "--seed"
    )) {
      index <- index + 1L
      stopifnot(index <= length(args))
      name <- sub("^--", "", arg)
      name <- gsub("-", "_", name)
      value <- args[[index]]
      if (identical(name, "target_bytes")) {
        value <- as.numeric(value)
        out$markdown_bytes <- value
        out$yaml_bytes <- value
        out$frontmatter_bytes <- value
      } else if (identical(name, "out_dir")) {
        out$out_dir <- value
      } else {
        out[[name]] <- as.numeric(value)
      }
    } else {
      stop("unknown argument: ", arg, call. = FALSE)
    }
    index <- index + 1L
  }

  out$seed <- as.integer(out$seed)
  out
}

main <- function() {
  args <- parse_args(commandArgs(trailingOnly = TRUE))
  paths <- generate_big_corpus(
    out_dir = args$out_dir,
    markdown_bytes = args$markdown_bytes,
    yaml_bytes = args$yaml_bytes,
    frontmatter_bytes = args$frontmatter_bytes,
    frontmatter_yaml_bytes = args$frontmatter_yaml_bytes,
    seed = args$seed
  )

  for (name in names(paths)) {
    size <- file.info(paths[[name]])$size
    cat(sprintf("%s: %s bytes\n", paths[[name]], format(size, scientific = FALSE)))
  }
}

if (identical(environment(), globalenv()) && !length(sys.frames())) {
  main()
}
