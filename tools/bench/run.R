#!/usr/bin/env Rscript

run_yaml_benchmark <- function(
  invocation = c("per-file", "directory"),
  tools = "yamark",
  files = 400L,
  items = 80L,
  reps = 2L,
  warmups = 1L,
  artifact_dir = file.path("docs", "benchmarks", "yaml"),
  out_dir = file.path("target", "bench-yaml"),
  extra_args = character()
) {
  invocation <- match.arg(invocation)
  stopifnot(length(tools) <= 1L)
  stopifnot(files > 0L)
  stopifnot(items > 0L)
  stopifnot(reps > 0L)
  stopifnot(warmups >= 0L)

  args <- c(
    "tools/bench/run.py",
    "--invocation", invocation,
    "--files", as.character(as.integer(files)),
    "--items", as.character(as.integer(items)),
    "--reps", as.character(as.integer(reps)),
    "--warmups", as.character(as.integer(warmups)),
    "--out-dir", out_dir,
    "--artifact-dir", artifact_dir
  )
  if (!is.null(tools)) {
    args <- c(args, "--tools", tools)
  }
  args <- c(args, extra_args)

  status <- system2(command = python_bin(), args = args)
  stopifnot(identical(status, 0L))
  invisible(status)
}

python_bin <- function() {
  python <- Sys.getenv("PYTHON", unset = "")
  if (nzchar(python)) python else "python3"
}

parse_args <- function(args) {
  out <- list(
    invocation = "per-file",
    tools = "yamark",
    files = 400L,
    items = 80L,
    reps = 2L,
    warmups = 1L,
    artifact_dir = file.path("docs", "benchmarks", "yaml"),
    out_dir = file.path("target", "bench-yaml"),
    extra_args = character()
  )

  index <- 1L
  while (index <= length(args)) {
    arg <- args[[index]]
    if (arg %in% c(
      "--invocation", "--tools", "--files", "--items", "--reps", "--warmups",
      "--artifact-dir", "--out-dir"
    )) {
      index <- index + 1L
      stopifnot(index <= length(args))
      name <- sub("^--", "", arg)
      name <- gsub("-", "_", name)
      out[[name]] <- args[[index]]
    } else {
      out$extra_args <- c(out$extra_args, arg)
    }
    index <- index + 1L
  }

  out$files <- as.integer(out$files)
  out$items <- as.integer(out$items)
  out$reps <- as.integer(out$reps)
  out$warmups <- as.integer(out$warmups)
  out
}

main <- function() {
  args <- parse_args(commandArgs(trailingOnly = TRUE))
  run_yaml_benchmark(
    invocation = args$invocation,
    tools = args$tools,
    files = args$files,
    items = args$items,
    reps = args$reps,
    warmups = args$warmups,
    artifact_dir = args$artifact_dir,
    out_dir = args$out_dir,
    extra_args = args$extra_args
  )
}

if (identical(environment(), globalenv()) && !length(sys.frames())) {
  main()
}
