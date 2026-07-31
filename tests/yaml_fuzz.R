args <- commandArgs(trailingOnly = TRUE)
stopifnot(length(args) == 1L, file.exists(args[[1L]]))
yamark <- normalizePath(args[[1L]], mustWork = TRUE)

stopifnot(requireNamespace("yaml12", quietly = TRUE))

fuzz_seed <- 20260731L
set.seed(fuzz_seed)

widths <- c(1, 2, 5, 10, 20, 40, 80, Inf)
contexts <- c(
  "root",
  "sequence",
  "mapping",
  "nested",
  "mapping_key",
  "tagged"
)
families <- c("paragraphs", "literal", "fallback", "arbitrary")
paragraph_words <- c(
  "alpha",
  "beta",
  "gamma",
  "punctuation,",
  "colon:inside",
  "quote\"inside",
  "back\\slash",
  "caf\u00e9",
  "e\u0301",
  "\u6f22\u5b57",
  strrep("unbreakable", 12)
)
first_lines <- c(
  "first line",
  "- outer",
  "> blockquote",
  "1. ordered item",
  "```r"
)
later_lines <- c(
  "second line",
  "  - nested",
  "    indented code",
  "\tindented with a tab",
  "",
  "line with trailing space ",
  "line with trailing tab\t"
)
unicode_whitespace_code_points <- c(
  0x0085L,
  0x00a0L,
  0x1680L,
  0x2000L:0x200aL,
  0x2028L,
  0x2029L,
  0x202fL,
  0x205fL,
  0x3000L
)
unicode_whitespace <- intToUtf8(
  unicode_whitespace_code_points,
  multiple = TRUE
)
fallback_values <- c(
  " leading space",
  "trailing space ",
  "\tleading tab",
  "trailing tab\t",
  "repeated  spaces",
  "embedded\ttab",
  "line\r\nbreak",
  "carriage\rreturn",
  "form\ffeed",
  "vertical\vtab",
  "\n",
  "\n\n",
  "value\n\n",
  "value\n\n\n",
  " \n  \n\t",
  unicode_whitespace,
  paste0(unicode_whitespace, "value"),
  paste0("value", unicode_whitespace)
)
arbitrary_tokens <- c(
  "",
  paragraph_words,
  "\U0001f642",
  "-",
  "?",
  ":",
  "#",
  "---",
  "...",
  "[value]",
  "{value}",
  "'single'",
  "\"double\"",
  "\u0001",
  "\u0007",
  "\u007f",
  unicode_whitespace
)
separators <- c(
  "",
  " ",
  "  ",
  "\t",
  "\n",
  "\n\n",
  "\n\n\n",
  " \n",
  "\n ",
  "\r\n",
  "\r",
  "\f",
  "\v",
  "\u00a0"
)
edges <- c("", " ", "\t", "\n", "\n\n", unicode_whitespace)

cases <- vector("list", 2000L)
for (case in seq_along(cases)) {
  combination <- case - 1L
  family <- families[[combination %% length(families) + 1L]]
  context <- contexts[[
    (combination %/% length(families)) %% length(contexts) + 1L
  ]]
  width <- widths[[
    (combination %/% (length(families) * length(contexts))) %%
      length(widths) +
      1L
  ]]

  value <- switch(
    family,
    paragraphs = {
      paragraph_count <- sample.int(3L, 1L)
      paragraphs <- vapply(
        seq_len(paragraph_count),
        function(i) {
          paste(
            sample(
              paragraph_words,
              sample(18L:24L, 1L),
              replace = TRUE
            ),
            collapse = " "
          )
        },
        character(1)
      )
      paste0(
        paste(paragraphs, collapse = "\n\n"),
        if (sample(c(FALSE, TRUE), 1L)) "\n" else ""
      )
    },
    literal = {
      lines <- c(
        sample(first_lines, 1L),
        sample(later_lines, sample.int(6L, 1L), replace = TRUE)
      )
      paste0(
        sample(c("", "\n", "\n\n"), 1L),
        paste(lines, collapse = "\n"),
        if (sample(c(FALSE, TRUE), 1L)) "\n" else ""
      )
    },
    fallback = sample(fallback_values, 1L),
    arbitrary = {
      token_count <- sample.int(8L, 1L)
      tokens <- sample(arbitrary_tokens, token_count, replace = TRUE)
      value <- tokens[[1L]]
      if (token_count > 1L) {
        for (i in 2L:token_count) {
          value <- paste0(value, sample(separators, 1L), tokens[[i]])
        }
      }
      paste0(sample(edges, 1L), value, sample(edges, 1L))
    }
  )

  object <- switch(
    context,
    root = value,
    sequence = list(value, "after"),
    mapping = list(value = value, after = "after"),
    nested = list(outer = list(value = value, after = "after")),
    mapping_key = setNames(list("payload"), value),
    tagged = structure(value, yaml_tag = "!generated")
  )
  cases[[case]] <- list(
    family = family,
    context = context,
    width = width,
    value = value,
    input = yaml12::format_yaml(object, width = width)
  )
}

scalar_pool <- list(
  NULL,
  TRUE,
  FALSE,
  0L,
  1L,
  -1L,
  42L,
  0,
  1.5,
  1e20,
  Inf,
  -Inf,
  NaN,
  "",
  "null",
  "true",
  "yes",
  "0x2A",
  "1.0",
  "---",
  "...",
  "foo:",
  " leading",
  "trailing ",
  "a\tb",
  "a\nb",
  "\n",
  "\n\n",
  "alpha  beta",
  "café",
  "é",
  "漢字",
  " value ",
  paste(rep("long prose value", 12L), collapse = " ")
)

random_yaml_object <- function(depth = 0L) {
  if (depth >= 4L || runif(1L) < 0.5) {
    return(scalar_pool[[sample.int(length(scalar_pool), 1L)]])
  }
  size <- sample.int(5L, 1L) - 1L
  values <- lapply(
    seq_len(size),
    function(i) random_yaml_object(depth + 1L)
  )
  if (runif(1L) < 0.5) {
    return(values)
  }
  if (size > 0L) {
    names(values) <- paste0("key", seq_len(size))
  }
  values
}

set.seed(fuzz_seed + 1L)
for (case in seq_len(1000L)) {
  object <- random_yaml_object()
  width <- widths[[(case - 1L) %% length(widths) + 1L]]
  cases[[length(cases) + 1L]] <- list(
    family = "recursive",
    context = "root",
    width = width,
    value = paste(capture.output(dput(object)), collapse = "\n"),
    input = yaml12::format_yaml(object, width = width)
  )
}

regression_inputs <- list(
  bom_block_scalar = paste0(
    "\ufeffvalue: |-\n",
    "  first\n",
    "  second\n",
    "after: y\n"
  ),
  bom_sequence_stream = paste0(
    "\ufeff- one\n",
    "---\n",
    "name:    value\n"
  ),
  bom_flow_collection = "\ufeff[a,b,{x: y}]\n",
  bom_compact_mapping = paste0(
    "\ufeffa: 1\n",
    "b: [2,3]\n"
  ),
  later_document_fe_ff = paste0(
    "first:    one\n",
    "---\n",
    "\ufeffsecond:    two\n",
    "after: y\n"
  ),
  cr_only_block_scalar = paste0(
    "outer:\r",
    "  value: |-\r",
    "    first\r",
    "    second\r",
    "  after: x\r"
  ),
  explicit_block_indent = paste0(
    "outer:\n",
    "  - value: |2-\n",
    "      first\n",
    "       \n",
    "      last\n",
    "    after: x\n"
  ),
  multiline_plain_scalar = paste0(
    "outer:\n",
    "  value: first line\n",
    "    second line\n",
    "    third line\n",
    "  after: x\n"
  ),
  block_plain_scalar_moved_inline = paste0(
    "Warning:\n",
    "  This is an error message\n",
    "  for the log file\n"
  ),
  multiline_quoted_sequence_mapping_scalar = paste0(
    "outer:\n",
    "  - value: 'first\n",
    "      second\n",
    "      third'\n",
    "    after: x\n"
  )
)
for (name in names(regression_inputs)) {
  cases[[length(cases) + 1L]] <- list(
    family = "regression",
    context = name,
    width = Inf,
    value = name,
    input = regression_inputs[[name]]
  )
}

configurations <- list(
  default = character(),
  indent4 = c("--indent-width", "4"),
  compact = "--compact",
  canonical = "--canonical",
  combined = c("--indent-width", "4", "--compact", "--canonical")
)

root <- tempfile("yamark-yaml-fuzz-")
dir.create(root)
on.exit(unlink(root, recursive = TRUE), add = TRUE)

source_root <- file.path(root, "source")
dir.create(source_root)
source_paths <- file.path(
  source_root,
  sprintf("case-%04d.yaml", seq_along(cases))
)
for (case in seq_along(cases)) {
  writeBin(charToRaw(cases[[case]]$input), source_paths[[case]])
}
before <- lapply(
  source_paths,
  yaml12::read_yaml,
  multi = TRUE,
  simplify = FALSE
)

for (configuration in names(configurations)) {
  case_root <- file.path(root, configuration)
  dir.create(case_root)
  paths <- file.path(
    case_root,
    sprintf("case-%04d.yaml", seq_along(cases))
  )
  for (case in seq_along(cases)) {
    writeBin(charToRaw(cases[[case]]$input), paths[[case]])
  }

  command_output <- system2(
    yamark,
    c("format", configurations[[configuration]], case_root),
    stdout = TRUE,
    stderr = TRUE
  )
  status <- attr(command_output, "status")
  if (is.null(status)) {
    status <- 0L
  }
  if (!identical(status, 0L)) {
    stop(paste(command_output, collapse = "\n"), call. = FALSE)
  }

  for (case in seq_along(cases)) {
    output <- readChar(
      paths[[case]],
      file.info(paths[[case]])$size,
      useBytes = TRUE
    )
    after <- tryCatch(
      yaml12::read_yaml(paths[[case]], multi = TRUE, simplify = FALSE),
      error = identity
    )
    if (!inherits(after, "error") && identical(before[[case]], after)) {
      next
    }

    details <- cases[[case]]
    before_text <- paste(
      capture.output(dput(before[[case]])),
      collapse = "\n"
    )
    after_text <- if (inherits(after, "error")) {
      conditionMessage(after)
    } else {
      paste(capture.output(dput(after)), collapse = "\n")
    }
    stop(
      sprintf(
        paste0(
          "YAML value changed after formatting\n",
          "seed: %d\nconfiguration: %s\ncase: %d\n",
          "family: %s\ncontext: %s\nwidth: %s\n",
          "value: %s\ninput:\n%s\noutput:\n%s\nbefore:\n%s\nafter:\n%s"
        ),
        fuzz_seed,
        configuration,
        case,
        details$family,
        details$context,
        details$width,
        encodeString(details$value, quote = "\""),
        details$input,
        output,
        before_text,
        after_text
      ),
      call. = FALSE
    )
  }
}

cat(sprintf(
  "YAML fuzz cases passed: %d cases x %d configurations\n",
  length(cases),
  length(configurations)
))
